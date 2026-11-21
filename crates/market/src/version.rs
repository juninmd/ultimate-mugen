use crate::model::FileRef;

pub fn item_version(files: &[FileRef]) -> String {
    if files.is_empty() || files.iter().any(|f| f.sha.is_empty()) {
        return String::new();
    }
    let mut sorted: Vec<&FileRef> = files.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut hash: u64 = 0xcbf29ce484222325;
    for f in sorted {
        for byte in f
            .path
            .bytes()
            .chain([0])
            .chain(f.sha.bytes())
            .chain([b'\n'])
        {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    format!("{hash:016x}")
}

pub fn cache_bust(sha: &str) -> String {
    if sha.is_empty() || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        String::new()
    } else {
        format!("?v={sha}")
    }
}

pub fn has_unversioned(items: &[crate::model::Item]) -> bool {
    items.iter().any(|i| i.version.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, sha: &str) -> FileRef {
        FileRef {
            path: path.into(),
            size: 1,
            sha: sha.into(),
        }
    }

    #[test]
    fn version_changes_when_any_file_content_changes() {
        let a = item_version(&[file("a.def", "aa"), file("a.sff", "bb")]);
        assert_eq!(a.len(), 16);
        assert_eq!(a, item_version(&[file("a.sff", "bb"), file("a.def", "aa")]));
        assert_ne!(a, item_version(&[file("a.def", "aa"), file("a.sff", "cc")]));
        assert_ne!(a, item_version(&[file("a.def", "aa")]));
        assert_ne!(
            item_version(&[file("ab", "c")]),
            item_version(&[file("a", "bc")])
        );
    }

    #[test]
    fn unknown_shas_give_no_version() {
        assert_eq!(item_version(&[]), "");
        assert_eq!(item_version(&[file("a", "")]), "");
    }

    #[test]
    fn cache_bust_only_accepts_hex_shas() {
        assert_eq!(cache_bust("0a1F"), "?v=0a1F");
        assert_eq!(cache_bust(""), "");
        assert_eq!(cache_bust("x&y=1"), "");
    }
}
