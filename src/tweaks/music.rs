use super::model::Labels;
use toml::Value;

const PREFIX: &str = "music.";
const DEFAULT_KEY: &str = "fight";
const EXTENSIONS: [&str; 4] = ["ogg", "mp3", "wav", "flac"];

pub fn collect(value: &Value, labels: &mut Labels, warnings: &mut Vec<String>) {
    let Value::Table(table) = value else {
        warnings.push("`music` must be a [music] section".to_string());
        return;
    };
    for (key, entry) in table {
        let path = entry
            .as_str()
            .map(str::trim)
            .filter(|path| !path.is_empty());
        let known = path.is_some_and(|path| {
            EXTENSIONS.iter().any(|extension| {
                path.to_ascii_lowercase()
                    .ends_with(&format!(".{extension}"))
            })
        });
        match path {
            Some(path) if known => labels.push((
                format!("{PREFIX}{}", key.to_ascii_lowercase()),
                path.to_string(),
            )),
            _ => warnings.push(format!(
                "[music] `{key}` must be a path to a .ogg, .mp3, .wav or .flac file"
            )),
        }
    }
}

pub fn path_for(stage_id: &str, lookup: impl Fn(&str) -> Option<String>) -> Option<String> {
    let id = stage_id
        .rsplit([':', '/'])
        .next()
        .unwrap_or(stage_id)
        .to_ascii_lowercase();
    lookup(&format!("{PREFIX}{id}")).or_else(|| lookup(&format!("{PREFIX}{DEFAULT_KEY}")))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn track_for(stage_id: &str) -> Option<std::sync::Arc<(Vec<u8>, crate::audio::MusicLoop)>> {
    let path = path_for(stage_id, super::label)?;
    match std::fs::read(crate::storage::beside_config(&path)) {
        Ok(bytes) => Some(std::sync::Arc::new((
            bytes,
            crate::audio::MusicLoop::default(),
        ))),
        Err(error) => {
            eprintln!("tweaks: [music] {path}: {error}");
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn track_for(_stage_id: &str) -> Option<std::sync::Arc<(Vec<u8>, crate::audio::MusicLoop)>> {
    None
}
