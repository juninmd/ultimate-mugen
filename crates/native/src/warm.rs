use super::lookup::register_native;
use super::types::NativeChar;
use std::sync::Arc;

pub fn register_native_warm(nc: Arc<NativeChar>) {
    register_native(nc.clone());
    #[cfg(not(target_arch = "wasm32"))]
    queue(nc);
}

#[cfg(not(target_arch = "wasm32"))]
fn queue(nc: Arc<NativeChar>) {
    use std::sync::{mpsc, Mutex, OnceLock};
    static TX: OnceLock<Option<mpsc::Sender<Arc<NativeChar>>>> = OnceLock::new();
    let tx = TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Arc<NativeChar>>();
        let rx = Arc::new(Mutex::new(rx));
        for _ in 0..ume_formats::par::workers(4) {
            let rx = rx.clone();
            let spawned = std::thread::Builder::new().name("portrait-warm".into()).spawn(move || loop {
                let next = rx.lock().unwrap_or_else(|p| p.into_inner()).recv();
                let Ok(nc) = next else { break };
                super::portrait::select_portrait(&nc, 1);
            });
            if spawned.is_err() { return None; }
        }
        Some(tx)
    });
    if let Some(tx) = tx { let _ = tx.send(nc); }
}
