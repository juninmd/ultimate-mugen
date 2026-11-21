use crate::gfx::*;
use crate::poly;

const SEGMENTS: usize = 3;
const GAP: f32 = 2.2;
const STRIPE_PITCH: f32 = 4.6;

pub(super) struct EnergyFx {
    prev: [f32; 2],
    rise: [f32; 2],
    flash: [f32; 2],
    lvl: [i32; 2],
}

pub(super) struct Cell<'a, P: Fn(f32, f32) -> Vec2> {
    pub p: &'a P,
    pub s: f32,
    pub gt: f32,
    pub side: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub power: f32,
    pub colors: [Color; 4],
}

impl Default for EnergyFx {
    fn default() -> Self {
        Self {
            prev: [-1.0; 2],
            rise: [0.0; 2],
            flash: [0.0; 2],
            lvl: [0; 2],
        }
    }
}

impl EnergyFx {
    pub(super) fn track(&mut self, side: usize, power: f32) {
        let side = side.min(1);
        let up = if power > self.prev[side] + 0.05 {
            1.0
        } else {
            0.0
        };
        self.rise[side] += (up - self.rise[side]) * 0.12;
        let lvl = (power / 1000.0).floor() as i32;
        if self.prev[side] >= 0.0 && lvl > self.lvl[side] {
            self.flash[side] = 1.0;
        }
        self.flash[side] = (self.flash[side] - 0.03).max(0.0);
        self.lvl[side] = lvl;
        self.prev[side] = power;
    }

    pub(super) fn draw<P: Fn(f32, f32) -> Vec2>(&self, c: &Cell<P>) {
        let side = c.side.min(1);
        let (s, gt, h) = (c.s, c.gt, c.h);
        let p = c.p;
        let lvl = (c.power / 1000.0).floor() as i32;
        let charging = self.rise[side];
        let flash = self.flash[side];
        let pulse = 0.5 + 0.5 * (gt * 7.0).sin();
        let dir = if side == 0 { 1.0 } else { -1.0 };
        let sp = |v: Vec<Vec2>| -> Vec<Vec2> { v.into_iter().map(|q| p(q.x, q.y)).collect() };
        let seg = ((c.w - GAP * (SEGMENTS as f32 - 1.0)) / SEGMENTS as f32).max(4.0);
        let cut = h * 0.5;
        poly::solid(
            &sp(poly::chamfer(c.x - 0.7, c.y - 0.7, c.w + 1.4, h + 1.4, cut + 0.35)),
            Color::new(1.0, 1.0, 1.0, 0.2),
        );
        poly::grad_v(
            &sp(poly::chamfer(c.x, c.y, c.w, h, cut)),
            Color::new(0.0, 0.0, 0.03, 0.86),
            Color::new(0.04, 0.05, 0.11, 0.6),
        );
        for i in 0..SEGMENTS {
            let sx = if side == 0 {
                c.x + i as f32 * (seg + GAP)
            } else {
                c.x + c.w - (i as f32 + 1.0) * seg - i as f32 * GAP
            };
            let col = c.colors[i.min(3)];
            let fr = if lvl >= SEGMENTS as i32 {
                1.0
            } else {
                (c.power / 1000.0 - i as f32).clamp(0.0, 1.0)
            };
            let full = fr >= 0.999;
            poly::grad_v(
                &sp(poly::chamfer(sx, c.y, seg, h, cut)),
                Color::new(0.02, 0.03, 0.08, 0.9),
                Color::new(0.04, 0.05, 0.1, 0.82),
            );
            if full {
                poly::solid(
                    &sp(poly::chamfer(sx - 0.9, c.y - 0.9, seg + 1.8, h + 1.8, cut + 0.45)),
                    poly::with_a(col, 0.1 + 0.08 * pulse + 0.3 * flash),
                );
            }
            if fr > 0.01 {
                let fw = (seg * fr).max(0.5);
                let fx = if side == 0 { sx } else { sx + seg - fw };
                let body = sp(poly::chamfer(fx, c.y, fw, h, cut * (fw / seg).min(1.0)));
                poly::grad_v(&body, poly::shade(col, 1.6), poly::shade(col, 0.5));
                poly::grad_v(
                    &sp(poly::chamfer(
                        fx + 0.5,
                        c.y + 0.35,
                        (fw - 1.0).max(0.0),
                        h * 0.38,
                        0.7,
                    )),
                    Color::new(1.0, 1.0, 1.0, 0.55),
                    Color::new(1.0, 1.0, 1.0, 0.0),
                );
                let speed = 7.0 + 26.0 * charging;
                let base = (gt * speed).rem_euclid(STRIPE_PITCH);
                let n = (fw / STRIPE_PITCH) as i32 + 2;
                let (lo, hi) = (fx, fx + fw);
                for k in 0..n {
                    let x0 = fx - STRIPE_PITCH + k as f32 * STRIPE_PITCH + dir * base;
                    let q = [
                        (x0, c.y + h),
                        (x0 + 1.8, c.y + h),
                        (x0 + 3.6, c.y),
                        (x0 + 1.8, c.y),
                    ];
                    let pts: Vec<Vec2> = q.iter().map(|&(x, y)| p(x.clamp(lo, hi), y)).collect();
                    let w = pts.iter().map(|v| v.x).fold(f32::MIN, f32::max)
                        - pts.iter().map(|v| v.x).fold(f32::MAX, f32::min);
                    if w > 0.3 * s {
                        poly::solid(&pts, Color::new(1.0, 1.0, 1.0, 0.12 + 0.1 * charging));
                    }
                }
                if !full {
                    let ex = if side == 0 { fx + fw } else { fx };
                    let a = (0.5 + 0.4 * pulse + 0.4 * charging).min(1.0);
                    poly::solid(
                        &sp(poly::chamfer(ex - 0.5, c.y - 0.5, 1.0, h + 1.0, 0.3)),
                        Color::new(1.0, 1.0, 1.0, a),
                    );
                    if charging > 0.25 {
                        for sk in 0..3 {
                            let ph = (gt * 2.3 + sk as f32 * 0.37 + i as f32 * 0.11).fract();
                            let r = (sk as f32 - 1.0).signum().max(0.4);
                            let sx2 = ex + dir * (0.8 + 3.4 * ph) * r;
                            let sy = c.y + h * 0.5 - ph * (h + 1.6) * (0.6 + 0.2 * sk as f32);
                            poly::circle(
                                p(sx2, sy),
                                0.45 * s,
                                Color::new(1.0, 1.0, 1.0, (1.0 - ph) * charging),
                            );
                        }
                    }
                }
            }
            for t in 1..4 {
                let tx = sx + seg * t as f32 / 4.0;
                let a = p(tx, c.y + 0.25);
                let b = p(tx, c.y + h - 0.25);
                draw_line(a.x, a.y, b.x, b.y, 0.45 * s, Color::new(0.0, 0.0, 0.05, 0.45));
            }
            if flash > 0.0 && lvl as usize == i + 1 {
                poly::solid(
                    &sp(poly::chamfer(sx, c.y, seg, h, cut)),
                    Color::new(1.0, 1.0, 1.0, 0.7 * flash),
                );
            }
        }
    }
}
