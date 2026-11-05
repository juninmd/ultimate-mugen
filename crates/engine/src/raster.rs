use crate::chardef::ImageData;
use std::sync::Arc;
pub use tiny_skia::{
    Color, FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Paint, PathBuilder, Pixmap,
    Point, RadialGradient, Shader, SpreadMode, Stroke, Transform,
};

pub fn pixmap_to_image(p: &Pixmap) -> Arc<ImageData> {
    let len = p.data().len();
    let mut data = vec![0u8; len];
    let dst_chunks = data.as_chunks_mut::<4>().0;
    for (px, dst) in p.pixels().iter().zip(dst_chunks.iter_mut()) {
        let a = px.alpha();
        if a == 255 {
            dst[0] = px.red();
            dst[1] = px.green();
            dst[2] = px.blue();
            dst[3] = 255;
        } else if a == 0 {
            dst[0] = 0;
            dst[1] = 0;
            dst[2] = 0;
            dst[3] = 0;
        } else {
            let c = px.demultiply();
            dst[0] = c.red();
            dst[1] = c.green();
            dst[2] = c.blue();
            dst[3] = a;
        }
    }
    ImageData::new(p.width(), p.height(), data)
}

pub fn rgba(hex: &str, a: f32) -> Color {
    let h = hex.trim_start_matches('#');
    let v = u32::from_str_radix(h, 16).unwrap_or(0x808080);
    let (r, g, b) = if h.len() == 3 {
        (((v >> 8) & 15) * 17, ((v >> 4) & 15) * 17, (v & 15) * 17)
    } else {
        ((v >> 16) & 255, ((v >> 8) & 255), v & 255)
    };
    Color::from_rgba8(r as u8, g as u8, b as u8, (a.clamp(0.0, 1.0) * 255.0) as u8)
}

pub fn solid(c: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(c);
    p.anti_alias = true;
    p
}

pub fn circle(x: f32, y: f32, r: f32) -> Option<tiny_skia::Path> {
    PathBuilder::from_circle(x, y, r.max(0.01))
}
