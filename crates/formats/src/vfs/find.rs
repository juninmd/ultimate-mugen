use super::{norm_path, text, Vfs, VfsFile};
use crate::ini::unquote;

pub(crate) const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
pub(crate) const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub(crate) fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
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

/// Resolve a file name referenced from a .def in `base_dir` to its VFS key.
/// Tries `base_dir/name`, `name`, `data/name`, then the basename anywhere under
/// `base_dir`, then the basename anywhere.
pub fn resolve_key(vfs: &Vfs, base_dir: &str, name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let n = norm_path(&unquote(name));
    if n.is_empty() {
        return None;
    }
    let base_dir = norm_path(base_dir.trim_end_matches(['/', '\\']));
    let tries = [
        if base_dir.is_empty() {
            n.clone()
        } else {
            format!("{base_dir}/{n}")
        },
        n.clone(),
        format!("data/{n}"),
    ];
    for t in &tries {
        if vfs.contains_key(t) {
            return Some(t.clone());
        }
    }
    let base = n.rsplit('/').next().unwrap_or(&n);
    let suffix = format!("/{base}");
    if !base_dir.is_empty() {
        let folder = format!("{base_dir}/");
        if let Some(k) = vfs
            .keys_from(&folder)
            .take_while(|k| k.starts_with(&folder))
            .find(|k| k.ends_with(&suffix))
        {
            return Some(k.clone());
        }
    }
    vfs.keys()
        .find(|k| k.ends_with(&suffix) || k.as_str() == base)
        .cloned()
}

/// Same as [`resolve_key`], returning the bytes.
pub fn resolve<'a>(vfs: &'a Vfs, base_dir: &str, name: &str) -> Option<VfsFile<'a>> {
    let key = resolve_key(vfs, base_dir, name)?;
    vfs.get(&key)
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
    let candidates: Vec<&String> = vfs.keys().filter(|k| k.ends_with(".def")).collect();
    let hits = crate::par::map(&candidates, |key| {
        let Some(bytes) = vfs.get(key) else {
            return false;
        };
        if bytes.len() > 1 << 20 {
            return false;
        }
        let t = text(&bytes).to_ascii_lowercase();
        t.contains("[files]") && has_key(&t, "cns") && has_key(&t, "sprite")
    });
    candidates
        .into_iter()
        .zip(hits)
        .filter(|(_, hit)| *hit)
        .map(|(k, _)| k.clone())
        .collect()
}

/// Stage .def files (has `[BGdef` and `[StageInfo]` or `[Camera]`).
pub fn find_stage_defs(vfs: &Vfs) -> Vec<String> {
    let candidates: Vec<&String> = vfs.keys().filter(|k| k.ends_with(".def")).collect();
    let hits = crate::par::map(&candidates, |key| {
        let Some(bytes) = vfs.get(key) else {
            return false;
        };
        if bytes.len() > 1 << 20 {
            return false;
        }
        let t = text(&bytes).to_ascii_lowercase();
        t.contains("[bgdef") && (t.contains("[stageinfo]") || t.contains("[camera]"))
    });
    candidates
        .into_iter()
        .zip(hits)
        .filter(|(_, hit)| *hit)
        .map(|(k, _)| k.clone())
        .collect()
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
    let secs = crate::ini::parse_ini(&text(&def_bytes));
    let sprite = crate::ini::find_section(&secs, section)
        .and_then(|s| s.get(sprite_key))
        .map(|v| unquote(v))
        .unwrap_or_default();
    let base = def.rfind('/').map(|i| &def[..i]).unwrap_or("");
    let mut h = fnv(FNV_OFFSET, canonical_def_text(&def_bytes).as_bytes());
    match resolve_key(vfs, base, &sprite).and_then(|k| vfs.signature(&k)) {
        Some((len, sample)) => {
            h = fnv(h, &len.to_le_bytes());
            h = fnv(h, &sample.to_le_bytes());
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
