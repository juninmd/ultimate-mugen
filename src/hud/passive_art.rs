use std::sync::Arc;

use crate::gfx::*;

thread_local! {
    static PASSIVE_GEM: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-passive-gem.png"));
    static PASSIVE_RAIL: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-passive-rail.png"));
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

pub(super) fn draw_rail<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    s: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> bool {
    PASSIVE_RAIL.with(|rail| {
        let Some(tex) = rail else {
            return false;
        };
        let (px, py) = (0.8, 0.5);
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

pub(super) fn draw_gem(at: Vec2, sz: f32, state: u8, alpha: f32) -> bool {
    PASSIVE_GEM.with(|gem| {
        let Some(tex) = gem else {
            return false;
        };
        let sx = (state.min(2) as f32) * 16.0;
        draw_texture_ex(
            tex,
            at.x - sz * 0.5,
            at.y - sz * 0.5,
            Color::new(1.0, 1.0, 1.0, alpha),
            DrawTextureParams {
                source: Some(Rect::new(sx, 0.0, 16.0, 16.0)),
                dest_size: Some(vec2(sz, sz)),
                ..Default::default()
            },
        );
        true
    })
}
