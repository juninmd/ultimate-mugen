use std::sync::Arc;
use ume_engine::chardef::ImageData;

const FRAME_COUNT: usize = 8;
const FRAME_TICKS: usize = 8;

macro_rules! stage_bytes {
    ($id:literal) => {
        [
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/00.png")).as_slice(),
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/01.png")).as_slice(),
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/02.png")).as_slice(),
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/03.png")).as_slice(),
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/04.png")).as_slice(),
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/05.png")).as_slice(),
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/06.png")).as_slice(),
            include_bytes!(concat!("../../assets/stages/animation/", $id, "/07.png")).as_slice(),
        ]
    };
}

#[cfg(not(target_arch = "wasm32"))]
fn stage_bytes_for(id: &str) -> Option<[&'static [u8]; FRAME_COUNT]> {
    match id {
        "dojo" => Some(stage_bytes!("dojo")),
        "neon" => Some(stage_bytes!("neon")),
        "bamboo" => Some(stage_bytes!("bamboo")),
        "plains" => Some(stage_bytes!("plains")),
        "spirit" => Some(stage_bytes!("spirit")),
        "volcano" => Some(stage_bytes!("volcano")),
        "brazil" => Some(stage_bytes!("brazil")),
        "great_wall" => Some(stage_bytes!("great_wall")),
        "petra" => Some(stage_bytes!("petra")),
        "christ_redeemer" => Some(stage_bytes!("christ_redeemer")),
        "texas" => Some(stage_bytes!("texas")),
        "chichen_itza" => Some(stage_bytes!("chichen_itza")),
        "colosseum" => Some(stage_bytes!("colosseum")),
        "taj_mahal" => Some(stage_bytes!("taj_mahal")),
        "pyramids_giza" => Some(stage_bytes!("pyramids_giza")),
        "hanging_gardens" => Some(stage_bytes!("hanging_gardens")),
        "temple_artemis" => Some(stage_bytes!("temple_artemis")),
        "statue_zeus" => Some(stage_bytes!("statue_zeus")),
        "mausoleum_halicarnassus" => Some(stage_bytes!("mausoleum_halicarnassus")),
        "colossus_rhodes" => Some(stage_bytes!("colossus_rhodes")),
        "lighthouse_alexandria" => Some(stage_bytes!("lighthouse_alexandria")),
        _ => None,
    }
}

fn is_animated(id: &str) -> bool {
    matches!(id, "dojo" | "neon" | "bamboo" | "plains" | "spirit" | "volcano" | "brazil" | "great_wall" | "petra" | "christ_redeemer" | "texas" | "chichen_itza" | "colosseum" | "taj_mahal" | "pyramids_giza" | "hanging_gardens" | "temple_artemis" | "statue_zeus" | "mausoleum_halicarnassus" | "colossus_rhodes" | "lighthouse_alexandria")
}

fn decode_frame(bytes: &[u8], base: &ImageData) -> Result<Arc<ImageData>, String> {
    let png = ume_formats::png::decode_png(bytes)?;
    let rgba = match (png.rgba, png.indexed, png.palette) {
        (Some(rgba), _, _) => rgba,
        (None, Some(indices), Some(palette)) => {
            let mut rgba = Vec::with_capacity(indices.len() * 4);
            for index in indices {
                let offset = index as usize * 4;
                rgba.extend_from_slice(&palette[offset..offset + 4]);
            }
            rgba
        }
        _ => return Err("stage loop PNG has no pixel data".into()),
    };
    if (png.width, png.height) != (base.w, base.h) {
        return Err("stage loop overlay does not match its background".into());
    }
    let mut composed = base.rgba.to_vec();
    for (pixel, overlay) in composed.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
        let alpha = overlay[3] as u16;
        if alpha == 0 {
            continue;
        }
        for channel in 0..3 {
            pixel[channel] = ((overlay[channel] as u16 * alpha
                + pixel[channel] as u16 * (255 - alpha)
                + 127)
                / 255) as u8;
        }
    }
    Ok(ImageData::new(png.width, png.height, composed))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn frames(id: &str) -> Vec<Arc<ImageData>> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Option<(String, Vec<Arc<ImageData>>)>>> =
        std::sync::OnceLock::new();
    let Some(bytes) = stage_bytes_for(id) else {
        return Vec::new();
    };
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = cache.lock().expect("stage frame cache");
    if let Some((_, images)) = guard.as_ref().filter(|(cached_id, _)| cached_id == id) {
        return images.clone();
    }
    let Some(base) = super::illustration(id) else {
        return Vec::new();
    };
    let images = bytes
        .iter()
        .map(|frame| decode_frame(frame, &base).expect("built-in stage loop frame"))
        .collect::<Vec<_>>();
    *guard = Some((id.to_owned(), images.clone()));
    images
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_FRAMES: std::cell::RefCell<Option<(String, Vec<Arc<ImageData>>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
pub fn frames(id: &str) -> Vec<Arc<ImageData>> {
    WEB_FRAMES.with(|cache| {
        cache.borrow().as_ref().filter(|(key, _)| key == id)
            .map_or_else(Vec::new, |(_, frames)| frames.clone())
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn prepare(id: &str) -> Result<(), String> {
    if !is_animated(id) || !frames(id).is_empty() {
        return Ok(());
    }
    let Some(base) = super::illustration(id) else {
        return Ok(());
    };
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
    let mut images = Vec::with_capacity(FRAME_COUNT);
    for frame in 0..FRAME_COUNT {
        let path = format!("assets/stages/animation/{id}/{frame:02}.png");
        let Ok(data) = js_sys::futures::JsFuture::from(ume_fetch_asset(&path)).await else { return Ok(()); };
        let Ok(image) = decode_frame(&js_sys::Uint8Array::new(&data).to_vec(), &base) else { return Ok(()); };
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
    fn every_native_stage_has_eight_full_size_distinct_frames() {
        for id in [
            "dojo",
            "neon",
            "bamboo",
            "plains",
            "spirit",
            "volcano",
            "brazil",
            "great_wall",
            "petra",
            "christ_redeemer",
            "texas",
            "chichen_itza",
            "colosseum",
            "taj_mahal",
            "pyramids_giza",
            "hanging_gardens",
            "temple_artemis",
            "statue_zeus",
            "mausoleum_halicarnassus",
            "colossus_rhodes",
            "lighthouse_alexandria",
        ] {
            let images = frames(id);
            assert_eq!(images.len(), FRAME_COUNT, "{id}");
            assert!(images.iter().all(|frame| (frame.w, frame.h) == (1672, 941)), "{id}");
            assert!(images.windows(2).all(|pair| pair[0].rgba != pair[1].rgba), "{id}");
        }
        assert!(frames("custom").is_empty());
    }
}
