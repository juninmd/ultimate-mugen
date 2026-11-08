use super::layout::Rect;
use crate::gfx::*;
use crate::poly;
use crate::ui::Ui;

pub fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn corners(ui: &Ui, r: Rect, cut: f32) -> Vec<Vec2> {
    poly::chamfer(r.x, r.y, r.w, r.h, cut)
        .into_iter()
        .map(|p| vec2(ui.x(p.x), ui.y(p.y)))
        .collect()
}

pub fn pill(ui: &Ui, r: Rect, cut: f32, top: Color, bottom: Color, edge: Option<Color>) {
    let pts = corners(ui, r, cut);
    poly::grad_v(&pts, top, bottom);
    if let Some(e) = edge {
        poly::outline(&pts, (1.4 * ui.s).max(1.0), e);
    }
}

pub fn glass(ui: &Ui, r: Rect, radius: f32, tint: [f32; 3], a: f32) {
    if glass_panel(ui.x(r.x), ui.y(r.y), r.w * ui.s, r.h * ui.s, radius * ui.s, tint) {
        return;
    }
    pill(
        ui,
        r,
        radius * 0.6,
        Color::new(0.16 + tint[0] * 0.12, 0.18 + tint[1] * 0.1, 0.28 + tint[2] * 0.1, 0.86 * a),
        Color::new(0.05, 0.06, 0.12, 0.9 * a),
        Some(Color::new(1.0, 1.0, 1.0, 0.3 * a)),
    );
}

pub fn clip(ui: &Ui, r: Rect) {
    set_scissor(Some((
        ui.x(r.x).floor() as i32,
        ui.y(r.y).floor() as i32,
        (r.w * ui.s).ceil() as i32,
        (r.h * ui.s).ceil() as i32,
    )));
}

pub fn unclip() {
    set_scissor(None);
}
