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

pub fn draw_button(p: Vec2, rr: f32, label: &str, base: Color, pressed: bool) {
    let fill = if pressed {
        Color::new(1.0, 1.0, 1.0, 0.6)
    } else {
        base
    };
    draw_circle(p.x, p.y, rr, Color::new(0.04, 0.02, 0.1, 0.35));
    draw_circle(p.x, p.y, rr * 0.92, fill);
    draw_circle(
        p.x,
        p.y - rr * 0.32,
        rr * 0.55,
        Color::new(1.0, 1.0, 1.0, 0.12),
    );
    draw_circle_lines(p.x, p.y, rr, 3.0, Color::new(1.0, 1.0, 1.0, 0.7));
    let mut fs = (rr * 0.5) as u16;
    while fs > 8 && measure_text(label, None, fs, 1.0).width > rr * 1.8 {
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
