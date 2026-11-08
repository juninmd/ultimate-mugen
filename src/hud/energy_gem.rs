use super::energy::{bar_width, Cell};
use crate::gfx::*;
use crate::poly;

pub(super) fn draw<P: Fn(f32, f32) -> Vec2>(c: &Cell<P>, lvl: i32, flash: f32, pulse: f32) -> f32 {
    let side = c.side.min(1);
    let (s, gt) = (c.s, c.gt);
    let h = c.h;
    let p = c.p;
    let pw = bar_width();
    let sp = |v: Vec<Vec2>| -> Vec<Vec2> { v.into_iter().map(|q| p(q.x, q.y)).collect() };
    let stages = (c.forms + 1).min(7);
    if c.forms > 0 {
        let sw = 6.2;
        for k in 0..stages {
            let px = if side == 0 {
                c.x + 3.0 + k as f32 * sw
            } else {
                c.x + pw - 3.0 - k as f32 * sw
            };
            let pc = p(px, c.y - 4.2);
            let r = 2.3 * s;
            let d = poly::regular(pc, r, 4, 0.0);
            if k as u8 <= c.form {
                let col = c.colors[k.min(3)];
                poly::solid(&d, poly::shade(col, 1.2));
                poly::outline(&d, 0.5 * s, Color::new(1.0, 1.0, 1.0, 0.9));
            } else if k as u8 == c.form + 1 {
                let a = if c.awaken { 0.6 + 0.4 * pulse } else { 0.5 };
                poly::solid(&d, Color::new(0.0, 0.0, 0.05, 0.7));
                poly::outline(&d, 0.7 * s, Color::new(1.0, 0.85, 0.3, a));
                if c.awaken {
                    poly::solid(
                        &poly::regular(pc, r * 0.55, 4, 0.0),
                        Color::new(1.0, 0.85, 0.3, a),
                    );
                }
            } else {
                poly::solid(&d, Color::new(0.0, 0.0, 0.05, 0.6));
                poly::outline(&d, 0.4 * s, Color::new(1.0, 1.0, 1.0, 0.3));
            }
        }
    }
    let gem_x = if side == 0 {
        c.x + pw + 11.0
    } else {
        c.x - 11.0
    };
    let gc = p(gem_x, c.y + h / 2.0);
    let bump = (1.0 + 0.06 * (lvl.min(3)) as f32 * pulse) + 0.25 * flash;
    let gcol = c.colors[(lvl.max(1) - 1).min(3) as usize];
    let r = 6.8 * s * bump;
    if lvl >= 1 {
        poly::circle(
            gc,
            r * (1.3 + 0.15 * pulse + flash),
            poly::with_a(gcol, 0.14 + 0.1 * flash),
        );
    }
    if c.awaken {
        let ph = (gt * 1.6).fract();
        poly::ring(
            gc,
            r * (1.2 + ph * 0.9),
            1.1 * s,
            Color::new(1.0, 0.85, 0.3, 0.9 * (1.0 - ph)),
        );
        poly::ring(
            gc,
            r * 1.22,
            1.0 * s,
            Color::new(1.0, 0.85, 0.3, 0.55 + 0.4 * pulse),
        );
    }
    poly::solid(
        &poly::regular(gc, r * 1.12, 6, 0.0),
        Color::new(0.0, 0.0, 0.06, 0.75),
    );
    let outer = poly::regular(gc, r, 6, 0.0);
    for k in 0..6 {
        let lit = [1.35, 1.05, 0.7, 0.55, 0.8, 1.15][k];
        poly::solid(
            &[gc, outer[k], outer[(k + 1) % 6]],
            poly::shade(gcol, if lvl >= 1 { lit } else { lit * 0.45 }),
        );
    }
    let inner = poly::regular(gc, r * 0.62, 6, 0.0);
    poly::grad_v(
        &inner,
        Color::new(0.0, 0.02, 0.1, 0.78),
        Color::new(0.0, 0.0, 0.05, 0.55),
    );
    poly::outline(&outer, 0.9 * s, Color::new(1.0, 1.0, 1.0, 0.85));
    poly::outline(&inner, 0.5 * s, Color::new(1.0, 1.0, 1.0, 0.3));
    if lvl >= 3 {
        let a = 0.1 + 0.08 * pulse;
        poly::solid(
            &sp(poly::chamfer(c.x - 2.0, c.y - 2.0, pw + 4.0, h + 4.0, 3.0)),
            Color::new(1.0, 0.3, 0.85, a),
        );
    }
    gem_x
}
