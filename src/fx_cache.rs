use std::sync::{Arc, OnceLock};
use ume_engine::chardef::CharDef;

static SHARED: OnceLock<Arc<CharDef>> = OnceLock::new();

pub fn shared() -> Arc<CharDef> {
    SHARED.get_or_init(|| ume_engine::fx::build_fx(true)).clone()
}
