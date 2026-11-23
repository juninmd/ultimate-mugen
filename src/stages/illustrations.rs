use super::*;

pub(super) const ILLUSTRATION_IDS: [&str; 7] = [
    "dojo", "neon", "bamboo", "plains", "spirit", "volcano", "brazil",
];

pub(crate) fn decode_illustration(bytes: &[u8]) -> Result<Arc<ImageData>, String> {
    let png = ume_formats::png::decode_png(bytes).map_err(|e| e.to_string())?;
    let rgba = png.rgba.ok_or("expected truecolor stage PNG")?;
    Ok(ImageData::new(png.width, png.height, rgba))
}

pub fn illustration_preview(id: &str) -> Option<Arc<ImageData>> {
    if let Some(image) = wonders::illustration_preview(id) {
        return Some(image);
    }
    use std::sync::OnceLock;
    static IMAGES: [OnceLock<Arc<ImageData>>; 7] = [const { OnceLock::new() }; 7];
    let index = ILLUSTRATION_IDS.iter().position(|name| *name == id)?;
    let bytes: [&[u8]; 7] = [
        include_bytes!("../../assets/stages/thumbs/dojo.png"),
        include_bytes!("../../assets/stages/thumbs/neon.png"),
        include_bytes!("../../assets/stages/thumbs/bamboo.png"),
        include_bytes!("../../assets/stages/thumbs/plains.png"),
        include_bytes!("../../assets/stages/thumbs/spirit.png"),
        include_bytes!("../../assets/stages/thumbs/volcano.png"),
        include_bytes!("../../assets/stages/thumbs/brazil.png"),
    ];
    Some(
        IMAGES[index]
            .get_or_init(|| decode_illustration(bytes[index]).expect("embedded stage thumbnail"))
            .clone(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn illustration(id: &str) -> Option<Arc<ImageData>> {
    if let Some(image) = wonders::illustration(id) {
        return Some(image);
    }
    use std::sync::OnceLock;
    static IMAGES: [OnceLock<Arc<ImageData>>; 7] = [const { OnceLock::new() }; 7];
    let index = ILLUSTRATION_IDS.iter().position(|name| *name == id)?;
    let bytes: [&[u8]; 7] = [
        include_bytes!("../../assets/stages/dojo.png"),
        include_bytes!("../../assets/stages/neon.png"),
        include_bytes!("../../assets/stages/bamboo.png"),
        include_bytes!("../../assets/stages/plains.png"),
        include_bytes!("../../assets/stages/spirit.png"),
        include_bytes!("../../assets/stages/volcano.png"),
        include_bytes!("../../assets/stages/brazil.png"),
    ];
    Some(
        IMAGES[index]
            .get_or_init(|| decode_illustration(bytes[index]).expect("embedded stage PNG"))
            .clone(),
    )
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_ILLUSTRATION: std::cell::RefCell<Option<(String, Arc<ImageData>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
pub fn illustration(id: &str) -> Option<Arc<ImageData>> {
    WEB_ILLUSTRATION.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .filter(|(key, _)| key == id)
            .map(|(_, image)| image.clone())
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn prepare_illustration(id: &str) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        fn ume_fetch_asset(path: &str) -> js_sys::Promise;
    }
    story_strip::prepare().await?; looping::prepare(id).await?;
    if illustration(id).is_some() {
        return Ok(());
    }
    if !ILLUSTRATION_IDS.contains(&id) && !wonders::WONDER_IDS.contains(&id) {
        return Err(format!("Unknown stage: {id}"));
    }
    let data = js_sys::futures::JsFuture::from(ume_fetch_asset(&format!("assets/stages/{id}.png")))
        .await
        .map_err(|e| format!("Stage download failed: {e:?}"))?;
    let image = super::web_decode::decode_opaque_illustration(&js_sys::Uint8Array::new(&data)).await?;
    WEB_ILLUSTRATION.with(|cache| *cache.borrow_mut() = Some((id.to_string(), image)));
    Ok(())
}

pub(super) fn illustrated(id: &str, name: &str, music: u32, kind: StageKind) -> Stage {
    Stage {
        info: info(name),
        art: StageArt::Illustrated { image: illustration(id).expect("builtin stage image"), frames: looping::frames(id), ambience: ambience::profile_for(id), span: 0.0 },
        music,
        kind,
        bgm: None,
    }
}
