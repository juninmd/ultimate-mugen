use std::sync::{Arc, OnceLock};
use ume_engine::chardef::CharDef;

static SHARED: OnceLock<Arc<CharDef>> = OnceLock::new();

pub fn shared() -> Arc<CharDef> {
    SHARED.get_or_init(|| ume_engine::fx::build_fx(true)).clone()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn prewarm() {
    static STARTED: std::sync::Once = std::sync::Once::new();
    STARTED.call_once(|| {
        let _ = std::thread::Builder::new().name("fx-warm".into()).spawn(|| {
            shared();
        });
    });
}
