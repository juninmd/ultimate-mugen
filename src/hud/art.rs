use std::sync::Arc;

use crate::gfx::*;

thread_local! {
    static TIMER_CREST: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-timer-crest.png"));
    static ROUND_ORB: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-round-orb.png"));
    static BANNER_RIBBON: Option<Texture2D> = load_pixel_art(include_bytes!("../../assets/ui/hud-banner-ribbon.png"));
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

pub(super) fn draw_timer_crest(c: Vec2, r: f32) -> bool {
    TIMER_CREST.with(|crest| {
        let Some(tex) = crest else {
            return false;
        };
        let sz = r * 3.15;
        draw_texture_ex(
            tex,
            c.x - sz * 0.5,
            c.y - sz * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(sz, sz)),
                ..Default::default()
            },
        );
        true
    })
}

pub(super) fn draw_round_orb(c: Vec2, s: f32, won: bool) -> bool {
    ROUND_ORB.with(|orb| {
        let Some(tex) = orb else {
            return false;
        };
        let sz = 5.4 * s;
        let sx = if won { 32.0 } else { 0.0 };
        draw_texture_ex(
            tex,
            c.x - sz * 0.5,
            c.y - sz * 0.5,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(sx, 0.0, 32.0, 32.0)),
                dest_size: Some(vec2(sz, sz)),
                ..Default::default()
            },
        );
        true
    })
}

pub(super) fn draw_banner_ribbon(c: Vec2, w: f32, h: f32, alpha: f32) -> bool {
    BANNER_RIBBON.with(|ribbon| {
        let Some(tex) = ribbon else {
            return false;
        };
        draw_texture_ex(
            tex,
            c.x - w * 0.5,
            c.y - h * 0.5,
            Color::new(1.0, 1.0, 1.0, alpha),
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
        true
    })
}
