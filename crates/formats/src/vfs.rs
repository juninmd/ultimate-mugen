//! In-memory virtual file system (port of the file-resolution parts of
//! `src/mugen/loader.ts`) plus a tiny zip reader (stored + deflate, zip64).

use crate::ini::unquote;
use crate::Reader;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Keys: normalized (lowercase, `/`-separated, relative) paths.
pub type Vfs = BTreeMap<String, Vec<u8>>;

/// `\` -> `/`, strip a leading `./`, collapse repeated `/`, lowercase.
pub fn norm_path(p: &str) -> String {
    if p.is_ascii() {
        let p = p
            .strip_prefix("./")
            .or_else(|| p.strip_prefix(".\\"))
            .unwrap_or(p);
        let mut out = Vec::with_capacity(p.len());
        let mut last_slash = false;
        for b in p.bytes() {
            let b = if b == b'\\' {
                b'/'
            } else {
                b.to_ascii_lowercase()
            };
            if b == b'/' && last_slash {
                continue;
            }
            last_slash = b == b'/';
            out.push(b);
        }
        return unsafe { String::from_utf8_unchecked(out) };
    }
    let s = p.replace('\\', "/");
    let s = s.strip_prefix("./").unwrap_or(&s);
    let mut out = String::with_capacity(s.len());
    let mut last_slash = false;
    for c in s.chars() {
        if c == '/' {
            if last_slash {
                continue;
            }
            last_slash = true;
        } else {
            last_slash = false;
        }
        out.push(c);
    }
    out.to_lowercase()
}

pub fn join(prefix: &str, name: &str) -> String {
    let prefix = prefix.trim_end_matches(['/', '\\']);
    if prefix.is_empty() {
        norm_path(name)
    } else {
        norm_path(&format!("{prefix}/{name}"))
    }
}

/// Latin-1 decode (strips a UTF-8 BOM).
pub fn text(b: &[u8]) -> String {
    let b = b.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(b);
    if b.is_ascii() {
        return unsafe { std::str::from_utf8_unchecked(b) }.to_string();
    }
    let mut s = String::with_capacity(b.len() + (b.len() >> 2));
    for &c in b {
        s.push(c as char);
    }
    s
}

// zip
// ---------------------------------------------------------------------------

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

/// Recursively add every file under `dir` (keys relative to `dir`, under
/// `prefix`). `.zip` files are expanded in place (their entries are keyed
/// relative to the zip's folder); unreadable zips are stored as plain files.
#[cfg(not(target_arch = "wasm32"))]
pub fn add_dir(vfs: &mut Vfs, dir: &std::path::Path, prefix: &str) -> std::io::Result<usize> {
    fn walk(vfs: &mut Vfs, dir: &std::path::Path, rel: &str, n: &mut usize) -> std::io::Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let fname = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            let child = if rel.is_empty() {
                fname.clone()
            } else {
                format!("{rel}/{fname}")
            };
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            if meta.is_dir() {
                walk(vfs, &path, &child, n)?;
            } else if meta.is_file() {
                let Ok(data) = std::fs::read(&path) else {
                    continue;
                };
                if fname.to_ascii_lowercase().ends_with(".zip") {
                    let mut tmp = Vfs::new();
                    if let Ok(k) = add_zip(&mut tmp, &data, rel) {
                        vfs.append(&mut tmp);
                        *n += k;
                        continue;
                    }
                }
                vfs.insert(norm_path(&child), data);
                *n += 1;
            }
        }
        Ok(())
    }
    let mut n = 0;
    let rel = norm_path(prefix.trim_end_matches(['/', '\\']));
    walk(vfs, dir, &rel, &mut n)?;
    Ok(n)
}

// ---------------------------------------------------------------------------
// resolution / discovery
// ---------------------------------------------------------------------------

/// Resolve a file name referenced from a .def in `base_dir`. Tries
/// `base_dir/name`, `name`, `data/name`, then the basename anywhere under
/// `base_dir`, then the basename anywhere.
pub fn resolve<'a>(vfs: &'a Vfs, base_dir: &str, name: &str) -> Option<&'a [u8]> {
    if name.is_empty() {
        return None;
    }
    let n = norm_path(&unquote(name));
    if n.is_empty() {
        return None;
    }
    let base_dir = norm_path(base_dir.trim_end_matches(['/', '\\']));
    if !base_dir.is_empty() {
        let candidate = format!("{base_dir}/{n}");
        if let Some(v) = vfs.get(&candidate) {
            return Some(v);
        }
    }
    if let Some(v) = vfs.get(&n) {
        return Some(v);
    }
    let candidate = format!("data/{n}");
    if let Some(v) = vfs.get(&candidate) {
        return Some(v);
    }
    let base = n.rsplit('/').next().unwrap_or(&n);
    let suffix = format!("/{base}");
    if !base_dir.is_empty() {
        let folder = format!("{base_dir}/");
        if let Some((_, v)) = vfs
            .range::<String, _>(&folder..)
            .take_while(|(k, _)| k.starts_with(&folder))
            .find(|(k, _)| k.ends_with(&suffix))
        {
            return Some(v);
        }
    }
    vfs.iter()
        .find(|(k, _)| k.ends_with(&suffix) || k.as_str() == base)
        .map(|(_, v)| v.as_slice())
}

/// `\b<word>\s*=` in lowercase text.
fn has_key(t: &str, word: &str) -> bool {
    let b = t.as_bytes();
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut from = 0;
    while let Some(p) = t[from..].find(word) {
        let s = from + p;
        if s == 0 || !is_word(b[s - 1]) {
            let mut i = s + word.len();
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if b.get(i) == Some(&b'=') {
                return true;
            }
        }
        from = s + 1;
    }
    false
}

/// Character .def files (has `[Files]` with `cns =` and `sprite =`).
pub fn find_char_defs(vfs: &Vfs) -> Vec<String> {
    vfs.iter()
        .filter(|(k, v)| {
            k.ends_with(".def") && {
                let t = text(v).to_lowercase();
                t.contains("[files]") && has_key(&t, "cns") && has_key(&t, "sprite")
            }
        })
        .map(|(k, _)| k.clone())
        .collect()
}

/// Stage .def files (has `[BGdef` and `[StageInfo]` or `[Camera]`).
pub fn find_stage_defs(vfs: &Vfs) -> Vec<String> {
    vfs.iter()
        .filter(|(k, v)| {
            k.ends_with(".def") && {
                let t = text(v).to_lowercase();
                t.contains("[bgdef") && (t.contains("[stageinfo]") || t.contains("[camera]"))
            }
        })
        .map(|(k, _)| k.clone())
        .collect()
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Stable 64-bit hash (hex) of arbitrary bytes, for deduplicating persisted or repeated imports.
pub fn content_hash(bytes: &[u8]) -> String {
    format!("{:016x}", fnv(FNV_OFFSET, bytes))
}

fn canonical_def_text(def_bytes: &[u8]) -> String {
    text(def_bytes)
        .lines()
        .map(|l| l.split(';').next().unwrap_or("").trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn def_key(vfs: &Vfs, def: &str, section: &str, sprite_key: &str) -> String {
    let def = norm_path(def);
    let Some(def_bytes) = vfs.get(&def) else {
        return format!("path:{def}");
    };
    let secs = crate::ini::parse_ini(&text(def_bytes));
    let sprite = crate::ini::find_section(&secs, section)
        .and_then(|s| s.get(sprite_key))
        .map(unquote)
        .unwrap_or_default();
    let base = def.rfind('/').map(|i| &def[..i]).unwrap_or("");
    let mut h = fnv(FNV_OFFSET, canonical_def_text(def_bytes).as_bytes());
    match resolve(vfs, base, &sprite) {
        Some(b) => {
            h = fnv(h, &(b.len() as u64).to_le_bytes());
            h = fnv(h, b);
        }
        None => h = fnv(h, norm_path(&sprite).as_bytes()),
    }
    format!("{h:016x}")
}

/// Identity of a character independent of where it was found: hash of its normalized `.def` and sprite file.
/// The same fighter imported twice, from another path, or with different letter case yields the same key.
pub fn char_key(vfs: &Vfs, def: &str) -> String {
    def_key(vfs, def, "files", "sprite")
}

/// Identity of a stage, same idea as [`char_key`] (uses `[BGdef] spr`).
pub fn stage_key(vfs: &Vfs, def: &str) -> String {
    def_key(vfs, def, "bgdef", "spr")
}
