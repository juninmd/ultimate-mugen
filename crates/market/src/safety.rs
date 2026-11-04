pub const MAX_FILE_BYTES: u64 = 64 << 20;
pub const MAX_ITEM_BYTES: u64 = 256 << 20;
pub const MAX_ITEM_FILES: usize = 4000;
pub const MAX_TREE_BODY_BYTES: u64 = 48 << 20;
pub const MAX_TREE_ENTRIES: usize = 200_000;
pub const MAX_PATH_BYTES: usize = 240;
pub const MAX_SEGMENT_BYTES: usize = 120;

const ALLOWED_HOSTS: [&str; 5] = ["api.github.com", "raw.githubusercontent.com", "media.githubusercontent.com", "github.com", "codeload.github.com"];

pub fn host_of(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains(['@', ':', '\\', ' ']) {
        return None;
    }
    Some(authority)
}

pub fn url_allowed(url: &str) -> bool {
    host_of(url).is_some_and(|h| ALLOWED_HOSTS.iter().any(|a| a.eq_ignore_ascii_case(h)))
}

pub fn safe_rel_path(p: &str) -> Option<String> {
    if p.is_empty() || p.len() > MAX_PATH_BYTES || p.starts_with('/') || p.ends_with('/') {
        return None;
    }
    if p.chars().any(|c| c.is_control() || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')) {
        return None;
    }
    for seg in p.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." || seg.len() > MAX_SEGMENT_BYTES || seg.ends_with('.') || seg.ends_with(' ') {
            return None;
        }
    }
    Some(p.to_string())
}

pub fn safe_subdir(p: &str) -> Option<String> {
    let t = p.trim_matches('/');
    if t.is_empty() { Some(String::new()) } else { safe_rel_path(t) }
}

pub fn valid_owner(s: &str) -> bool {
    !s.is_empty() && s.len() <= 39 && !s.starts_with('-') && !s.ends_with('-') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub fn valid_repo_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 100 && s != "." && s != ".." && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

pub fn valid_ref(s: &str) -> bool {
    !s.is_empty() && s.len() <= 100 && !s.starts_with(['/', '-', '.']) && !s.ends_with(['/', '.']) && !s.contains("..") && !s.contains("//") && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
}

pub fn percent_encode_path(p: &str) -> String {
    let mut out = String::with_capacity(p.len() + 8);
    for b in p.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~' | b'/') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn lower_ext(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rfind('.') {
        Some(i) if i > 0 => name[i + 1..].to_ascii_lowercase(),
        _ => String::new(),
    }
}

pub fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub fn file_stem(path: &str) -> &str {
    let n = file_name(path);
    match n.rfind('.') {
        Some(i) if i > 0 => &n[..i],
        _ => n,
    }
}

pub fn parent_dir(path: &str) -> &str {
    path.rfind('/').map(|i| &path[..i]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_and_absolute_paths() {
        for bad in ["", "/etc/passwd", "../x", "a/../b", "a/./b", "a//b", "a\\b", "C:/x", "a/b/", "x\0y", "a/b.", "con:", "a/b?c"] {
            assert!(safe_rel_path(bad).is_none(), "{bad:?}");
        }
        assert_eq!(safe_rel_path("chars/KFM/kfm.def").as_deref(), Some("chars/KFM/kfm.def"));
        assert_eq!(safe_rel_path("sprites/ç é.sff").as_deref(), Some("sprites/ç é.sff"));
    }

    #[test]
    fn only_github_hosts_over_https_are_allowed() {
        assert!(url_allowed("https://api.github.com/repos/a/b/git/trees/HEAD"));
        assert!(url_allowed("https://raw.githubusercontent.com/a/b/HEAD/x.def"));
        assert!(!url_allowed("http://api.github.com/x"));
        assert!(!url_allowed("https://evil.com/x"));
        assert!(!url_allowed("https://api.github.com.evil.com/x"));
        assert!(!url_allowed("https://api.github.com@evil.com/x"));
        assert!(!url_allowed("https://evil.com\\@api.github.com/x"));
        assert!(!url_allowed("https://api.github.com:8443/x"));
        assert!(!url_allowed("ftp://github.com/x"));
        assert!(!url_allowed("https:///x"));
    }

    #[test]
    fn percent_encoding_keeps_slashes_and_escapes_the_rest() {
        assert_eq!(percent_encode_path("a b/ç#?.def"), "a%20b/%C3%A7%23%3F.def");
    }

    #[test]
    fn ref_and_owner_validation() {
        assert!(valid_owner("juninmd") && !valid_owner("-x") && !valid_owner("a b") && !valid_owner(""));
        assert!(valid_repo_name("ultimate-mugen") && valid_repo_name("a.b_c") && !valid_repo_name("..") && !valid_repo_name("a/b"));
        assert!(valid_ref("main") && valid_ref("feature/x") && !valid_ref("../x") && !valid_ref("a..b") && !valid_ref("-x"));
    }

    #[test]
    fn path_helpers() {
        assert_eq!(lower_ext("a/B.SFF"), "sff");
        assert_eq!(lower_ext("a/.hidden"), "");
        assert_eq!(file_stem("a/kfm.def"), "kfm");
        assert_eq!(parent_dir("a/b/c.x"), "a/b");
        assert_eq!(parent_dir("c.x"), "");
    }
}
