pub const OVERRIDE_PATH: &str = "story/campaign.toml";
pub const STORE_KEY: &str = "story-campaign.toml";

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn from_env() -> Option<String> {
    let path = std::env::var("UME_STORY_SCRIPT").ok().filter(|p| !p.trim().is_empty())?;
    std::fs::read_to_string(path).ok()
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn from_files() -> Option<String> {
    std::fs::read_to_string(OVERRIDE_PATH).ok().or_else(|| {
        let exe = std::env::current_exe().ok()?;
        std::fs::read_to_string(exe.parent()?.join(OVERRIDE_PATH)).ok()
    })
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn from_env() -> Option<String> {
    None
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn from_files() -> Option<String> {
    None
}

pub fn read_external_toml() -> Option<String> {
    from_env()
        .or_else(|| crate::storage::read(STORE_KEY).filter(|text| !text.trim().is_empty()))
        .or_else(from_files)
}
