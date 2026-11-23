use std::sync::Arc;
use ume_engine::chardef::ImageData;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn ume_decode_image(bytes: &js_sys::Uint8Array) -> js_sys::Promise;
}

pub(crate) async fn decode_opaque_illustration(bytes: &js_sys::Uint8Array) -> Result<Arc<ImageData>, String> {
    if let Some(image) = browser_decode(bytes).await {
        return Ok(image);
    }
    let png = ume_formats::png::decode_png(&bytes.to_vec())?;
    let rgba = png.rgba.clone().unwrap_or_else(|| ume_formats::png::png_indexed_to_rgba(&png));
    Ok(ImageData::new(png.width, png.height, rgba))
}

async fn browser_decode(bytes: &js_sys::Uint8Array) -> Option<Arc<ImageData>> {
    let result = js_sys::futures::JsFuture::from(ume_decode_image(bytes)).await.ok()?;
    let field = |name: &str| js_sys::Reflect::get(&result, &JsValue::from_str(name)).ok();
    let width = field("width")?.as_f64()? as u32;
    let height = field("height")?.as_f64()? as u32;
    let data = js_sys::Uint8Array::new(&field("data")?).to_vec();
    (data.len() == width as usize * height as usize * 4).then(|| ImageData::new(width, height, data))
}
