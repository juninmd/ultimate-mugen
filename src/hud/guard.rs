use crate::gfx::*;
use crate::poly;
use crate::ui::{hex, Align};
use ume_engine::fighter::Fighter;

fn rect<P: Fn(f32, f32) -> Vec2>(p: &P, s: f32, x: f32, y: f32, w: f32, h: f32) -> [Vec2; 4] {
    let a = p(x, y);
    [
        a,
        a + vec2(w * s, 0.0),
        a + vec2(w * s, h * s),
        a + vec2(0.0, h * s),
    ]
}

pub(super) fn draw<
    P: Fn(f32, f32) -> Vec2,
    T: Fn(&str, f32, f32, f32, Color, Align, bool),
>(
    p: &P,
    text: &T,
    s: f32,
    gt: f32,
    f: &Fighter,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
    side: usize,
) {
    if f.ikemen.guard_max <= 0.0 {
        return;
    }
    let ratio = (f.ikemen.guard_points / f.ikemen.guard_max).clamp(0.0, 1.0);
    let inset = bh * 0.25;
    let (gw, gh) = (bw - inset * 2.0, 1.6);
    let gx = bx + inset;
    let gy = by + bh + 1.2;

    let bg = rect(p, s, gx, gy, gw, gh);
    poly::solid(&bg, Color::new(0.03, 0.05, 0.1, 0.7));

    let filled_w = gw * ratio;
    if filled_w > 0.5 {
        let fill_x = if side == 0 { gx + gw - filled_w } else { gx };
        let fill = rect(p, s, fill_x, gy, filled_w, gh);
        if ratio > 0.6 {
            poly::grad_h(&fill, hex("#38bdf8"), hex("#0284c7"));
        } else if ratio > 0.3 {
            poly::grad_h(&fill, hex("#fbbf24"), hex("#d97706"));
        } else {
            let strobe = 0.7 + 0.3 * (gt * 16.0).sin().abs();
            poly::solid(&fill, Color::new(0.95 * strobe, 0.2, 0.2, 0.95));
        }
    }

    let border_col = if ratio < 0.3 {
        Color::new(1.0, 0.3, 0.3, 0.8)
    } else {
        Color::new(0.4, 0.6, 0.8, 0.45)
    };
    poly::outline(&bg, 0.45 * s, border_col);

    if f.ikemen.guard_break || f.state_no == 5400 {
        let pulse = 0.8 + 0.2 * (gt * 18.0).sin().abs();
        let (cw, ch) = (48.0, 7.5);
        let cx = if side == 0 { bx + bw - cw - 4.0 } else { bx + 4.0 };
        let cy = by + bh + 4.5;
        let c_bg = poly::slanted(cx, cy, cw, ch, 2.5);
        let sp_c: Vec<Vec2> = c_bg.into_iter().map(|q| p(q.x, q.y)).collect();
        poly::solid(&sp_c, Color::new(0.05, 0.08, 0.16, 0.92));
        poly::grad_h(
            &sp_c,
            Color::new(0.08, 0.75, 1.0, 0.9 * pulse),
            Color::new(0.2, 0.95, 1.0, 0.9 * pulse),
        );
        poly::outline(&sp_c, 0.75 * s, Color::new(1.0, 1.0, 1.0, 0.95 * pulse));
        let label = crate::i18n::tr3("QUEBRA DE GUARDA", "GUARD CRUSH", "GUARD CRUSH");
        text(label, cx + cw / 2.0, cy + 5.5, 4.5, WHITE, Align::Center, true);
    }
}
