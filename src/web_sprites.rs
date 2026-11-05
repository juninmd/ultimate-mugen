use std::cell::RefCell;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn ume_fetch_asset(path: &str) -> js_sys::Promise;
}

thread_local! {
    static MISSING: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

fn give_up(character: &str, error: String) -> Result<bool, String> {
    MISSING.with(|missing| { missing.borrow_mut().insert(character.to_string()); });
    Err(error)
}

pub fn prefetch_roster(ids: Vec<String>) {
    wasm_bindgen_futures::spawn_local(async move {
        for id in ids {
            let _ = prepare(id).await;
        }
    });
}

pub async fn prepare(character: String) -> Result<bool, String> {
    if !character.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return give_up(&character, format!("sprite pack: invalid character id {character:?}"));
    }
    if ume_engine::native::has_sprite_pack(&character) || MISSING.with(|missing| missing.borrow().contains(&character)) {
        return Ok(false);
    }
    let promise = ume_fetch_asset(&format!("assets/chars/sprites/{character}.pack"));
    let data = match js_sys::futures::JsFuture::from(promise).await {
        Ok(data) => data,
        Err(error) => return give_up(&character, format!("sprite pack {character}: {error:?}")),
    };
    let bytes = js_sys::Uint8Array::new(&data).to_vec();
    match ume_engine::native::insert_sprite_pack(&character, &bytes) {
        Ok(()) => Ok(true),
        Err(error) => give_up(&character, error),
    }
}
