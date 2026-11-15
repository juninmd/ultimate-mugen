use crate::gfx::*;
use crate::poly;

pub(super) fn shine<P: Fn(f32, f32) -> Vec2>(
    p: &P,
    gt: f32,
    side: usize,
    fx: f32,
    fw: f32,
    y: f32,
    h: f32,
) {
    let phase = ((gt * 0.35 + side as f32 * 0.5) % 1.4) - 0.2;
    let t = if side == 0 { 1.0 - phase } else { phase };
    let cx = fx + fw * t;
    let half = 12.0;
    let (a, z) = ((cx - half).max(fx), (cx + half).min(fx + fw));
    if z <= a {
        return;
    }
    let m = cx.clamp(a, z);
    let lit = Color::new(1.0, 1.0, 1.0, 0.32);
    let off = Color::new(1.0, 1.0, 1.0, 0.0);
    let rect = |l: f32, r: f32| vec![p(l, y), p(r, y), p(r, y + h), p(l, y + h)];
    poly::grad_h(&rect(a, m), off, lit);
    poly::grad_h(&rect(m, z), lit, off);
}

pub(super) fn fill_colors(ratio: f32, low: bool) -> (Color, Color) {
    let (top, bot) = crate::tweaks::life_fill(false);
    let (ltop, lbot) = crate::tweaks::life_fill(true);
    if low {
        return (ltop, lbot);
    }
    let t = ((0.5 - ratio) / 0.25).clamp(0.0, 1.0);
    let warm = Color::new(1.0, 0.55, 0.12, 1.0);
    if t <= 0.0 {
        return (top, bot);
    }
    (
        poly::lerp_c(top, poly::lerp_c(warm, ltop, 0.3), t),
        poly::lerp_c(bot, poly::lerp_c(warm, lbot, 0.5), t),
    )
}
