use crate::ini::unquote;
use std::collections::BTreeMap;

mod zip;
pub use zip::{add_zip, add_zip_with_progress, ZipProgress};

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

fn with_joined<R>(a: &str, b: &str, f: impl FnOnce(&str) -> R) -> R {
    let mut buf = [0u8; 512];
    let len = a.len() + 1 + b.len();
    if len <= buf.len() {
        let a_b = a.as_bytes();
        let b_b = b.as_bytes();
        buf[..a_b.len()].copy_from_slice(a_b);
        buf[a_b.len()] = b'/';
        buf[a_b.len() + 1..len].copy_from_slice(b_b);
        if let Ok(s) = std::str::from_utf8(&buf[..len]) {
            return f(s);
        }
    }
    f(&format!("{a}/{b}"))
}

/// Resolve a file name referenced from a .def in `base_dir`. Tries
/// `base_dir/name`, `name`, `data/name`, then the basename anywhere under
/// `base_dir`, then the basename anywhere.
pub fn with_norm_path<R>(p: &str, f: impl FnOnce(&str) -> R) -> R {
    let unquoted = crate::ini::unquote_str(p);
    let p = unquoted
        .strip_prefix("./")
        .or_else(|| unquoted.strip_prefix(".\\"))
        .unwrap_or(unquoted);
    if p.is_ascii() && p.len() <= 512 {
        let mut buf = [0u8; 512];
        let mut len = 0;
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
            buf[len] = b;
            len += 1;
        }
        if let Ok(s) = std::str::from_utf8(&buf[..len]) {
            return f(s);
        }
    }
    f(&norm_path(p))
}

pub fn resolve<'a>(vfs: &'a Vfs, base_dir: &str, name: &str) -> Option<&'a [u8]> {
    if name.is_empty() {
        return None;
    }
    with_norm_path(name, |n| {
        if n.is_empty() {
            return None;
        }
        let trimmed_base = base_dir.trim_end_matches(['/', '\\']);
        with_norm_path(trimmed_base, |base_dir| {
            if !base_dir.is_empty() {
                if let Some(v) = with_joined(base_dir, n, |cand| vfs.get(cand)) {
                    return Some(v.as_slice());
                }
            }
            if let Some(v) = vfs.get(n) {
                return Some(v.as_slice());
            }
            if let Some(v) = with_joined("data", n, |cand| vfs.get(cand)) {
                return Some(v.as_slice());
            }
            let base = n.rsplit('/').next().unwrap_or(n);
            let is_match = |k: &str| {
                k.ends_with(base)
                    && (k.len() == base.len()
                        || k.as_bytes().get(k.len() - base.len() - 1) == Some(&b'/'))
            };
            if !base_dir.is_empty() {
                let folder = format!("{base_dir}/");
                if let Some((_, v)) = vfs
                    .range::<String, _>(&folder..)
                    .take_while(|(k, _)| k.starts_with(&folder))
                    .find(|(k, _)| is_match(k.as_str()))
                {
                    return Some(v.as_slice());
                }
            }
            vfs.iter()
                .find(|(k, _)| is_match(k.as_str()))
                .map(|(_, v)| v.as_slice())
        })
    })
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
