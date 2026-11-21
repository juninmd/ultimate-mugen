use std::sync::Arc;

use crate::gfx::*;
use crate::ui::Align;

thread_local! {
    static COMBO_PLATE: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-combo-plate.png"));
    static GUARD_FRAME: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-guard-frame.png"));
    static GUARD_SHIELD: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-guard-shield.png"));
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

pub(super) fn draw_combo_plate<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    s: f32,
    x: f32,
    y: f32,
    al: Align,
    alpha: f32,
) -> bool {
    COMBO_PLATE.with(|plate| {
        let Some(tex) = plate else {
            return false;
        };
        let (w, h) = (44.0, 9.5);
        let left = match al {
            Align::Left => x - 4.0,
            Align::Center => x - w * 0.5,
            Align::Right => x - w + 4.0,
        };
        let tl = p(left, y - h * 0.72);
        draw_texture_ex(
            tex,
            tl.x,
            tl.y,
            Color::new(1.0, 1.0, 1.0, 0.92 * alpha),
            DrawTextureParams {
                dest_size: Some(vec2(w * s, h * s)),
                flip_x: matches!(al, Align::Right),
                ..Default::default()
            },
        );
        true
    })
}

pub(super) fn draw_guard_frame<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    s: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> bool {
    GUARD_FRAME.with(|frame| {
        let Some(tex) = frame else {
            return false;
        };
        let (px, py) = (1.2, 0.8);
        let tl = p(x - px, y - py);
        draw_texture_ex(
            tex,
            tl.x,
            tl.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2((w + px * 2.0) * s, (h + py * 2.0) * s)),
                ..Default::default()
            },
        );
        true
    })
}

pub(super) fn draw_guard_shield(at: Vec2, s: f32, cracked: bool) -> bool {
    GUARD_SHIELD.with(|shield| {
        let Some(tex) = shield else {
            return false;
        };
        let sz = 5.4 * s;
        let sx = if cracked { 24.0 } else { 0.0 };
        draw_texture_ex(
            tex,
            at.x - sz * 0.5,
            at.y - sz * 0.5,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(sx, 0.0, 24.0, 24.0)),
                dest_size: Some(vec2(sz, sz)),
                ..Default::default()
            },
        );
        true
    })
}
