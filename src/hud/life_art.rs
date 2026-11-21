use std::sync::Arc;

use crate::gfx::*;

thread_local! {
    static LIFE_FRAME: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-life-frame.png"));
    static PORTRAIT_FRAME: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-portrait-frame.png"));
    static BOSS_FRAME: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-boss-frame.png"));
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

pub(super) fn draw_life_frame<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    s: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    flip_x: bool,
) -> bool {
    LIFE_FRAME.with(|frame| {
        let Some(tex) = frame else {
            return false;
        };
        let pad_x = 3.2;
        let pad_y = 2.2;
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

pub(super) fn draw_portrait_frame(pc: Vec2, half: f32, tint: Color) -> bool {
    PORTRAIT_FRAME.with(|frame| {
        let Some(tex) = frame else {
            return false;
        };
        let sz = half * 2.38;
        let col = Color::new(
            0.82 + tint.r * 0.18,
            0.82 + tint.g * 0.18,
            0.82 + tint.b * 0.18,
            tint.a,
        );
        draw_texture_ex(
            tex,
            pc.x - sz * 0.5,
            pc.y - sz * 0.5,
            col,
            DrawTextureParams {
                dest_size: Some(vec2(sz, sz)),
                ..Default::default()
            },
        );
        true
    })
}

pub(super) fn draw_boss_frame<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    s: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> bool {
    BOSS_FRAME.with(|frame| {
        let Some(tex) = frame else {
            return false;
        };
        let pad_x = 3.0;
        let pad_y = 2.4;
        let tl = p(x - pad_x, y - pad_y);
        draw_texture_ex(
            tex,
            tl.x,
            tl.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2((w + pad_x * 2.0) * s, (h + pad_y * 2.0) * s)),
                ..Default::default()
            },
        );
        true
    })
}
