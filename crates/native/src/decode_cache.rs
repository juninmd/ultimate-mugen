//! Memoizes PNG decode and `meta.json` parse per sprite file: many frames resolve to the same pose.
//! Entries are keyed by path and reused only when the raw bytes are identical, so edited files re-decode.
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use ume_formats::png::DecodedPng;

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

pub(super) fn png(root: &Path, name: &str, bytes: &Arc<Vec<u8>>) -> Option<Arc<DecodedPng>> {
    static STORE: OnceLock<Mutex<HashMap<Key, Entry<DecodedPng>>>> = OnceLock::new();
    cached(&STORE, root, name, bytes, |b| ume_formats::png::decode_png(b).ok())
}

pub(super) fn meta(root: &Path, bytes: &Arc<Vec<u8>>) -> Arc<Value> {
    static STORE: OnceLock<Mutex<HashMap<Key, Entry<Value>>>> = OnceLock::new();
    cached(&STORE, root, "meta.json", bytes, |b| serde_json::from_slice(b).ok()).unwrap_or_else(|| Arc::new(Value::Null))
}
