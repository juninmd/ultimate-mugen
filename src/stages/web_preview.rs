use super::decode_illustration;
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use ume_engine::chardef::ImageData;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn ume_fetch_asset(path: &str) -> js_sys::Promise;
}

enum Slot {
    Pending,
    Ready(Arc<ImageData>),
}

thread_local! {
    static SLOTS: RefCell<HashMap<String, Slot>> = RefCell::new(HashMap::new());
}

async fn load(id: String) {
    let path = format!("assets/stages/thumbs/{id}.png");
    let Ok(data) = js_sys::futures::JsFuture::from(ume_fetch_asset(&path)).await else {
        return;
    };
    if let Ok(image) = decode_illustration(&js_sys::Uint8Array::new(&data).to_vec()) {
        SLOTS.with(|slots| slots.borrow_mut().insert(id, Slot::Ready(image)));
    }
}

pub(super) fn get(id: &str) -> Option<Arc<ImageData>> {
    let state = SLOTS.with(|slots| {
        slots.borrow().get(id).map(|slot| match slot {
            Slot::Ready(image) => Some(image.clone()),
            Slot::Pending => None,
        })
    });
    if let Some(image) = state {
        return image;
    }
    SLOTS.with(|slots| slots.borrow_mut().insert(id.to_string(), Slot::Pending));
    wasm_bindgen_futures::spawn_local(load(id.to_string()));
    None
}
