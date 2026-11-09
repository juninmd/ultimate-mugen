use crate::ini::{find_section, parse_ini_no_lines, unquote};
use crate::vfs::{resolve_key, text, Vfs};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingRef {
    pub source: String,
    pub key: String,
    pub value: String,
}

const FILE_KEYS: &[&str] = &["sprite", "anim", "sound", "cmd", "cns", "st", "stcommon"];

fn is_file_key(key: &str) -> bool {
    if FILE_KEYS.contains(&key) {
        return true;
    }
    key.strip_prefix("st")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

pub fn def_references(def_text: &str) -> Vec<(String, String)> {
    let sections = parse_ini_no_lines(def_text);
    let Some(files) = find_section(&sections, "files") else {
        return Vec::new();
    };
    files
        .entries
        .iter()
        .filter(|(key, value)| is_file_key(key) && !unquote(value).is_empty())
        .map(|(key, value)| (key.clone(), unquote(value)))
        .collect()
}

pub fn missing_in_char(vfs: &Vfs, def_key: &str) -> Vec<MissingRef> {
    let Some(file) = vfs.get(def_key) else {
        return Vec::new();
    };
    let base = def_key.rsplit_once('/').map_or("", |(dir, _)| dir);
    def_references(&text(file.as_slice()))
        .into_iter()
        .filter(|(_, value)| resolve_key(vfs, base, value).is_none())
        .map(|(key, value)| MissingRef {
            source: def_key.to_string(),
            key,
            value,
        })
        .collect()
}
