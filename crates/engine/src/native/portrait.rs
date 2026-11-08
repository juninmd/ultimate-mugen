use super::pack::read_sprite;
use super::pixel::sprite_roots;
use super::recolor::Recolor;
use super::types::NativeChar;
use crate::chardef::{ImageData, Sprite};
use serde_json::Value;

const MAX_PORTRAIT_EDGE: u32 = 256;

pub(super) fn portrait(nc: &NativeChar, palette: Option<&Value>) -> Option<Sprite> {
    let recolor = palette.and_then(|palette| Recolor::new(nc.art.style.get("colors"), palette));
    sprite_roots(&nc.id).iter().find_map(|root| {
        let bytes = read_sprite(root, "portrait.png")?;
        let png = ume_formats::png::decode_png(&bytes).ok()?;
        let rgba = match &png.rgba {
            Some(rgba) => rgba.clone(),
            None => ume_formats::png::png_indexed_to_rgba(&png),
        };
        let (width, height, mut rgba) = portrait_thumbnail(png.width, png.height, rgba);
        if let Some(recolor) = &recolor {
            recolor.apply(&mut rgba);
        }
        Some(Sprite {
            image: Some(ImageData::new(width, height, rgba)),
            width: width as f32,
            height: height as f32,
            axis_x: 0.0,
            axis_y: 0.0,
            scale: 1.0,
        })
    })
}

fn portrait_thumbnail(width: u32, height: u32, rgba: Vec<u8>) -> (u32, u32, Vec<u8>) {
    if width.max(height) <= MAX_PORTRAIT_EDGE {
        return (width, height, rgba);
    }
    let (target_width, target_height) = if width >= height {
        (
            MAX_PORTRAIT_EDGE,
            ((height as u64 * MAX_PORTRAIT_EDGE as u64 + width as u64 / 2) / width as u64).max(1)
                as u32,
        )
    } else {
        (
            ((width as u64 * MAX_PORTRAIT_EDGE as u64 + height as u64 / 2) / height as u64).max(1)
                as u32,
            MAX_PORTRAIT_EDGE,
        )
    };
    let mut resized = Vec::with_capacity((target_width * target_height * 4) as usize);
    for y in 0..target_height {
        let source_y = ((2 * y as u64 + 1) * height as u64 / (2 * target_height as u64))
            .min(height as u64 - 1) as u32;
        for x in 0..target_width {
            let source_x = ((2 * x as u64 + 1) * width as u64 / (2 * target_width as u64))
                .min(width as u64 - 1) as u32;
            let source = ((source_y as u64 * width as u64 + source_x as u64) * 4) as usize;
            resized.extend_from_slice(&rgba[source..source + 4]);
        }
    }
    (target_width, target_height, resized)
}
