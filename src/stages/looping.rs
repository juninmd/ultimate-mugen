use std::sync::Arc;
use ume_engine::chardef::ImageData;

include!(concat!(env!("OUT_DIR"), "/stage_loops_gen.rs"));

const KEYFRAME_COUNT: usize = 8;
const LOOP_TICKS: usize = 64;
const SMOOTH_LOOP_TICKS: usize = 96;

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
    let images = ume_formats::par::map(bytes, |raw| decode_loop_frame(raw).expect("stage loop frame"));
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
    let suffixes: &[&str] = if STAGE_LOOP_MIDPOINT_IDS.contains(&id) { &["", "-mid"] } else { &[""] };
    let names = (0..KEYFRAME_COUNT).flat_map(|frame| suffixes.iter().map(move |suffix| {
        format!("assets/stages/animation/{id}/{frame:02}{suffix}.png")
    }));
    let mut images = Vec::with_capacity(KEYFRAME_COUNT * suffixes.len());
    for path in names {
        let Ok(data) = js_sys::futures::JsFuture::from(ume_fetch_asset(&path)).await else {
            return Ok(());
        };
        let Ok(image) = super::web_decode::decode_opaque_illustration(&js_sys::Uint8Array::new(&data)).await else {
            return Ok(());
        };
        images.push(image);
    }
    WEB_FRAMES.with(|cache| *cache.borrow_mut() = Some((id.to_string(), images)));
    Ok(())
}

pub fn frame_index(game_time: usize, frame_count: usize) -> usize {
    if frame_count == 0 {
        return 0;
    }
    (game_time % LOOP_TICKS) * frame_count / LOOP_TICKS
}

pub fn loop_sample(game_time: usize, frame_count: usize) -> (usize, usize, f32) {
    if frame_count <= 1 {
        return (0, 0, 0.0);
    }
    let phase = (game_time % SMOOTH_LOOP_TICKS) as f32 / SMOOTH_LOOP_TICKS as f32;
    let pos = phase * frame_count as f32;
    let a = (pos.floor() as usize) % frame_count;
    let b = (a + 1) % frame_count;
    let t = (pos - pos.floor()).clamp(0.0, 1.0);
    let mix = t * t * (3.0 - 2.0 * t);
    (a, b, mix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_frame_counts_preserve_the_cycle() {
        assert_eq!(frame_index(0, 8), 0);
        assert_eq!(frame_index(7, 8), 0);
        assert_eq!(frame_index(8, 8), 1);
        assert_eq!(frame_index(LOOP_TICKS, 8), 0);
        assert_eq!(frame_index(4, 16), 1);
        assert_eq!(frame_index(63, 16), 15);
        assert_eq!(frame_index(LOOP_TICKS, 16), 0);
    }

    #[test]
    fn smooth_loop_wraps_seamlessly_from_last_frame_to_zero() {
        let (a0, b0, m0) = loop_sample(0, 16);
        assert_eq!((a0, b0), (0, 1));
        assert!(m0.abs() < 1e-6);
        let (al, bl, ml) = loop_sample(SMOOTH_LOOP_TICKS - 1, 16);
        assert_eq!((al, bl), (15, 0));
        assert!(ml > 0.7 && ml < 1.0);
        let (aw, bw, mw) = loop_sample(SMOOTH_LOOP_TICKS, 16);
        assert_eq!((aw, bw), (0, 1));
        assert!(mw.abs() < 1e-6);
    }

    #[test]
    fn midpoint_loop_has_sixteen_distinct_full_size_frames() {
        let images = frames("great_wall");
        assert_eq!(images.len(), KEYFRAME_COUNT * 2);
        assert!(images.iter().all(|frame| (frame.w, frame.h) == (1672, 941)));
        assert!(images.windows(2).all(|pair| pair[0].rgba != pair[1].rgba));
        assert_ne!(images.first().unwrap().rgba, images.last().unwrap().rgba);
    }

    #[test]
    fn every_builtin_stage_has_a_complete_motion_loop() {
        assert_eq!(STAGE_LOOP_FRAMES.len(), 21);
        assert!(STAGE_LOOP_FRAMES.iter().all(|(_, frames)| frames.len() == KEYFRAME_COUNT * 2));
        assert!(frames("unknown_stage").is_empty());
    }
}
