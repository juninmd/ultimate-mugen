use std::path::{Path, PathBuf};
use ume_netplay::Replay;

pub use ume_netplay::store::{default_name, format_duration, format_time, now, Entry, Sort};
#[cfg(not(target_arch = "wasm32"))]
pub use ume_netplay::store::{delete, list_in, load, rename, save_in};

pub fn sort_label(sort: Sort) -> &'static str {
    match sort {
        Sort::Newest => "mais recentes",
        Sort::Oldest => "mais antigos",
        Sort::Name => "nome",
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn dir() -> PathBuf {
    if let Ok(custom) = std::env::var("UME_REPLAY_DIR") {
        return custom.into();
    }
    #[cfg(target_os = "android")]
    if let Some(d) = ume_shell::ANDROID_APP.get().and_then(|a| a.internal_data_path()) {
        return d.join("replays");
    }
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("replays"))).unwrap_or_else(|| "replays".into())
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;

    const UNAVAILABLE: &str = "replays em disco não estão disponíveis na versão web";

    pub fn dir() -> PathBuf {
        PathBuf::new()
    }

    pub fn list_in(_: &Path, _: Sort) -> Vec<Entry> {
        vec![]
    }

    pub fn save_in(_: &Path, _: &str, _: &Replay) -> Result<PathBuf, String> {
        Err(UNAVAILABLE.into())
    }

    pub fn load(_: &Path) -> Result<Replay, String> {
        Err(UNAVAILABLE.into())
    }

    pub fn rename(_: &Path, _: &str) -> Result<PathBuf, String> {
        Err(UNAVAILABLE.into())
    }

    pub fn delete(_: &Path) -> Result<(), String> {
        Err(UNAVAILABLE.into())
    }
}

#[cfg(target_arch = "wasm32")]
pub use web::{delete, dir, list_in, load, rename, save_in};
