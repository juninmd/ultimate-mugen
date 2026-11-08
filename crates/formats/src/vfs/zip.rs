use super::archive::ZipArchive;
use super::{join, text, File, Source, Vfs};
use crate::Reader;
use std::sync::Arc;

const EOCD_SIG: u32 = 0x0605_4b50;
const EOCD64_LOC_SIG: u32 = 0x0706_4b50;
const EOCD64_SIG: u32 = 0x0606_4b50;
const CDIR_SIG: u32 = 0x0201_4b50;
const LOCAL_SIG: u32 = 0x0403_4b50;
const MAX_ENTRY_BYTES: usize = 256 << 20;

pub(super) struct ZipEntry {
    name: String,
    method: u16,
    csize: u64,
    usize: u64,
    local: u64,
}

/// Import progress shared with the UI while an in-memory archive is unpacked.
#[derive(Clone, Default)]
pub struct ZipProgress {
    pub done: Arc<std::sync::atomic::AtomicUsize>,
    pub total: Arc<std::sync::atomic::AtomicUsize>,
    pub current_name: Arc<std::sync::Mutex<String>>,
}

/// Read every file of a zip archive into `vfs` under `prefix` (may be empty).
/// Directory entries are skipped. Supports stored and deflated entries (and
/// zip64). Returns the number of files added.
pub fn add_zip(vfs: &mut Vfs, bytes: &[u8], prefix: &str) -> Result<usize, String> {
    add_zip_with_progress(vfs, bytes, prefix, None)
}

/// Same as [`add_zip`], reporting progress entry by entry. Archives that arrive
/// as bytes (web imports, bundled packs) are unpacked eagerly so the caller can
/// show a progress bar and the bytes can be released afterwards.
pub fn add_zip_with_progress(
    vfs: &mut Vfs,
    bytes: &[u8],
    prefix: &str,
    progress: Option<&ZipProgress>,
) -> Result<usize, String> {
    use std::sync::atomic::Ordering::Relaxed;
    let archive = Arc::new(bytes.to_vec());
    let entries = zip_entries(
        |at, len| {
            let start = usize::try_from(at).ok()?;
            Some(archive.get(start..start.checked_add(len)?)?.to_vec())
        },
        archive.len() as u64,
    )?;
    if let Some(p) = progress {
        p.total.store(entries.len(), Relaxed);
    }
    let source = ZipArchive::Bytes(archive);
    let mut added = 0;
    for e in &entries {
        let data = read_zip_member(&source, e.method, e.csize, e.usize, e.local)
            .ok_or_else(|| format!("zip: {}: could not read entry", e.name))?;
        if let Some(p) = progress {
            *p.current_name.lock().unwrap_or_else(|e| e.into_inner()) = e.name.clone();
            p.done.fetch_add(1, Relaxed);
        }
        vfs.insert(join(prefix, &e.name), data);
        added += 1;
    }
    Ok(added)
}

pub(super) fn index_entries(
    vfs: &mut Vfs,
    archive: ZipArchive,
    entries: &[ZipEntry],
    prefix: &str,
) -> usize {
    let mut added = 0;
    for e in entries {
        let source = Source::Zip {
            archive: archive.clone(),
            method: e.method,
            csize: e.csize,
            usize: e.usize,
            local: e.local,
        };
        vfs.put(join(prefix, &e.name), File::new(source));
        added += 1;
    }
    added
}

/// Index a zip file on disk by reading only its end record and central
/// directory. Member bytes stay on disk until read.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn add_zip_file(
    vfs: &mut Vfs,
    path: &std::path::Path,
    prefix: &str,
) -> std::io::Result<usize> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len < 22 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "zip: file too small",
        ));
    }
    let tail_len = (22 + 0xffffu64).min(len);
    let mut tail = vec![0u8; tail_len as usize];
    file.seek(SeekFrom::Start(len - tail_len))?;
    file.read_exact(&mut tail)?;
    let base = len - tail_len;
    let entries = zip_entries(
        |at, n| {
            // the end record lives in the buffered tail; the central directory can
            // sit anywhere before it, so everything else is a positioned read
            if at >= base {
                let off = usize::try_from(at - base).ok()?;
                return tail.get(off..off.checked_add(n)?).map(|s| s.to_vec());
            }
            super::archive::read_at(&mut file, at, n)
        },
        len,
    )
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let archive = ZipArchive::Disk(Arc::new(path.to_path_buf()));
    Ok(index_entries(vfs, archive, &entries, prefix))
}

struct Probe<F>(F);

impl<F: FnMut(u64, usize) -> Option<Vec<u8>>> Probe<F> {
    fn u16(&mut self, at: u64) -> Option<u16> {
        (self.0)(at, 2).and_then(|b| Some(u16::from_le_bytes(b.get(..2)?.try_into().ok()?)))
    }
    fn u32(&mut self, at: u64) -> Option<u32> {
        (self.0)(at, 4).and_then(|b| Some(u32::from_le_bytes(b.get(..4)?.try_into().ok()?)))
    }
    fn u64(&mut self, at: u64) -> Option<u64> {
        (self.0)(at, 8).and_then(|b| Some(u64::from_le_bytes(b.get(..8)?.try_into().ok()?)))
    }
}

/// Parse the end-of-central-directory record and every central directory
/// entry, using `read(at, len)` for random access into the archive.
fn zip_entries(
    read: impl FnMut(u64, usize) -> Option<Vec<u8>>,
    size: u64,
) -> Result<Vec<ZipEntry>, String> {
    let mut probe = Probe(read);
    let min = size.saturating_sub(22 + 0xffff);
    let mut eocd = None;
    let mut p = size.checked_sub(22).ok_or("zip: file too small")?;
    loop {
        if probe.u32(p) == Some(EOCD_SIG) && p + 22 + probe.u16(p + 20).unwrap_or(0) as u64 <= size
        {
            eocd = Some(p);
            break;
        }
        if p == min {
            break;
        }
        p -= 1;
    }
    let eocd = eocd.ok_or("zip: end of central directory not found")?;
    let mut entries = probe.u16(eocd + 10).unwrap_or(0) as u64;
    let mut cd_ofs = probe.u32(eocd + 16).unwrap_or(0) as u64;
    let mut cd_size = probe.u32(eocd + 12).unwrap_or(0) as u64;
    if (entries == 0xffff || cd_ofs == 0xffff_ffff)
        && eocd >= 20
        && probe.u32(eocd - 20) == Some(EOCD64_LOC_SIG)
    {
        let z64 = probe.u64(eocd - 20 + 8).unwrap_or(0);
        if probe.u32(z64) == Some(EOCD64_SIG) {
            entries = probe.u64(z64 + 32).unwrap_or(entries);
            cd_size = probe.u64(z64 + 40).unwrap_or(cd_size);
            cd_ofs = probe.u64(z64 + 48).unwrap_or(cd_ofs);
        }
    }
    let dir = (probe.0)(
        cd_ofs,
        usize::try_from(cd_size).map_err(|_| "zip: central directory too large")?,
    )
    .ok_or("zip: truncated central directory")?;
    let mut out = Vec::with_capacity(entries.min(1 << 20) as usize);
    let mut p = 0usize;
    for _ in 0..entries {
        let r = Reader(&dir);
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
        let name_bytes = dir
            .get(name_start..name_start + name_len)
            .ok_or("zip: truncated central directory")?;
        let extra = dir
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
            e += 4 + sz;
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
        if usize_ > MAX_ENTRY_BYTES as u64 {
            return Err(format!("zip: {name}: entry too large"));
        }
        if method == 0 && csize > MAX_ENTRY_BYTES as u64 {
            return Err(format!("zip: {name}: entry too large"));
        }
        if method != 0 && method != 8 {
            return Err(format!(
                "zip: {name}: unsupported compression method {method}"
            ));
        }
        out.push(ZipEntry {
            name,
            method,
            csize,
            usize: usize_,
            local,
        });
    }
    Ok(out)
}

pub(super) fn read_zip_member(
    archive: &ZipArchive,
    method: u16,
    csize: u64,
    usize_: u64,
    local: u64,
) -> Option<Vec<u8>> {
    let read = |at: u64, len: usize| -> Option<Vec<u8>> {
        match archive {
            ZipArchive::Bytes(b) => {
                let start = usize::try_from(at).ok()?;
                Some(b.get(start..start.checked_add(len)?)?.to_vec())
            }
            #[cfg(not(target_arch = "wasm32"))]
            ZipArchive::Disk(path) => {
                let mut file = std::fs::File::open(&**path).ok()?;
                super::archive::read_at(&mut file, at, len)
            }
        }
    };
    if csize > MAX_ENTRY_BYTES as u64 {
        return None;
    }
    let header = read(local, 30)?;
    let r = Reader(&header);
    if r.u32(0) != LOCAL_SIG {
        return None;
    }
    let data_start = local + 30 + r.u16(26) as u64 + r.u16(28) as u64;
    let data = read(data_start, usize::try_from(csize).ok()?)?;
    match method {
        0 => Some(data),
        8 => inflate_raw(&data, usize::try_from(usize_).unwrap_or(0)).ok(),
        _ => None,
    }
}

fn inflate_raw(data: &[u8], size_hint: usize) -> Result<Vec<u8>, String> {
    use miniz_oxide::inflate::core::{decompress, inflate_flags, DecompressorOxide};
    use miniz_oxide::inflate::TINFLStatus;
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
