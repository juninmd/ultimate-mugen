use crate::gfx::*;
use crate::poly;
use crate::ui::Align;

pub(super) fn draw_badge<
    P: Fn(f32, f32) -> Vec2,
    T: Fn(&str, f32, f32, f32, Color, Align, bool),
>(
    p: &P,
    text: &T,
    s: f32,
    gt: f32,
    bx: f32,
    by: f32,
    bw: f32,
    side: usize,
    ratio: f32,
) {
    if ratio <= 0.0 || ratio >= 0.25 {
        return;
    }
    let pulse = 0.75 + 0.25 * (gt * 14.0).sin().abs();
    let (w, h) = (24.0, 5.0);
    let x = if side == 0 { bx + bw - w } else { bx };
    let y = by - 6.4;
    let sp = |pts: Vec<Vec2>| -> Vec<Vec2> { pts.into_iter().map(|q| p(q.x, q.y)).collect() };
    let bg = sp(poly::slanted(x, y, w, h, 2.0));
    poly::solid(&bg, Color::new(0.04, 0.01, 0.02, 0.85));
    poly::grad_h(
        &bg,
        Color::new(0.9, 0.12, 0.15, 0.9 * pulse),
        Color::new(1.0, 0.45, 0.1, 0.9 * pulse),
    );
    poly::outline(&bg, 0.6 * s, Color::new(1.0, 0.9, 0.8, 0.95 * pulse));
    let label = crate::i18n::tr3("PERIGO", "DANGER", "PELIGRO");
    text(label, x + w / 2.0, y + 3.8, 4.0, WHITE, Align::Center, true);
}
