//! Browser import queue: JS writes `.zip` bytes into wasm memory, the app drains one pack per frame.
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

pub static QUEUE: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());
static BATCHES: AtomicUsize = AtomicUsize::new(0);
static DISCARD: AtomicBool = AtomicBool::new(false);

/// The JS loader is still reading/digesting files, even before it queues the first pack.
pub fn busy() -> bool {
    BATCHES.load(Ordering::Relaxed) > 0 && !DISCARD.load(Ordering::Relaxed)
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_import_begin() {
    if BATCHES.fetch_add(1, Ordering::Relaxed) == 0 {
        DISCARD.store(false, Ordering::Relaxed);
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_import_end() {
    let left = BATCHES.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1));
    if left == Ok(1) {
        DISCARD.store(false, Ordering::Relaxed);
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ume_import(bytes: Vec<u8>) {
    if DISCARD.load(Ordering::Relaxed) {
        return;
    }
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

/// User cancelled: drop what is queued and ignore the rest of the current JS batches.
pub fn cancel() {
    DISCARD.store(true, Ordering::Relaxed);
    if let Ok(mut q) = QUEUE.lock() {
        q.clear();
    }
}

/// The JS loader stopped answering (blocked storage, lost promise): stop waiting for it.
pub fn abandon() {
    BATCHES.store(0, Ordering::Relaxed);
    DISCARD.store(false, Ordering::Relaxed);
}
