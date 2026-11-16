use std::sync::Mutex;

static LOG: Mutex<Vec<(String, Vec<String>)>> = Mutex::new(Vec::new());

pub fn record(name: &str, items: Vec<String>) {
    if items.is_empty() {
        return;
    }
    if let Ok(mut log) = LOG.lock() {
        log.push((name.to_string(), items));
    }
}

pub fn snapshot() -> Vec<(String, Vec<String>)> {
    LOG.lock().map(|log| log.clone()).unwrap_or_default()
}
