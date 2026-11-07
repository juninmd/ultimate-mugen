use super::types::NativeChar;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

static NATIVE_CHARS: OnceLock<RwLock<HashMap<String, Arc<NativeChar>>>> = OnceLock::new();

fn store() -> &'static RwLock<HashMap<String, Arc<NativeChar>>> {
    NATIVE_CHARS.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_native(nc: Arc<NativeChar>) {
    if let Ok(mut map) = store().write() {
        map.insert(nc.id.clone(), nc);
    }
}

pub fn find_native(id: &str) -> Option<Arc<NativeChar>> {
    store().read().ok().and_then(|map| map.get(id).cloned())
}
