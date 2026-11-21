use super::*;
use crate::poly;
use std::f32::consts::TAU;

pub(super) fn hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9e37_79b9) ^ 0x85eb_ca6b;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    (x & 0xffff) as f32 / 65535.0
}

fn with(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

pub(super) struct Side {
    pub col: Color,
    pub off: f32,
}

impl App {
    pub(super) fn sp(&self, x: f32, y: f32) -> Vec2 {
        vec2(self.ui.x(x), self.ui.y(y))
    }

    fn quad_v(&self, q: [Vec2; 4], top: Color, bot: Color) {
        let pts: Vec<Vec2> = q.iter().map(|p| self.sp(p.x, p.y)).collect();
        poly::grad_v(&pts, top, bot);
    }

    fn quad_h(&self, q: [Vec2; 4], l: Color, r: Color) {
        let pts: Vec<Vec2> = q.iter().map(|p| self.sp(p.x, p.y)).collect();
        poly::grad_h(&pts, l, r);
    }

    pub(super) fn vs_backdrop(&self, sides: [Side; 2], split: f32, lean: f32) {
        clear_background(hex("#05030c"));
        let base_alpha = if crate::screen_art::ScreenArt::Versus.draw(&self.ui, 1.0) { 0.5 } else { 1.0 };
        let t = self.ui.t;
        for (i, s) in sides.iter().enumerate() {
            let left = i == 0;
            let deep = poly::shade(s.col, 0.18);
            let rich = poly::shade(s.col, 0.75);
            let (a, b) = if left {
                (s.off - 40.0, s.off + split)
            } else {
                (s.off + split, s.off + VW + 40.0)
            };
            let q = if left {
                [
                    vec2(a, 0.0),
                    vec2(b + lean / 2.0, 0.0),
                    vec2(b - lean / 2.0, VH),
                    vec2(a, VH),
                ]
            } else {
                [
                    vec2(a + lean / 2.0, 0.0),
                    vec2(b, 0.0),
                    vec2(b, VH),
                    vec2(a - lean / 2.0, VH),
                ]
            };
            if left {
                self.quad_h(q, with(deep, base_alpha), with(rich, base_alpha));
            } else {
                self.quad_h(q, with(rich, base_alpha), with(deep, base_alpha));
            }
            let cx = if left { 330.0 + s.off } else { 950.0 + s.off };
            let spin = t * if left { 0.12 } else { -0.12 };
            let c = self.sp(cx, 380.0);
            for k in 0..18 {
                let a0 = spin + k as f32 / 18.0 * TAU;
                let a1 = a0 + TAU / 36.0;
                let r = 1100.0 * self.ui.s;
                let pts = [
                    c,
                    c + vec2(a0.cos(), a0.sin()) * r,
                    c + vec2(a1.cos(), a1.sin()) * r,
                ];
                poly::fill(&pts, |p| {
                    if p == c {
                        with(s.col, 0.16)
                    } else {
                        with(s.col, 0.0)
                    }
                });
            }
            glow(c.x, c.y, 380.0 * self.ui.s, with(s.col, 0.55));
            glow(
                c.x,
                self.ui.y(600.0),
                300.0 * self.ui.s,
                Color::new(1.0, 0.9, 0.7, 0.12),
            );
            for row in 0..14 {
                for col in 0..10 {
                    let fx = col as f32 / 9.0;
                    let x = if left {
                        s.off + fx * 520.0
                    } else {
                        s.off + VW - fx * 520.0
                    };
                    let y = 30.0 + row as f32 * 50.0 + (col % 2) as f32 * 25.0;
                    let r = (1.0 - fx) * 7.0 + 1.0;
                    poly::circle(
                        self.sp(x, y),
                        r * self.ui.s,
                        Color::new(0.0, 0.0, 0.0, 0.22),
                    );
                }
            }
            for k in 0..5 {
                let ph = (t * 0.35 + hash(k + i as u32 * 31)) % 1.0;
                let x = if left {
                    split - 60.0 - ph * 900.0
                } else {
                    split + 60.0 + ph * 900.0
                } + s.off;
                let w = 8.0 + hash(k * 7) * 30.0;
                let a = 0.10 * (1.0 - ph) * ph * 4.0;
                self.quad_v(
                    [
                        vec2(x, 0.0),
                        vec2(x + w, 0.0),
                        vec2(x + w - lean, VH),
                        vec2(x - lean, VH),
                    ],
                    Color::new(1.0, 1.0, 1.0, a),
                    with(s.col, a * 0.4),
                );
            }
            for k in 0..26u32 {
                let seed = k + i as u32 * 97;
                let life = (t * (0.18 + hash(seed) * 0.2) + hash(seed + 5)) % 1.0;
                let x0 = if left {
                    30.0 + hash(seed + 1) * 560.0
                } else {
                    690.0 + hash(seed + 1) * 560.0
                };
                let x = x0 + s.off + (life * 6.0 + hash(seed) * 9.0).sin() * 14.0;
                let y = VH + 20.0 - life * (VH + 60.0);
                let a = (life * (1.0 - life) * 4.0).min(1.0);
                let p = self.sp(x, y);
                poly::circle(p, 4.0 * self.ui.s, with(s.col, 0.35 * a));
                poly::circle(
                    p,
                    (1.2 + hash(seed + 3) * 1.8) * self.ui.s,
                    Color::new(1.0, 0.95, 0.8, 0.85 * a),
                );
            }
        }
        self.quad_v(
            [
                vec2(0.0, VH - 200.0),
                vec2(VW, VH - 200.0),
                vec2(VW, VH),
                vec2(0.0, VH),
            ],
            Color::new(0.0, 0.0, 0.0, 0.0),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
    }

    pub(super) fn vs_seam(&self, split: f32, lean: f32) {
        let t = self.ui.t;
        let frame = (t * 18.0) as u32;
        let n = 16;
        let pt = |k: usize| -> Vec2 {
            let f = k as f32 / n as f32;
            let jit = if k == 0 || k == n {
                0.0
            } else {
                (hash(frame * 31 + k as u32) - 0.5) * 34.0
            };
            vec2(split + lean / 2.0 - lean * f + jit, VH * f)
        };
        for (w, c) in [
            (14.0, Color::new(1.0, 0.6, 0.2, 0.18)),
            (6.0, Color::new(1.0, 0.85, 0.5, 0.6)),
            (2.2, Color::new(1.0, 1.0, 0.95, 1.0)),
        ] {
            for k in 0..n {
                let (a, b) = (pt(k), pt(k + 1));
                let (a, b) = (self.sp(a.x, a.y), self.sp(b.x, b.y));
                draw_line(a.x, a.y, b.x, b.y, w * self.ui.s, c);
            }
        }
        for k in (2..n).step_by(5) {
            let p = pt(k);
            let p = self.sp(p.x, p.y);
            glow(p.x, p.y, 40.0 * self.ui.s, Color::new(1.0, 0.8, 0.4, 0.35));
        }
    }
}
