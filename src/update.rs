#![cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod command;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod download;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod install;
pub mod manifest;
mod updater;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use command::{fetch_latest, update_now};
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use install::{install, relaunch};
pub use manifest::Update;
pub use updater::{State, Updater};

pub const VERSION: &str = env!("UME_VERSION");
pub const GIT_HASH: &str = env!("UME_GIT_HASH");
pub const MANIFEST_URL: &str =
    "https://github.com/juninmd/ultimate-mugen/releases/latest/download/latest.json";
pub const RELEASES_URL: &str = "https://github.com/juninmd/ultimate-mugen/releases/latest";

pub fn build_label() -> String {
    if GIT_HASH.is_empty() {
        format!("v{VERSION}")
    } else {
        format!("v{VERSION} · {GIT_HASH}")
    }
}

pub fn disabled_by(args: &[String], env_flag: Option<&str>) -> bool {
    args.iter().any(|a| a == "--no-update-check")
        || env_flag.is_some_and(|v| !v.is_empty() && v != "0")
}

/// `--no-auto-update` / `UME_NO_AUTO_UPDATE`: nunca pergunta (o aviso da versão nova continua no
/// rodapé e o `--update` manual continua valendo).
pub fn auto_disabled(args: &[String], env_flag: Option<&str>) -> bool {
    args.iter().any(|a| a == "--no-auto-update")
        || env_flag.is_some_and(|v| !v.is_empty() && v != "0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_out_via_flag_or_env() {
        let flag = vec!["--no-update-check".to_string()];
        assert!(disabled_by(&flag, None));
        assert!(disabled_by(&[], Some("1")));
        assert!(!disabled_by(&[], Some("0")));
        assert!(!disabled_by(&[], Some("")));
        assert!(!disabled_by(&[], None));
    }

    #[test]
    fn auto_update_can_be_turned_off_without_losing_the_notice() {
        let flag = vec!["--no-auto-update".to_string()];
        assert!(auto_disabled(&flag, None));
        assert!(auto_disabled(&[], Some("1")));
        assert!(!auto_disabled(&[], Some("0")));
        assert!(!auto_disabled(&[], None));
        assert!(!disabled_by(&flag, None));
    }
}
