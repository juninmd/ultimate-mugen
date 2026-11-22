use super::pack::SpritePack;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

include!(concat!(env!("OUT_DIR"), "/embedded_sprites.rs"));

fn pack(character: &str) -> Option<Arc<SpritePack>> {
    static PARSED: OnceLock<Mutex<HashMap<String, Option<Arc<SpritePack>>>>> = OnceLock::new();
    let mut parsed = PARSED.get_or_init(Default::default).lock().unwrap_or_else(|p| p.into_inner());
    parsed
        .entry(character.to_string())
        .or_insert_with(|| EMBEDDED.iter().find(|(id, _)| *id == character).and_then(|(_, bytes)| SpritePack::parse(bytes).ok()).map(Arc::new))
        .clone()
}

pub(super) fn read(character: &str, name: &str) -> Option<Arc<Vec<u8>>> {
    if EMBEDDED.is_empty() { return None; }
    pack(character)?.file(name)
}

pub(super) fn has(character: &str) -> bool {
    EMBEDDED.iter().any(|(id, _)| *id == character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_packs_serve_meta_when_present() {
        if EMBEDDED.is_empty() { return; }
        assert!(has("kaito") && read("kaito", "meta.json").is_some());
        assert!(read("kaito", "no-such-file.png").is_none());
    }
}
