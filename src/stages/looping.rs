use std::sync::Arc;
use ume_engine::chardef::ImageData;

include!(concat!(env!("OUT_DIR"), "/stage_loops_gen.rs"));

const FRAME_COUNT: usize = 8;
const FRAME_TICKS: usize = 8;

#[cfg(not(target_arch = "wasm32"))]
pub fn frames(id: &str) -> Vec<Arc<ImageData>> {
    static FRAMES: std::sync::OnceLock<std::sync::Mutex<Option<(String, Vec<Arc<ImageData>>)>>> =
        std::sync::OnceLock::new();
    let mut cache = FRAMES.get_or_init(|| std::sync::Mutex::new(None)).lock().unwrap();
    if let Some((_, images)) = cache.as_ref().filter(|(cached_id, _)| cached_id == id) {
        return images.clone();
    }
    let Some((_, bytes)) = STAGE_LOOP_FRAMES.iter().find(|(stage_id, _)| *stage_id == id) else {
        return Vec::new();
    };
    let images = bytes
        .iter()
        .map(|bytes| decode_loop_frame(bytes).expect("stage loop frame"))
        .collect::<Vec<_>>();
    *cache = Some((id.to_string(), images.clone()));
    images
}

fn decode_loop_frame(bytes: &[u8]) -> Result<Arc<ImageData>, String> {
    let png = ume_formats::png::decode_png(bytes).map_err(|error| error.to_string())?;
    let rgba = png
        .rgba
        .clone()
        .unwrap_or_else(|| ume_formats::png::png_indexed_to_rgba(&png));
    Ok(ImageData::new(png.width, png.height, rgba))
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_FRAMES: std::cell::RefCell<Option<(String, Vec<Arc<ImageData>>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
pub fn frames(id: &str) -> Vec<Arc<ImageData>> {
    WEB_FRAMES.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .filter(|(key, _)| key == id)
            .map_or_else(Vec::new, |(_, frames)| frames.clone())
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn prepare(id: &str) -> Result<(), String> {
    if !STAGE_LOOP_IDS.contains(&id) || !frames(id).is_empty() {
        return Ok(());
    }
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
    let mut images = Vec::with_capacity(FRAME_COUNT);
    for frame in 0..FRAME_COUNT {
        let path = format!("assets/stages/animation/{id}/{frame:02}.png");
        let Ok(data) = js_sys::futures::JsFuture::from(ume_fetch_asset(&path)).await else {
            return Ok(());
        };
        let bytes = js_sys::Uint8Array::new(&data).to_vec();
        let Ok(image) = decode_loop_frame(&bytes) else {
            return Ok(());
        };
        images.push(image);
    }
    WEB_FRAMES.with(|cache| *cache.borrow_mut() = Some((id.to_string(), images)));
    Ok(())
}

pub fn frame_index(game_time: usize) -> usize {
    (game_time / FRAME_TICKS) % FRAME_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_sequence_loops_after_eight_frames() {
        assert_eq!(frame_index(0), 0);
        assert_eq!(frame_index(7), 0);
        assert_eq!(frame_index(8), 1);
        assert_eq!(frame_index(FRAME_COUNT * FRAME_TICKS), 0);
    }

    #[test]
    fn generated_loop_has_eight_full_size_distinct_frames() {
        let images = frames("colossus_rhodes");
        assert_eq!(images.len(), FRAME_COUNT);
        assert!(images.iter().all(|frame| (frame.w, frame.h) == (1672, 941)));
        assert!(images.windows(2).all(|pair| pair[0].rgba != pair[1].rgba));
        assert!(frames("unknown_stage").is_empty());
    }
}
