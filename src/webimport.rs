//! Browser import queue: JS writes `.zip` bytes into wasm memory, the app drains one pack per frame.
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub static QUEUE: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());
static BATCHES: AtomicUsize = AtomicUsize::new(0);

/// The JS loader is still reading/digesting files, even before it queues the first pack.
pub fn busy() -> bool {
    BATCHES.load(Ordering::Relaxed) > 0
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_import_begin() {
    BATCHES.fetch_add(1, Ordering::Relaxed);
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_import_end() {
    let _ = BATCHES.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1));
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_import(bytes: Vec<u8>) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push_back(bytes);
    }
}

/// Backpressure for the JS loader: keep at most a couple of raw packs in wasm memory.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_import_pending() -> usize {
    queued_imports()
}

/// Next pack for the incremental web importer (one per frame, never the whole batch).
pub fn take_import() -> Option<Vec<u8>> {
    QUEUE.lock().ok().and_then(|mut q| q.pop_front())
}

pub fn queued_imports() -> usize {
    QUEUE.lock().map(|q| q.len()).unwrap_or(0)
}
