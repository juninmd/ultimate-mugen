use std::collections::HashMap;
use std::sync::Mutex;

pub trait Persist: Send + Sync {
    fn read(&self, key: &str) -> Option<Vec<u8>>;
    fn write(&self, key: &str, data: &[u8]) -> Result<(), String>;
    fn delete(&self, key: &str);
}

pub const STATE_KEY: &str = "state.json";

pub fn catalog_key(repo_key: &str) -> String {
    format!("catalog/{repo_key}")
}

pub fn item_key(repo_key: &str, item_id: &str) -> String {
    format!("item/{repo_key}/{item_id}")
}

#[derive(Default)]
pub struct MemPersist {
    map: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemPersist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_entries(entries: HashMap<String, Vec<u8>>) -> Self {
        MemPersist { map: Mutex::new(entries) }
    }

    pub fn keys(&self) -> Vec<String> {
        let mut k: Vec<String> = self.map.lock().map(|m| m.keys().cloned().collect()).unwrap_or_default();
        k.sort();
        k
    }
}

impl Persist for MemPersist {
    fn read(&self, key: &str) -> Option<Vec<u8>> {
        self.map.lock().ok()?.get(key).cloned()
    }

    fn write(&self, key: &str, data: &[u8]) -> Result<(), String> {
        self.map.lock().map_err(|_| "storage poisoned".to_string())?.insert(key.to_string(), data.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) {
        if let Ok(mut m) = self.map.lock() {
            m.remove(key);
        }
    }
}
