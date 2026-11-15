use super::energy_gem as gem;
use crate::gfx::*;
use crate::poly;

pub(super) const CELL_W: f32 = 26.0;
const GAP: f32 = 3.0;
const STRIPE_PITCH: f32 = 5.0;

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
    pub h: f32,
    pub power: f32,
    pub awaken: bool,
    pub form: u8,
    pub forms: usize,
    pub colors: [Color; 4],
}

pub(super) fn bar_width() -> f32 {
    CELL_W * 3.0 + GAP * 2.0
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

    pub(super) fn draw<P: Fn(f32, f32) -> Vec2>(&self, c: &Cell<P>) -> f32 {
        let side = c.side.min(1);
        let (s, gt, h) = (c.s, c.gt, c.h);
        let p = c.p;
        let lvl = (c.power / 1000.0).floor() as i32;
        let charging = self.rise[side];
        let flash = self.flash[side];
        let pw = bar_width();
        let pulse = 0.5 + 0.5 * (gt * 7.0).sin();
        let dir = if side == 0 { 1.0 } else { -1.0 };
        let sp = |v: Vec<Vec2>| -> Vec<Vec2> { v.into_iter().map(|q| p(q.x, q.y)).collect() };
        for i in 0..3 {
            let cx = if side == 0 {
                c.x + i as f32 * (CELL_W + GAP)
            } else {
                c.x + pw - (i as f32 + 1.0) * CELL_W - i as f32 * GAP
            };
            let col = c.colors[i.min(3)];
            let fr = if lvl >= 3 {
                1.0
            } else {
                (c.power / 1000.0 - i as f32).clamp(0.0, 1.0)
            };
            let full = fr >= 0.999;
            let cut = h * 0.42;
            if full || (flash > 0.0 && lvl as usize > i) {
                let k = if full { 0.12 + 0.1 * pulse } else { 0.0 } + flash * 0.35;
                poly::solid(
                    &sp(poly::chamfer(
                        cx - 2.0,
                        c.y - 2.0,
                        CELL_W + 4.0,
                        h + 4.0,
                        cut + 1.0,
                    )),
                    poly::with_a(col, k),
                );
            }
            poly::solid(
                &sp(poly::chamfer(
                    cx - 0.7,
                    c.y - 0.7,
                    CELL_W + 1.4,
                    h + 1.4,
                    cut + 0.6,
                )),
                Color::new(1.0, 1.0, 1.0, if full { 0.45 } else { 0.18 }),
            );
            poly::grad_v(
                &sp(poly::chamfer(cx, c.y, CELL_W, h, cut)),
                Color::new(0.0, 0.0, 0.03, 0.82),
                Color::new(0.03, 0.04, 0.09, 0.6),
            );
            let mid_x = cx + CELL_W * 0.5;
            poly::solid(&sp(poly::slanted(mid_x - 0.4, c.y, 0.8, h, 1.0)), Color::new(1.0, 1.0, 1.0, 0.22));
            if fr > 0.01 {
                let fw = CELL_W * fr;
                let fx = if side == 0 { cx } else { cx + CELL_W - fw };
                let body = sp(poly::chamfer(fx, c.y, fw, h, cut));
                poly::grad_v(&body, poly::shade(col, 1.5), poly::shade(col, 0.55));
                let speed = 6.0 + 22.0 * charging;
                let base = (gt * speed).rem_euclid(STRIPE_PITCH);
                let n = (fw / STRIPE_PITCH) as i32 + 3;
                let (lo, hi) = (fx, fx + fw);
                for k in 0..n {
                    let x0 = fx - STRIPE_PITCH + k as f32 * STRIPE_PITCH + dir * base;
                    let q = [
                        (x0, c.y + h),
                        (x0 + 2.2, c.y + h),
                        (x0 + 4.6, c.y),
                        (x0 + 2.4, c.y),
                    ];
                    let pts: Vec<Vec2> = q.iter().map(|&(x, y)| p(x.clamp(lo, hi), y)).collect();
                    let w = pts.iter().map(|v| v.x).fold(f32::MIN, f32::max)
                        - pts.iter().map(|v| v.x).fold(f32::MAX, f32::min);
                    if w > 0.4 * s {
                        poly::solid(&pts, Color::new(1.0, 1.0, 1.0, 0.13 + 0.1 * charging));
                    }
                }
                poly::grad_v(
                    &sp(poly::chamfer(
                        fx + 0.5,
                        c.y + 0.4,
                        (fw - 1.0).max(0.0),
                        h * 0.4,
                        1.0,
                    )),
                    Color::new(1.0, 1.0, 1.0, 0.6),
                    Color::new(1.0, 1.0, 1.0, 0.0),
                );
                poly::solid(
                    &sp(poly::chamfer(fx, c.y + h * 0.72, fw, h * 0.28, 0.8)),
                    Color::new(0.0, 0.0, 0.0, 0.22),
                );
                let sweep = ((gt * 0.55 + i as f32 * 0.21).rem_euclid(1.6) - 0.3) * CELL_W;
                if full || fr > 0.3 {
                    let gx = (cx + sweep).clamp(lo, hi);
                    let gw = (3.0f32).min(hi - gx);
                    if gw > 0.3 {
                        poly::solid(
                            &sp(poly::slanted(gx, c.y, gw, h, 1.6)),
                            Color::new(1.0, 1.0, 1.0, 0.32),
                        );
                    }
                }
                if !full {
                    let ex = if side == 0 { fx + fw } else { fx };
                    let a = 0.5 + 0.4 * pulse + 0.4 * charging;
                    poly::solid(
                        &sp(poly::chamfer(ex - 0.6, c.y - 0.5, 1.2, h + 1.0, 0.4)),
                        Color::new(1.0, 1.0, 1.0, a.min(1.0)),
                    );
                    poly::solid(
                        &sp(poly::chamfer(ex - 1.8, c.y - 0.5, 3.6, h + 1.0, 1.0)),
                        poly::with_a(poly::shade(col, 1.5), 0.25 * a.min(1.0)),
                    );
                    if charging > 0.25 {
                        for sk in 0..3 {
                            let ph = (gt * 2.3 + sk as f32 * 0.37 + i as f32 * 0.11).fract();
                            let sx =
                                ex + dir * (1.0 + 4.0 * ph) * (sk as f32 - 1.0).signum().max(0.4);
                            let sy = c.y + h * 0.5 - ph * (h + 2.0) * (0.6 + 0.2 * sk as f32);
                            poly::circle(
                                p(sx, sy),
                                0.55 * s,
                                Color::new(1.0, 1.0, 1.0, (1.0 - ph) * charging),
                            );
                        }
                    }
                }
            }
            for t in 1..4 {
                let tx = cx + CELL_W * t as f32 / 4.0;
                let a = p(tx, c.y + h * 0.15);
                let b = p(tx, c.y + h * 0.85);
                draw_line(a.x, a.y, b.x, b.y, 0.5 * s, Color::new(0.0, 0.0, 0.05, 0.4));
            }
            if flash > 0.0 && lvl as usize == i + 1 {
                poly::solid(
                    &sp(poly::chamfer(cx, c.y, CELL_W, h, cut)),
                    Color::new(1.0, 1.0, 1.0, 0.7 * flash),
                );
            }
        }
        gem::draw(c, lvl, flash, pulse)
    }
}
