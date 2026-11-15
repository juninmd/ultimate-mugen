use std::sync::Arc;
use ume_engine::chardef::ImageData;

#[cfg(not(target_arch = "wasm32"))]
pub fn want(_id: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn still(id: &str) -> Option<Arc<ImageData>> {
    super::illustration(id)
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static STILLS: std::cell::RefCell<std::collections::HashMap<String, Option<Arc<ImageData>>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(target_arch = "wasm32")]
pub fn still(id: &str) -> Option<Arc<ImageData>> {
    STILLS
        .with(|stills| stills.borrow().get(id).cloned().flatten())
        .or_else(|| super::illustration(id))
}

#[cfg(target_arch = "wasm32")]
pub fn want(id: &str) {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
    let known = STILLS.with(|stills| stills.borrow().contains_key(id));
    if known {
        return;
    }
    STILLS.with(|stills| stills.borrow_mut().insert(id.to_string(), None));
    let id = id.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let path = format!("assets/stages/{id}.png");
        let Ok(data) = js_sys::futures::JsFuture::from(ume_fetch_asset(&path)).await else {
            return;
        };
        let Ok(image) = super::decode_illustration(&js_sys::Uint8Array::new(&data).to_vec()) else {
            return;
        };
        STILLS.with(|stills| stills.borrow_mut().insert(id, Some(image)));
    });
}
