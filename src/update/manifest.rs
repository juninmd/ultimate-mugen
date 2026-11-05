use serde_json::Value;

pub const MAX_BINARY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_NOTES_CHARS: usize = 600;

#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    pub version: String,
    pub notes: String,
    pub binary: Option<Asset>,
}

pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let core = s.trim().trim_start_matches('v').split(['-', '+']).next()?;
    let mut it = core.split('.');
    let t = (
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    );
    if it.next().is_some() {
        return None;
    }
    Some(t)
}

pub fn is_newer(current: &str, latest: &str) -> bool {
    matches!((parse_version(current), parse_version(latest)), (Some(c), Some(l)) if l > c)
}

pub fn platform() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some("windows-x86_64"),
        ("linux", "x86_64") => Some("linux-x86_64"),
        ("macos", "aarch64") => Some("macos-aarch64"),
        _ => None,
    }
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn asset_of(value: Option<&Value>) -> Option<Asset> {
    let v = value?;
    let url = v.get("url")?.as_str()?.trim();
    let sha256 = v.get("sha256")?.as_str()?.trim();
    let size = v.get("size")?.as_u64()?;
    let https = url.starts_with("https://") && url.len() > 8;
    (https && is_sha256_hex(sha256) && size > 0 && size <= MAX_BINARY_BYTES).then(|| Asset {
        url: url.to_string(),
        sha256: sha256.to_ascii_lowercase(),
        size,
    })
}

fn parse_notes(value: Option<&Value>) -> String {
    value
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .chars()
        .take(MAX_NOTES_CHARS)
        .collect()
}

pub fn parse_manifest(manifest: &str, current: &str, platform: Option<&str>) -> Option<Update> {
    let v: Value = serde_json::from_str(manifest).ok()?;
    let version = v.get("version")?.as_str()?.trim().trim_start_matches('v');
    if !is_newer(current, version) {
        return None;
    }
    let binary = platform
        .and_then(|p| v.get("platforms")?.get(p))
        .and_then(|entry| asset_of(entry.get("binary")));
    Some(Update {
        version: version.to_string(),
        notes: parse_notes(v.get("notes")),
        binary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(binary: &str) -> String {
        format!(
            r#"{{"version":"0.4.0","notes":"hello","platforms":{{"windows-x86_64":{{{binary}}}}}}}"#
        )
    }

    const GOOD: &str = r#""binary":{"url":"https://github.com/o/r/releases/download/v0.4.0/app.exe","sha256":"0000000000000000000000000000000000000000000000000000000000000001","size":42}"#;

    #[test]
    fn versions_compare_numerically_not_lexically() {
        assert!(is_newer("0.9.0", "0.10.0"));
        assert!(is_newer("0.2.0-dev", "v0.2.1"));
        assert!(!is_newer("0.3.0", "0.3.0"));
        assert!(!is_newer("1.0.0", "0.99.99"));
    }

    #[test]
    fn malformed_versions_never_announce_an_update() {
        for bad in ["", "latest", "1.2", "1.2.3.4", "1.x.3", "-1.0.0"] {
            assert!(!is_newer("0.1.0", bad), "{bad}");
        }
        assert!(!is_newer("garbage", "9.9.9"));
    }

    #[test]
    fn manifest_announces_only_newer_releases() {
        assert_eq!(
            parse_manifest(&manifest(GOOD), "0.3.0", Some("windows-x86_64")).map(|u| u.version),
            Some("0.4.0".into())
        );
        assert_eq!(
            parse_manifest(&manifest(GOOD), "0.4.0", Some("windows-x86_64")),
            None
        );
        assert_eq!(parse_manifest("not json", "0.3.0", None), None);
        assert_eq!(parse_manifest(r#"{"version":7}"#, "0.3.0", None), None);
    }

    #[test]
    fn binary_is_used_only_for_the_current_platform() {
        let update = parse_manifest(&manifest(GOOD), "0.3.0", Some("windows-x86_64")).unwrap();
        assert_eq!(update.binary.unwrap().size, 42);
        let other = parse_manifest(&manifest(GOOD), "0.3.0", Some("linux-x86_64")).unwrap();
        assert_eq!(other.binary, None);
        assert_eq!(
            parse_manifest(&manifest(GOOD), "0.3.0", None)
                .unwrap()
                .binary,
            None
        );
    }

    #[test]
    fn unsafe_or_incomplete_binaries_are_ignored() {
        let bad = [
            r#""binary":{"url":"http://github.com/o/r/app.exe","sha256":"0000000000000000000000000000000000000000000000000000000000000001","size":42}"#,
            r#""binary":{"url":"https://github.com/o/r/app.exe","sha256":"xyz","size":42}"#,
            r#""binary":{"url":"https://github.com/o/r/app.exe","sha256":"0000000000000000000000000000000000000000000000000000000000000001","size":0}"#,
            r#""binary":{"url":"https://github.com/o/r/app.exe","sha256":"0000000000000000000000000000000000000000000000000000000000000001","size":999999999999}"#,
            r#""binary":{"url":"https://github.com/o/r/app.exe"}"#,
        ];
        for entry in bad {
            let update = parse_manifest(&manifest(entry), "0.3.0", Some("windows-x86_64")).unwrap();
            assert_eq!(update.binary, None, "{entry}");
        }
    }

    #[test]
    fn notes_are_bounded_and_missing_binaries_stay_notice_only() {
        let long = "x".repeat(MAX_NOTES_CHARS + 50);
        let m = format!(r#"{{"version":"0.4.0","notes":"{long}","platforms":{{}}}}"#);
        assert_eq!(
            parse_manifest(&m, "0.3.0", None)
                .unwrap()
                .notes
                .chars()
                .count(),
            MAX_NOTES_CHARS
        );
    }
}
