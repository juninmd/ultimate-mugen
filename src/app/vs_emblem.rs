use super::*;
use crate::poly;
use std::f32::consts::TAU;

fn hash(n: u32) -> f32 {
    super::vs_epic::hash(n)
}

fn with(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

impl App {
    pub(super) fn vs_emblem(&self, cx: f32, cy: f32, ek: f32, since: f32, cols: [Color; 2]) {
        if ek <= 0.0 {
            return;
        }
        let t = self.ui.t;
        let s = self.ui.s;
        let sc = 1.15 * ek * (1.0 + 0.03 * (t * 6.0).sin());
        let a = ek.min(1.0);
        let c = self.sp(cx, cy);
        glow(
            c.x,
            c.y,
            320.0 * sc * s,
            Color::new(1.0, 0.45, 0.1, 0.65 * a),
        );
        glow(c.x - 60.0 * s, c.y, 200.0 * sc * s, with(cols[0], 0.5 * a));
        glow(c.x + 60.0 * s, c.y, 200.0 * sc * s, with(cols[1], 0.5 * a));
        for k in 0..3 {
            let ph = ((since * 0.8 + k as f32 / 3.0) % 1.0).max(0.0);
            poly::ring(
                c,
                (110.0 + ph * 220.0) * sc * s,
                (5.0 * (1.0 - ph) + 0.5) * s,
                Color::new(1.0, 0.85, 0.5, 0.35 * (1.0 - ph) * a),
            );
        }
        for k in 0..12u32 {
            let ang = k as f32 / 12.0 * TAU + t * 0.6;
            let len = (150.0 + hash(k + (t * 10.0) as u32) * 90.0) * sc;
            let half = 0.035;
            let p = |r: f32, da: f32| c + vec2((ang + da).cos(), (ang + da).sin()) * r * s;
            poly::fill(
                &[p(100.0 * sc, -half), p(100.0 * sc, half), p(len, 0.0)],
                |q| {
                    if q == p(len, 0.0) {
                        Color::new(1.0, 0.9, 0.5, 0.0)
                    } else {
                        Color::new(1.0, 0.85, 0.4, 0.9 * a)
                    }
                },
            );
        }
        let ticks = 48;
        for k in 0..ticks {
            let ang = k as f32 / ticks as f32 * TAU - t * 0.4;
            let (r0, r1) = if k % 4 == 0 {
                (118.0, 134.0)
            } else {
                (124.0, 132.0)
            };
            let p0 = c + vec2(ang.cos(), ang.sin()) * r0 * sc * s;
            let p1 = c + vec2(ang.cos(), ang.sin()) * r1 * sc * s;
            draw_line(
                p0.x,
                p0.y,
                p1.x,
                p1.y,
                2.0 * s,
                Color::new(1.0, 0.85, 0.45, 0.8 * a),
            );
        }
        let dia = poly::regular(c, 104.0 * sc * s, 4, 0.0);
        poly::grad_v(
            &dia,
            Color::new(1.0, 0.92, 0.6, a),
            Color::new(0.55, 0.18, 0.05, a),
        );
        let inner = poly::regular(c, 92.0 * sc * s, 4, 0.0);
        poly::grad_v(
            &inner,
            Color::new(0.22, 0.04, 0.1, 0.95 * a),
            Color::new(0.05, 0.0, 0.04, 0.95 * a),
        );
        poly::outline(&inner, 2.0 * s, Color::new(1.0, 0.8, 0.4, 0.8 * a));
        let disc = poly::regular(c, 86.0 * sc * s, poly::facets(86.0 * s), 0.0);
        poly::grad_v(
            &disc,
            Color::new(0.35, 0.06, 0.12, 0.9 * a),
            Color::new(0.08, 0.0, 0.05, 0.9 * a),
        );
        for (i, col) in cols.iter().enumerate() {
            let base = t * 1.6 * if i == 0 { 1.0 } else { -1.0 } + i as f32 * 3.1;
            self.arc(
                c,
                122.0 * sc * s,
                7.0 * s,
                base,
                base + 2.2,
                with(*col, 0.95 * a),
            );
            self.arc(
                c,
                122.0 * sc * s,
                16.0 * s,
                base,
                base + 2.2,
                with(*col, 0.25 * a),
            );
        }
        let shake = if since < 0.25 {
            (since * 90.0).sin() * (0.25 - since) * 20.0
        } else {
            0.0
        };
        let (x, y, sz) = (cx + shake, cy + 44.0 * sc, 128.0 * sc);
        self.ui.text(
            "VS",
            x + 6.0,
            y + 6.0,
            sz,
            Color::new(0.0, 0.0, 0.0, 0.7 * a),
            Align::Center,
            true,
        );
        self.ui.text(
            "VS",
            x + 3.0,
            y + 3.0,
            sz,
            Color::new(0.75, 0.05, 0.12, a),
            Align::Center,
            true,
        );
        self.ui.text_bold(
            "VS",
            x,
            y,
            sz,
            Color::new(1.0, 0.86, 0.35, a),
            Color::new(0.35, 0.02, 0.06, a),
            Align::Center,
        );
        self.ui.text(
            "VS",
            x - 1.5,
            y - 2.5,
            sz,
            Color::new(1.0, 1.0, 0.9, 0.35 * a),
            Align::Center,
            true,
        );
    }

    fn arc(&self, c: Vec2, r: f32, th: f32, a0: f32, a1: f32, col: Color) {
        let n = 24;
        for k in 0..n {
            let (u, v) = (
                a0 + (a1 - a0) * k as f32 / n as f32,
                a0 + (a1 - a0) * (k + 1) as f32 / n as f32,
            );
            let fade = (k as f32 / n as f32).min(1.0);
            let cc = Color::new(col.r, col.g, col.b, col.a * fade);
            let d = |a: f32, rr: f32| c + vec2(a.cos(), a.sin()) * rr;
            poly::solid(
                &[
                    d(u, r - th / 2.0),
                    d(u, r + th / 2.0),
                    d(v, r + th / 2.0),
                    d(v, r - th / 2.0),
                ],
                cc,
            );
        }
    }
}
