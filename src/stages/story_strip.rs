use super::decode_illustration;
use std::sync::Arc;
use ume_engine::chardef::ImageData;

pub const STORY_ARENAS: [&str; 5] = ["brazil", "neon", "bamboo", "volcano", "dojo"];

#[cfg(not(target_arch = "wasm32"))]
pub fn story_strip(arena: &str) -> Option<Arc<ImageData>> {
    use std::sync::OnceLock;
    static STRIPS: [OnceLock<Arc<ImageData>>; 5] = [const { OnceLock::new() }; 5];
    let index = STORY_ARENAS.iter().position(|name| *name == arena)?;
    let bytes: [&[u8]; 5] = [
        include_bytes!("../../assets/stages/story/brazil.png"),
        include_bytes!("../../assets/stages/story/neon.png"),
        include_bytes!("../../assets/stages/story/bamboo.png"),
        include_bytes!("../../assets/stages/story/volcano.png"),
        include_bytes!("../../assets/stages/story/dojo.png"),
    ];
    Some(STRIPS[index].get_or_init(|| decode_illustration(bytes[index]).expect("embedded story strip")).clone())
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_STRIP: std::cell::RefCell<Option<(String, Arc<ImageData>)>> = const { std::cell::RefCell::new(None) };
    static WANT_STRIP: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
pub fn story_strip(arena: &str) -> Option<Arc<ImageData>> {
    WEB_STRIP.with(|cache| cache.borrow().as_ref().filter(|(key, _)| key == arena).map(|(_, image)| image.clone()))
}

#[cfg(target_arch = "wasm32")]
pub fn want_story_strip(arena: &str) {
    WANT_STRIP.with(|want| *want.borrow_mut() = Some(arena.to_string()));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn want_story_strip(_arena: &str) {}

#[cfg(target_arch = "wasm32")]
pub async fn prepare() -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
    let Some(arena) = WANT_STRIP.with(|want| want.borrow_mut().take()) else { return Ok(()) };
    if story_strip(&arena).is_some() || !STORY_ARENAS.contains(&arena.as_str()) {
        return Ok(());
    }
    let data = js_sys::futures::JsFuture::from(ume_fetch_asset(&format!("assets/stages/story/{arena}.png")))
        .await
        .map_err(|e| format!("Story strip download failed: {e:?}"))?;
    let image = decode_illustration(&js_sys::Uint8Array::new(&data).to_vec())?;
    WEB_STRIP.with(|cache| *cache.borrow_mut() = Some((arena, image)));
    Ok(())
}


pub fn illustration_layout(span: f32, aspect: f32, h: f32, view_w: f32, camera_x: f32, zoom: f32) -> (f32, f32, f32) {
    let h = if span > 0.0 { h.max(span / aspect) } else { h };
    let w = (h * aspect).max(view_w);
    let x = if span > 0.0 {
        ((view_w - w) * 0.5 - camera_x * w / span).clamp(view_w - w, 0.0)
    } else {
        ((view_w - w) * 0.5 - camera_x * 0.12 * zoom).clamp(view_w - w, 0.0)
    };
    (h, w, x)
}
