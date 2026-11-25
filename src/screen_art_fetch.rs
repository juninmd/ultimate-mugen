use crate::gfx::Texture2D;
use std::{cell::RefCell, collections::HashMap};

enum Slot {
    Pending,
    Ready(Texture2D),
}

thread_local! {
    static SLOTS: RefCell<HashMap<&'static str, Slot>> = RefCell::new(HashMap::new());
    static WARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        pub fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
}

async fn load(dir: &'static str, file: &'static str, decode: fn(&[u8]) -> Option<Texture2D>) {
    let Ok(data) =
        js_sys::futures::JsFuture::from(js::ume_fetch_asset(&format!("{dir}{file}"))).await
    else {
        return;
    };
    if let Some(texture) = decode(&js_sys::Uint8Array::new(&data).to_vec()) {
        SLOTS.with(|slots| slots.borrow_mut().insert(file, Slot::Ready(texture)));
    }
}

fn claim(file: &'static str) -> bool {
    SLOTS.with(|slots| slots.borrow_mut().insert(file, Slot::Pending).is_none())
}

pub(crate) fn get(dir: &'static str, file: &'static str, decode: fn(&[u8]) -> Option<Texture2D>) -> Option<Texture2D> {
    let found = SLOTS.with(|slots| {
        slots.borrow().get(file).map(|slot| match slot {
            Slot::Ready(texture) => Some(texture.clone()),
            Slot::Pending => None,
        })
    });
    if let Some(texture) = found {
        return texture;
    }
    if claim(file) {
        wasm_bindgen_futures::spawn_local(load(dir, file, decode));
    }
    None
}

pub(crate) fn warm(dir: &'static str, files: &[&'static str], decode: fn(&[u8]) -> Option<Texture2D>) {
    if WARMED.with(|warmed| warmed.replace(true)) {
        return;
    }
    let files = files.to_vec();
    wasm_bindgen_futures::spawn_local(async move {
        for file in files {
            if claim(file) {
                load(dir, file, decode).await;
            }
        }
    });
}
