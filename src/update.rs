//! Release update check: reads the public `latest.json` published with each GitHub Release (notify-only, never installs).
//! Opt out with `--no-update-check` or `UME_NO_UPDATE_CHECK=1`. Dev builds (`-dev` version) never check.
// the network check is native-only; its helpers are intentionally unused on web/mobile targets
#![cfg_attr(any(target_arch = "wasm32", target_os = "android", target_os = "ios"), allow(dead_code))]
use std::sync::mpsc::Receiver;

/// Release version stamped by CI (`UME_RELEASE_VERSION`); `<cargo version>-dev` for local builds.
pub const VERSION: &str = env!("UME_VERSION");
pub const MANIFEST_URL: &str = "https://github.com/juninmd/ultimate-mugen/releases/latest/download/latest.json";
pub const RELEASES_URL: &str = "https://github.com/juninmd/ultimate-mugen/releases/latest";

#[derive(Clone, Debug, PartialEq)]
pub struct Update { pub version: String }

/// `X.Y.Z` (optional leading `v`, optional `-suffix` ignored) -> numeric triple.
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let core = s.trim().trim_start_matches('v').split(['-', '+']).next()?;
    let mut it = core.split('.');
    let t = (it.next()?.parse().ok()?, it.next()?.parse().ok()?, it.next()?.parse().ok()?);
    if it.next().is_some() { return None; }
    Some(t)
}

pub fn is_newer(current: &str, latest: &str) -> bool {
    matches!((parse_version(current), parse_version(latest)), (Some(c), Some(l)) if l > c)
}

/// Returns the update when `manifest` (the `latest.json` body) announces a newer release than `current`.
pub fn parse_manifest(manifest: &str, current: &str) -> Option<Update> {
    let v: serde_json::Value = serde_json::from_str(manifest).ok()?;
    let version = v.get("version")?.as_str()?.trim_start_matches('v');
    is_newer(current, version).then(|| Update { version: version.to_string() })
}

pub fn disabled_by(args: &[String], env_flag: Option<&str>) -> bool {
    args.iter().any(|a| a == "--no-update-check") || env_flag.is_some_and(|v| !v.is_empty() && v != "0")
}

/// Fetches the manifest over HTTPS only (blocking; call from a worker thread).
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn fetch_latest(current: &str) -> Result<Option<Update>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(std::time::Duration::from_secs(8)))
        .build()
        .into();
    let body = agent.get(MANIFEST_URL).call().map_err(|e| e.to_string())?.body_mut().read_to_string().map_err(|e| e.to_string())?;
    Ok(parse_manifest(&body, current))
}

/// Background check started at boot; `poll()` is cheap and non-blocking.
pub struct Checker { rx: Option<Receiver<Option<Update>>>, found: Option<Update> }

impl Checker {
    pub fn start(args: &[String]) -> Self {
        let off = disabled_by(args, std::env::var("UME_NO_UPDATE_CHECK").ok().as_deref()) || VERSION.ends_with("-dev");
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if !off {
            let (tx, rx) = std::sync::mpsc::channel();
            // a failed check (offline, rate-limited) is deliberately silent: the game must never depend on the network
            std::thread::spawn(move || { let _ = tx.send(fetch_latest(VERSION).ok().flatten()); });
            return Checker { rx: Some(rx), found: None };
        }
        let _ = off;
        Checker { rx: None, found: None }
    }

    pub fn poll(&mut self) -> Option<&Update> {
        if let Some(rx) = &self.rx {
            if let Ok(res) = rx.try_recv() { self.found = res; self.rx = None; }
        }
        self.found.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically_not_lexically() {
        assert!(is_newer("0.9.0", "0.10.0"));
        assert!(is_newer("0.2.0-dev", "v0.2.1"));
        assert!(!is_newer("0.3.0", "0.3.0"));
        assert!(!is_newer("1.0.0", "0.99.99"));
    }

    #[test]
    fn malformed_versions_never_announce_an_update() {
        for bad in ["", "latest", "1.2", "1.2.3.4", "1.x.3", "-1.0.0"] { assert!(!is_newer("0.1.0", bad), "{bad}"); }
        assert!(!is_newer("garbage", "9.9.9"));
    }

    #[test]
    fn manifest_announces_only_newer_releases() {
        let m = r#"{"version":"0.4.0","notes":"x","platforms":{}}"#;
        assert_eq!(parse_manifest(m, "0.3.0"), Some(Update { version: "0.4.0".into() }));
        assert_eq!(parse_manifest(m, "0.4.0"), None);
        assert_eq!(parse_manifest("not json", "0.3.0"), None);
        assert_eq!(parse_manifest(r#"{"version":7}"#, "0.3.0"), None);
    }

    #[test]
    fn opt_out_via_flag_or_env() {
        let flag = vec!["--no-update-check".to_string()];
        assert!(disabled_by(&flag, None));
        assert!(disabled_by(&[], Some("1")));
        assert!(!disabled_by(&[], Some("0")));
        assert!(!disabled_by(&[], Some("")));
        assert!(!disabled_by(&[], None));
    }
}
