use super::decode_illustration;
use std::sync::Arc;
use ume_engine::chardef::ImageData;

const FRAME_COUNT: usize = 8;
const FRAME_TICKS: usize = 8;

#[cfg(not(target_arch = "wasm32"))]
pub fn frames(id: &str) -> Vec<Arc<ImageData>> {
    if id != "colossus_rhodes" {
        return Vec::new();
    }
    static FRAMES: std::sync::OnceLock<Vec<Arc<ImageData>>> = std::sync::OnceLock::new();
    FRAMES
        .get_or_init(|| {
            [
                include_bytes!("../../assets/stages/animation/colossus_rhodes/00.png").as_slice(),
                include_bytes!("../../assets/stages/animation/colossus_rhodes/01.png").as_slice(),
                include_bytes!("../../assets/stages/animation/colossus_rhodes/02.png").as_slice(),
                include_bytes!("../../assets/stages/animation/colossus_rhodes/03.png").as_slice(),
                include_bytes!("../../assets/stages/animation/colossus_rhodes/04.png").as_slice(),
                include_bytes!("../../assets/stages/animation/colossus_rhodes/05.png").as_slice(),
                include_bytes!("../../assets/stages/animation/colossus_rhodes/06.png").as_slice(),
                include_bytes!("../../assets/stages/animation/colossus_rhodes/07.png").as_slice(),
            ]
            .iter()
            .map(|bytes| decode_illustration(bytes).expect("stage loop frame"))
            .collect()
        })
        .clone()
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
    if id != "colossus_rhodes" || !frames(id).is_empty() {
        return Ok(());
    }
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
    let mut images = Vec::with_capacity(FRAME_COUNT);
    for frame in 0..FRAME_COUNT {
        let path = format!("assets/stages/animation/colossus_rhodes/{frame:02}.png");
        let Ok(data) = js_sys::futures::JsFuture::from(ume_fetch_asset(&path)).await else {
            return Ok(());
        };
        let Ok(image) = decode_illustration(&js_sys::Uint8Array::new(&data).to_vec()) else {
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
        assert!(frames("neon").is_empty());
    }
}
