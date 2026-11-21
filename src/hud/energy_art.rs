use std::sync::Arc;

use crate::gfx::*;

thread_local! {
    static ENERGY_FRAME: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-energy-frame.png"));
    static ENERGY_SPARK: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-energy-spark.png"));
}

fn load_pixel_art(data: &[u8]) -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(data).ok()?;
    let rgba = match image.rgba {
        Some(rgba) => rgba,
        None => ume_formats::png::png_indexed_to_rgba(&image),
    };
    let tex = Texture2D::from_rgba_shared(image.width as u16, image.height as u16, Arc::new(rgba));
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

pub(super) fn draw_frame<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    s: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    flip_x: bool,
) -> bool {
    ENERGY_FRAME.with(|frame| {
        let Some(tex) = frame else {
            return false;
        };
        let pad_x = 1.4;
        let pad_y = 1.0;
        let tl = p(x - pad_x, y - pad_y);
        draw_texture_ex(
            tex,
            tl.x,
            tl.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2((w + pad_x * 2.0) * s, (h + pad_y * 2.0) * s)),
                flip_x,
                ..Default::default()
            },
        );
        true
    })
}

pub(super) fn draw_spark(at: Vec2, s: f32, gt: f32, tint: Color) -> bool {
    ENERGY_SPARK.with(|spark| {
        let Some(tex) = spark else {
            return false;
        };
        let frame = ((gt * 12.0) as usize) % 4;
        let sz = 6.5 * s;
        draw_texture_ex(
            tex,
            at.x - sz * 0.5,
            at.y - sz * 0.5,
            tint,
            DrawTextureParams {
                source: Some(Rect::new(frame as f32 * 16.0, 0.0, 16.0, 16.0)),
                dest_size: Some(vec2(sz, sz)),
                ..Default::default()
            },
        );
        true
    })
}
