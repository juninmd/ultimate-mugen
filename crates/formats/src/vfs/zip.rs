use super::{join, text, Vfs};
use crate::Reader;
use std::sync::Arc;
const EOCD_SIG: u32 = 0x0605_4b50;
const EOCD64_LOC_SIG: u32 = 0x0706_4b50;
const EOCD64_SIG: u32 = 0x0606_4b50;
const CDIR_SIG: u32 = 0x0201_4b50;
const LOCAL_SIG: u32 = 0x0403_4b50;
const MAX_ENTRY_BYTES: usize = 256 << 20;

#[derive(Clone, Default)]
pub struct ZipProgress {
    pub done: Arc<std::sync::atomic::AtomicUsize>,
    pub total: Arc<std::sync::atomic::AtomicUsize>,
    pub current_name: Arc<std::sync::Mutex<String>>,
}

struct RawZipEntry<'a> {
    name: String,
    method: u16,
    data: &'a [u8],
    usize_: usize,
}

/// Read every file of a zip archive into `vfs` under `prefix` (may be empty).
/// Directory entries are skipped. Supports stored and deflated entries (and
/// zip64). Returns the number of files added.
pub fn add_zip(vfs: &mut Vfs, bytes: &[u8], prefix: &str) -> Result<usize, String> {
    add_zip_with_progress(vfs, bytes, prefix, None)
}

pub fn add_zip_with_progress(
    vfs: &mut Vfs,
    bytes: &[u8],
    prefix: &str,
    progress: Option<&ZipProgress>,
) -> Result<usize, String> {
    let r = Reader(bytes);
    if bytes.len() < 22 {
        return Err("zip: file too small".into());
    }
    // locate the end of central directory record (may be followed by a comment)
    let min = bytes.len().saturating_sub(22 + 0xffff);
    let mut eocd = None;
    let mut p = bytes.len() - 22;
    loop {
        if r.u32(p) == EOCD_SIG && p + 22 + r.u16(p + 20) as usize <= bytes.len() {
            eocd = Some(p);
            break;
        }
        if p == min {
            break;
        }
        p -= 1;
    }
    let eocd = eocd.ok_or("zip: end of central directory not found")?;
    let mut entries = r.u16(eocd + 10) as u64;
    let mut cd_ofs = r.u32(eocd + 16) as u64;
    if (entries == 0xffff || cd_ofs == 0xffff_ffff)
        && eocd >= 20
        && r.u32(eocd - 20) == EOCD64_LOC_SIG
    {
        let z64 = r.u64(eocd - 20 + 8) as usize;
        if r.u32(z64) == EOCD64_SIG {
            entries = r.u64(z64 + 32);
            cd_ofs = r.u64(z64 + 48);
        }
    }
    let mut p = usize::try_from(cd_ofs).map_err(|_| "zip: bad central directory offset")?;
    let cap = (entries as usize).min(bytes.len() / 46 + 1);
    let mut raw_entries: Vec<RawZipEntry> = Vec::with_capacity(cap);

    for _ in 0..entries {
        if r.u32(p) != CDIR_SIG {
            return Err("zip: corrupt central directory".into());
        }
        let flags = r.u16(p + 8);
        let method = r.u16(p + 10);
        let mut csize = r.u32(p + 20) as u64;
        let mut usize_ = r.u32(p + 24) as u64;
        let name_len = r.u16(p + 28) as usize;
        let extra_len = r.u16(p + 30) as usize;
        let comment_len = r.u16(p + 32) as usize;
        let mut local = r.u32(p + 42) as u64;
        let name_start = p + 46;
        let name_bytes = bytes
            .get(name_start..name_start + name_len)
            .ok_or("zip: truncated central directory")?;
        // zip64 extended information
        let extra = bytes
            .get(name_start + name_len..name_start + name_len + extra_len)
            .unwrap_or(&[]);
        let mut e = 0;
        while e + 4 <= extra.len() {
            let (id, sz) = (
                u16::from_le_bytes([extra[e], extra[e + 1]]),
                u16::from_le_bytes([extra[e + 2], extra[e + 3]]) as usize,
            );
            if id == 1 {
                let er = Reader(&extra[e + 4..(e + 4 + sz).min(extra.len())]);
                let mut q = 0;
                if usize_ == 0xffff_ffff {
                    usize_ = er.u64(q);
                    q += 8;
                }
                if csize == 0xffff_ffff {
                    csize = er.u64(q);
                    q += 8;
                }
                if local == 0xffff_ffff {
                    local = er.u64(q);
                }
            }
            e = e.saturating_add(4).saturating_add(sz);
        }
        p = name_start + name_len + extra_len + comment_len;

        let name = if flags & 0x800 != 0 {
            String::from_utf8_lossy(name_bytes).into_owned()
        } else {
            text(name_bytes)
        };
        if name.ends_with('/') || name.ends_with('\\') {
            continue;
        }
        if flags & 1 != 0 {
            return Err(format!("zip: {name}: encrypted entries are not supported"));
        }
        let local = usize::try_from(local).map_err(|_| "zip: bad offset")?;
        if r.u32(local) != LOCAL_SIG {
            return Err(format!("zip: {name}: bad local header"));
        }
        let data_start = local.saturating_add(30 + r.u16(local + 26) as usize + r.u16(local + 28) as usize);
        let csize = usize::try_from(csize).map_err(|_| "zip: entry too large")?;
        if method == 0 && csize > MAX_ENTRY_BYTES {
            return Err(format!("zip: {name}: entry too large"));
        }
        let data = bytes
            .get(data_start..data_start.saturating_add(csize))
            .ok_or_else(|| format!("zip: {name}: truncated data"))?;
        raw_entries.push(RawZipEntry {
            name,
            method,
            data,
            usize_: usize::try_from(usize_).unwrap_or(0),
        });
    }

    if let Some(prog) = progress {
        prog.total.store(raw_entries.len(), std::sync::atomic::Ordering::Relaxed);
    }

    let decompressed: Vec<Result<(String, Vec<u8>), String>> = crate::par::map(&raw_entries, |entry| {
        if let Some(prog) = progress {
            if let Ok(mut cur) = prog.current_name.try_lock() {
                *cur = entry.name.clone();
            }
        }
        let content = match entry.method {
            0 => entry.data.to_vec(),
            8 => inflate_raw(entry.data, entry.usize_)
                .map_err(|e| format!("zip: {}: {e}", entry.name))?,
            m => return Err(format!("zip: {}: unsupported compression method {m}", entry.name)),
        };
        if let Some(prog) = progress {
            prog.done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Ok((entry.name.clone(), content))
    });

    let mut added = 0;
    for r in decompressed {
        let (name, content) = r?;
        vfs.insert(join(prefix, &name), content);
        added += 1;
    }
    Ok(added)
}

fn inflate_raw(data: &[u8], size_hint: usize) -> Result<Vec<u8>, String> {
    use miniz_oxide::inflate::core::{decompress, inflate_flags, DecompressorOxide};
    use miniz_oxide::inflate::TINFLStatus;
    // fast path: exact-size output buffer from the declared size
    if size_hint > 0
        && size_hint <= MAX_ENTRY_BYTES
        && size_hint <= data.len().saturating_mul(64).max(1 << 20)
    {
        let mut out = vec![0u8; size_hint];
        let mut st = DecompressorOxide::new();
        let (status, _, written) = decompress(
            &mut st,
            data,
            &mut out,
            0,
            inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF,
        );
        if status == TINFLStatus::Done && written == size_hint {
            return Ok(out);
        }
    }
    miniz_oxide::inflate::decompress_to_vec_with_limit(data, MAX_ENTRY_BYTES)
        .map_err(|e| format!("inflate: {:?}", e.status))
}