use super::quantize::quantize;
use super::resize::{lanczos_resize, Image};
use ume_charforge::spritekit::encode_palette_png;
use ume_formats::png::{decode_png, png_indexed_to_rgba};

pub const MAX_EDGE: usize = 768;
pub const PORTRAIT_EDGE: usize = 256;
pub const COLORS: usize = 256;

pub fn encode_sprite(name: &str, data: &[u8]) -> Result<(Vec<u8>, f64), String> {
    let png = decode_png(data).map_err(|e| format!("{name}: {e}"))?;
    let rgba = match &png.rgba {
        Some(rgba) => rgba.clone(),
        None => png_indexed_to_rgba(&png),
    };
    let mut image = Image { width: png.width as usize, height: png.height as usize, rgba };
    let limit = if name == "portrait.png" { PORTRAIT_EDGE } else { MAX_EDGE };
    let scale = (limit as f64 / image.width.max(image.height) as f64).min(1.0);
    if scale < 1.0 {
        let width = ((image.width as f64 * scale).round() as usize).max(1);
        let height = ((image.height as f64 * scale).round() as usize).max(1);
        image = lanczos_resize(&image, width, height);
    }
    let (palette, indices) = quantize(&image.rgba, COLORS);
    Ok((encode_palette_png(image.width, image.height, &indices, &palette, 9), scale))
}
