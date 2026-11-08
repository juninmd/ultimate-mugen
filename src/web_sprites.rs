use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn ume_fetch_asset(path: &str) -> js_sys::Promise;
}

#[derive(Clone)]
enum Request {
    Queued,
    Loading,
    Failed(String),
}

thread_local! {
    static REQUESTS: RefCell<HashMap<String, Request>> = RefCell::new(HashMap::new());
}

fn give_up(character: &str, error: String) -> Result<bool, String> {
    REQUESTS.with(|requests| {
        requests
            .borrow_mut()
            .insert(character.to_string(), Request::Failed(error.clone()));
    });
    Err(error)
}

pub fn prefetch_roster(ids: Vec<String>) {
    REQUESTS.with(|requests| {
        for id in &ids {
            requests
                .borrow_mut()
                .entry(id.clone())
                .or_insert(Request::Queued);
        }
    });
    wasm_bindgen_futures::spawn_local(async move {
        for id in ids {
            let _ = prepare(id).await;
        }
    });
}

pub fn pack_name(entry: &crate::registry::CharEntry) -> String {
    match &entry.src {
        crate::registry::Src::Native(native) => native.sprite_dir().to_string(),
        _ => entry.id.trim_start_matches("umf:").to_string(),
    }
}

pub fn ready(entry: &crate::registry::CharEntry) -> Result<bool, String> {
    if entry.mugen {
        return Ok(true);
    }
    let character = pack_name(entry);
    let character = character.as_str();
    if ume_engine::native::has_sprite_pack(character) {
        return Ok(true);
    }
    let request = REQUESTS.with(|requests| requests.borrow().get(character).cloned());
    match request {
        None => Ok(true),
        Some(Request::Failed(error)) => Err(error),
        Some(Request::Loading) => Ok(false),
        Some(Request::Queued) => {
            let character = character.to_string();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = prepare(character).await;
            });
            Ok(false)
        }
    }
}

pub async fn prepare(character: String) -> Result<bool, String> {
    if !character
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return give_up(
            &character,
            format!("sprite pack: invalid character id {character:?}"),
        );
    }
    if ume_engine::native::has_sprite_pack(&character) {
        return Ok(false);
    }
    let request = REQUESTS.with(|requests| requests.borrow().get(&character).cloned());
    match request {
        Some(Request::Failed(error)) => return Err(error),
        Some(Request::Loading) => return Ok(false),
        _ => {}
    }
    REQUESTS.with(|requests| {
        requests
            .borrow_mut()
            .insert(character.clone(), Request::Loading);
    });
    let promise = ume_fetch_asset(&format!("assets/chars/sprites/{character}.pack"));
    let data = match js_sys::futures::JsFuture::from(promise).await {
        Ok(data) => data,
        Err(_) => {
            return give_up(
                &character,
                format!("Unable to load sprites for {character}. Reload the page to retry."),
            )
        }
    };
    let bytes = js_sys::Uint8Array::new(&data).to_vec();
    match ume_engine::native::insert_sprite_pack(&character, &bytes) {
        Ok(()) => {
            REQUESTS.with(|requests| {
                requests.borrow_mut().remove(&character);
            });
            Ok(true)
        }
        Err(_) => give_up(
            &character,
            format!("Invalid sprites for {character}. Reload the page to retry."),
        ),
    }
}
