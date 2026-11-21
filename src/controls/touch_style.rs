use super::touch::{CHARGE, EXTRA_START, QUICK_START};
use crate::gfx::*;

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
    draw_circle(c.x, c.y, sr, Color::new(1.0, 1.0, 1.0, 0.1));
    draw_circle_lines(c.x, c.y, sr, 2.5, Color::new(1.0, 1.0, 1.0, 0.55));
    draw_circle(c.x + knob.x, c.y + knob.y, sr * 0.42, Color::new(1.0, 1.0, 1.0, 0.45));
    draw_circle_lines(c.x + knob.x, c.y + knob.y, sr * 0.42, 2.0, Color::new(1.0, 1.0, 1.0, 0.7));
}

pub fn draw_button(p: Vec2, rr: f32, label: &str, base: Color, pressed: bool) {
    let fill = if pressed {
        Color::new(1.0, 1.0, 1.0, 0.6)
    } else {
        Color::new(base.r, base.g, base.b, base.a * 0.75)
    };
    draw_circle(p.x, p.y, rr * 0.92, fill);
    draw_circle_lines(p.x, p.y, rr, 2.0, Color::new(1.0, 1.0, 1.0, 0.55));
    let mut fs = (rr * 0.5) as u16;
    while fs > 6 && measure_text(label, None, fs, 1.0).width > rr * 1.8 {
        fs -= 1;
    }
    let d = measure_text(label, None, fs, 1.0);
    let (x, y) = (p.x - d.width / 2.0, p.y + d.offset_y / 2.0);
    draw_text(label, x + 1.0, y + 1.0, fs as f32, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text(label, x, y, fs as f32, WHITE);
}
