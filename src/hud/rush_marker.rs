use crate::gfx::*;
use crate::poly;
use crate::ui::Align;

pub(super) fn draw<P: Fn(f32, f32) -> Vec2, T: Fn(&str, f32, f32, f32, Color, Align, bool)>(
    p: &P,
    text: &T,
    s: f32,
    gt: f32,
    window: Option<f32>,
    frame: (f32, f32, f32, f32),
    side: usize,
) {
    let Some(left) = window else {
        return;
    };
    let (bx, by, bw, bh) = frame;
    let pulse = 0.8 + 0.2 * (gt * 14.0).sin().abs();
    let (cw, ch) = (52.0, 7.5);
    let cx = if side == 0 {
        bx + 4.0
    } else {
        bx + bw - cw - 4.0
    };
    let cy = by - 7.5;
    let has_art = super::badge_art::draw_rush_plate(p, s, cx, cy, cw, ch, pulse, side == 1);
    if !has_art {
        let shape: Vec<Vec2> = poly::slanted(cx, cy, cw, ch, 2.5)
            .into_iter()
            .map(|q| p(q.x, q.y))
            .collect();
        poly::solid(&shape, Color::new(0.05, 0.08, 0.16, 0.92));
        poly::outline(&shape, 0.7 * s, Color::new(0.4, 0.9, 1.0, 0.95 * pulse));
    }
    let bar_w = (cw - 6.0) * left;
    let bar = [
        p(cx + 3.0, cy + ch - 1.6),
        p(cx + 3.0 + bar_w, cy + ch - 1.6),
        p(cx + 3.0 + bar_w, cy + ch - 0.6),
        p(cx + 3.0, cy + ch - 0.6),
    ];
    poly::solid(&bar, Color::new(0.35, 0.85, 1.0, 0.95 * pulse));
    let label = crate::i18n::tr3("RUSH CANCEL", "RUSH CANCEL", "RUSH CANCEL");
    text(
        label,
        cx + cw / 2.0,
        cy + 4.6,
        4.3,
        Color::new(1.0, 1.0, 1.0, 1.0),
        Align::Center,
        true,
    );
}
