use super::touch::{CHARGE, EXTRA_START, QUICK_START};
use crate::gfx::*;
use std::sync::Arc;

thread_local! {
    static STICK_BASE: Option<Texture2D> = load_skin(include_bytes!("../../assets/ui/touch/stick-base.png"));
    static STICK_KNOB: Option<Texture2D> = load_skin(include_bytes!("../../assets/ui/touch/stick-knob.png"));
    static BUTTON: Option<Texture2D> = load_skin(include_bytes!("../../assets/ui/touch/button.png"));
}

fn load_skin(bytes: &[u8]) -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(bytes).ok()?;
    Some(Texture2D::from_rgba_shared(
        image.width as u16,
        image.height as u16,
        Arc::new(image.rgba?),
    ))
}

fn draw_skin(texture: &Option<Texture2D>, center: Vec2, radius: f32, tint: Color) -> bool {
    let Some(texture) = texture else {
        return false;
    };
    draw_texture_ex(
        texture,
        center.x - radius,
        center.y - radius,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(radius * 2.0, radius * 2.0)),
            ..Default::default()
        },
    );
    true
}

pub fn base_color(i: usize) -> Color {
    match i {
        0 | 1 => Color::new(0.2, 0.55, 1.0, 0.45),
        3 | 4 => Color::new(1.0, 0.27, 0.4, 0.45),
        2 => Color::new(1.0, 0.75, 0.15, 0.45),
        5 => Color::new(0.6, 0.2, 0.95, 0.55),
        CHARGE => Color::new(1.0, 0.8, 0.2, 0.5),
        12 | 16 => Color::new(0.6, 0.6, 0.65, 0.5),
        13 => Color::new(1.0, 0.75, 0.15, 0.45),
        14 => Color::new(0.2, 0.55, 1.0, 0.45),
        15 => Color::new(1.0, 0.27, 0.4, 0.45),
        QUICK_START..EXTRA_START => Color::new(0.2, 0.9, 0.8, 0.4),
        _ => Color::new(1.0, 0.45, 0.1, 0.5),
    }
}

pub fn draw_stick(c: Vec2, sr: f32, knob: Vec2) {
    let base_tint = Color::new(1.0, 1.0, 1.0, 0.56);
    let knob_tint = Color::new(1.0, 1.0, 1.0, 0.74);
    if !STICK_BASE.with(|texture| draw_skin(texture, c, sr, base_tint)) {
        draw_circle(c.x, c.y, sr, Color::new(1.0, 1.0, 1.0, 0.1));
        draw_circle_lines(c.x, c.y, sr, 2.5, base_tint);
    }
    let center = c + knob;
    let radius = sr * 0.42;
    if !STICK_KNOB.with(|texture| draw_skin(texture, center, radius, knob_tint)) {
        draw_circle(center.x, center.y, radius, Color::new(1.0, 1.0, 1.0, 0.45));
        draw_circle_lines(center.x, center.y, radius, 2.0, knob_tint);
    }
}

pub fn draw_button(p: Vec2, rr: f32, label: &str, base: Color, pressed: bool) {
    let fill = if pressed {
        Color::new(1.0, 1.0, 1.0, 0.6)
    } else {
        Color::new(base.r, base.g, base.b, base.a * 0.75)
    };
    let tint = if pressed {
        Color::new(1.0, 1.0, 1.0, 0.88)
    } else {
        Color::new(
            0.46 + base.r * 0.54,
            0.46 + base.g * 0.54,
            0.46 + base.b * 0.54,
            0.5 + base.a * 0.3,
        )
    };
    if !BUTTON.with(|texture| draw_skin(texture, p, rr, tint)) {
        draw_circle(p.x, p.y, rr * 0.92, fill);
        draw_circle_lines(p.x, p.y, rr, 2.0, Color::new(1.0, 1.0, 1.0, 0.55));
    }
    let mut fs = (rr * 0.5) as u16;
    while fs > 6 && measure_text(label, None, fs, 1.0).width > rr * 1.45 {
        fs -= 1;
    }
    let d = measure_text(label, None, fs, 1.0);
    let (x, y) = (p.x - d.width / 2.0, p.y + d.offset_y / 2.0);
    draw_text(
        label,
        x + 1.0,
        y + 1.0,
        fs as f32,
        Color::new(0.0, 0.0, 0.0, 0.5),
    );
    draw_text(label, x, y, fs as f32, WHITE);
}
