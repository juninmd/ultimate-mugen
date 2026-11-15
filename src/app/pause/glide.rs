use super::layout::Rect;
use super::skin::{fade, pill};
use crate::gfx::*;
use crate::ui::Ui;

pub(super) fn glide(key_x: &'static str, key_y: &'static str, r: Rect) -> Rect {
    Rect::new(
        crate::spring::follow(key_x, r.x),
        crate::spring::follow(key_y, r.y),
        r.w,
        r.h,
    )
}

pub(super) fn highlight(ui: &Ui, r: Rect, cut: f32, a: f32) {
    let halo = Rect::new(r.x - 4.0, r.y - 4.0, r.w + 8.0, r.h + 8.0);
    pill(
        ui,
        halo,
        cut + 3.0,
        fade(crate::tweaks::accent(0.22), a),
        fade(crate::tweaks::accent(0.08), a),
        None,
    );
    pill(
        ui,
        r,
        cut,
        fade(crate::tweaks::accent(0.9), a),
        fade(crate::tweaks::accent_deep(0.75), a),
        Some(fade(Color::new(1.0, 0.85, 0.9, 1.0), 0.7 * a)),
    );
}

pub(super) fn cost_colors(cost: &str) -> (Color, Color) {
    if cost.contains("BAR") {
        (crate::ui::hex("#ffcc33"), Color::new(1.0, 0.8, 0.2, 0.2))
    } else {
        (crate::ui::hex("#5ee8ff"), Color::new(0.37, 0.91, 1.0, 0.16))
    }
}
