//! Memoizes PNG decode and `meta.json` parse per sprite file: many frames resolve to the same pose.
//! Entries are keyed by path and reused only when the raw bytes are identical, so edited files re-decode.
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Decoded sprite: straight RGBA shared by every frame that uses the file unrecolored.
pub(super) struct Decoded {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<Vec<u8>>,
    pub bounds: Option<[usize; 4]>,
}

type Key = (PathBuf, String);
type Entry<T> = (Arc<Vec<u8>>, Arc<T>);

fn cached<T>(store: &'static OnceLock<Mutex<HashMap<Key, Entry<T>>>>, root: &Path, name: &str, bytes: &Arc<Vec<u8>>, build: impl FnOnce(&[u8]) -> Option<T>) -> Option<Arc<T>> {
    let map = store.get_or_init(Default::default);
    let key = (root.to_path_buf(), name.to_string());
    if let Some((seen, value)) = map.lock().unwrap_or_else(|p| p.into_inner()).get(&key) {
        if Arc::ptr_eq(seen, bytes) || **seen == **bytes { return Some(value.clone()); }
    }
    let value = Arc::new(build(bytes)?);
    map.lock().unwrap_or_else(|p| p.into_inner()).insert(key, (bytes.clone(), value.clone()));
    Some(value)
}

fn sprite_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if ume_formats::lz4_sprite::is_lz4_sprite(bytes) {
        return ume_formats::lz4_sprite::decode_rgba(bytes).ok();
    }
    let dec = ume_formats::png::decode_png(bytes).ok()?;
    let rgba = match dec.rgba { Some(ref rgba) => rgba.clone(), None => ume_formats::png::png_indexed_to_rgba(&dec) };
    Some((dec.width, dec.height, rgba))
}

pub(super) fn png(root: &Path, name: &str, bytes: &Arc<Vec<u8>>) -> Option<Arc<Decoded>> {
    static STORE: OnceLock<Mutex<HashMap<Key, Entry<Decoded>>>> = OnceLock::new();
    cached(&STORE, root, name, bytes, |b| {
        let (width, height, rgba) = sprite_rgba(b)?;
        let bounds = super::pixel_crop::content_box(width as usize, height as usize, &rgba);
        Some(Decoded { width, height, rgba: Arc::new(rgba), bounds })
    })
}

pub(super) fn thumbnail(root: &Path, name: &str, bytes: &Arc<Vec<u8>>, shrink: impl FnOnce(u32, u32, &[u8]) -> (u32, u32, Vec<u8>)) -> Option<Arc<Decoded>> {
    static STORE: OnceLock<Mutex<HashMap<Key, Entry<Decoded>>>> = OnceLock::new();
    cached(&STORE, root, name, bytes, |b| {
        let (full_width, full_height, full) = sprite_rgba(b)?;
        let (width, height, rgba) = shrink(full_width, full_height, &full);
        Some(Decoded { width, height, rgba: Arc::new(rgba), bounds: None })
    })
}

pub(super) fn meta(root: &Path, bytes: &Arc<Vec<u8>>) -> Arc<Value> {
    static STORE: OnceLock<Mutex<HashMap<Key, Entry<Value>>>> = OnceLock::new();
    cached(&STORE, root, "meta.json", bytes, |b| serde_json::from_slice(b).ok()).unwrap_or_else(|| Arc::new(Value::Null))
}
