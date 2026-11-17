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
