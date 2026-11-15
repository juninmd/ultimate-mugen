use crate::gfx::*;
use crate::poly;

const HOLD: f32 = 0.45;
const SHAKE: f32 = 0.35;
const SPARKS: usize = 14;
const GRAVITY: f32 = 110.0;

#[derive(Clone, Copy, Default)]
struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

pub(super) struct LifeFx {
    trail: [f32; 2],
    last: [f32; 2],
    hold: [f32; 2],
    shake: [f32; 2],
    flash: [f32; 2],
    edge: [Option<(f32, f32)>; 2],
    sparks: Vec<Spark>,
    clock: f32,
    dt: f32,
    seed: u32,
}

impl Default for LifeFx {
    fn default() -> Self {
        Self {
            trail: [1.0; 2],
            last: [1.0; 2],
            hold: [0.0; 2],
            shake: [0.0; 2],
            flash: [0.0; 2],
            edge: [None; 2],
            sparks: Vec::new(),
            clock: f32::NAN,
            dt: 0.0,
            seed: 0x9e37_79b9,
        }
    }
}

pub(super) struct Bar {
    pub side: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub ratio: f32,
    pub low: bool,
}

impl LifeFx {
    fn tick(&mut self, gt: f32) {
        let dt = if self.clock.is_nan() {
            0.0
        } else {
            (gt - self.clock).clamp(0.0, 0.05)
        };
        self.clock = gt;
        for s in &mut self.sparks {
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.vy += GRAVITY * dt;
            s.life -= dt;
        }
        self.sparks.retain(|s| s.life > 0.0);
        self.dt = dt;
    }

    pub(super) fn track(&mut self, side: usize, ratio: f32, gt: f32, frozen: bool) {
        if side == 0 {
            self.tick(gt);
        }
        let dt = self.dt;
        if ratio < self.last[side] - 0.001 {
            self.hold[side] = HOLD;
            self.shake[side] = SHAKE;
            self.flash[side] = 0.6;
            self.burst(side);
        }
        self.last[side] = ratio;
        self.shake[side] = (self.shake[side] - dt * 1.4).max(0.0);
        self.flash[side] = (self.flash[side] - dt * 2.4).max(0.0);
        if ratio >= self.trail[side] {
            self.trail[side] = ratio;
            return;
        }
        if frozen {
            return;
        }
        self.hold[side] = (self.hold[side] - dt).max(0.0);
        if self.hold[side] <= 0.0 {
            let gap = self.trail[side] - ratio;
            self.trail[side] = (self.trail[side] - (gap * 3.2).max(0.08) * dt).max(ratio);
        }
    }

    fn rand(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed & 0xffff) as f32 / 65535.0
    }

    fn burst(&mut self, side: usize) {
        let Some((ex, ey)) = self.edge[side] else {
            return;
        };
        let dir = if side == 0 { -1.0 } else { 1.0 };
        for _ in 0..SPARKS {
            let (a, b, c) = (self.rand(), self.rand(), self.rand());
            self.sparks.push(Spark {
                x: ex,
                y: ey,
                vx: (a - 0.5) * 60.0 + dir * 16.0,
                vy: -b * 45.0 - 6.0,
                life: 0.35 + c * 0.3,
            });
        }
    }

    pub(super) fn draw<P: Fn(f32, f32) -> Vec2>(&mut self, p: &P, s: f32, gt: f32, b: Bar) {
        let side = b.side;
        let k = self.shake[side];
        let (x, y) = (
            b.x + (k * 70.0).sin() * k * 3.0,
            b.y + (k * 90.0).cos() * k * 0.9,
        );
        let (w, h) = (b.w, b.h);
        let sp = |pts: Vec<Vec2>| -> Vec<Vec2> { pts.into_iter().map(|q| p(q.x, q.y)).collect() };
        let cap = |x0: f32, ww: f32| sp(poly::chamfer(x0, y, ww, h, h * 0.42));
        let span = |a: f32, z: f32| -> (f32, f32) {
            let (a, z) = (a.clamp(0.0, 1.0), z.clamp(0.0, 1.0));
            if side == 0 {
                (x + w * (1.0 - z), w * (z - a))
            } else {
                (x + w * a, w * (z - a))
            }
        };
        poly::solid(
            &sp(poly::chamfer(x - 1.0, y - 1.0, w + 2.0, h + 2.0, h * 0.5)),
            Color::new(0.0, 0.0, 0.0, 0.45),
        );
        poly::grad_v(
            &cap(x, w),
            Color::new(0.01, 0.0, 0.03, 0.85),
            Color::new(0.16, 0.08, 0.2, 0.75),
        );
        let trail = self.trail[side];
        if trail > b.ratio + 0.002 {
            let (fx, fw) = span(b.ratio, trail);
            let hot = (self.hold[side] * 2.5).min(1.0);
            let (top, bot) = crate::tweaks::life_trail();
            poly::grad_v(
                &cap(fx, fw),
                poly::lerp_c(top, WHITE, hot * 0.7),
                poly::lerp_c(bot, WHITE, hot * 0.4),
            );
        }
        if b.ratio > 0.002 {
            let (fx, fw) = span(0.0, b.ratio);
            let (top, bot) = super::life_paint::fill_colors(b.ratio, b.low);
            poly::grad_v(&cap(fx, fw), top, bot);
            if b.low {
                let pulse = 0.5 + 0.5 * (gt * 9.0).sin();
                poly::solid(&cap(fx, fw), Color::new(1.0, 1.0, 1.0, 0.22 * pulse));
            }
            let inset = h * 0.18;
            poly::grad_v(
                &sp(poly::chamfer(
                    fx + inset,
                    y + 0.6,
                    (fw - inset * 2.0).max(0.0),
                    h * 0.45,
                    h * 0.2,
                )),
                Color::new(1.0, 1.0, 1.0, 0.42),
                Color::new(1.0, 1.0, 1.0, 0.0),
            );
            super::life_paint::shine(p, gt, side, fx, fw, y, h);
            let ex = if side == 0 { fx } else { fx + fw };
            self.edge[side] = Some((ex, y + h * 0.5));
            let glow = 7.0;
            let lo = (ex - glow).max(fx);
            let hi = (ex + glow).min(fx + fw);
            if hi > lo {
                let warm = Color::new(1.0, 0.98, 0.85, 0.75);
                let clear = Color::new(1.0, 0.85, 0.4, 0.0);
                let rect = |a: f32, z: f32| {
                    vec![
                        p(a, y + 0.5),
                        p(z, y + 0.5),
                        p(z, y + h - 0.5),
                        p(a, y + h - 0.5),
                    ]
                };
                poly::grad_h(&rect(lo, ex.clamp(lo, hi)), clear, warm);
                poly::grad_h(&rect(ex.clamp(lo, hi), hi), warm, clear);
            }
            if self.flash[side] > 0.0 {
                poly::solid(
                    &cap(x, w),
                    Color::new(1.0, 1.0, 1.0, self.flash[side] * 0.3),
                );
            }
        }
        for q in [0.25, 0.5, 0.75] {
            let tx = x + w * q;
            let (a, z) = (p(tx, y + h * 0.6), p(tx, y + h - 0.8));
            draw_line(a.x, a.y, z.x, z.y, 0.6 * s, Color::new(0.0, 0.0, 0.0, 0.35));
        }
        poly::outline(&cap(x, w), 0.55 * s, Color::new(1.0, 1.0, 1.0, 0.8));
        self.draw_sparks(p, s);
    }

    fn draw_sparks<P: Fn(f32, f32) -> Vec2>(&self, p: &P, s: f32) {
        for sp in &self.sparks {
            let a = (sp.life / 0.5).min(1.0);
            let (h, t) = (p(sp.x, sp.y), p(sp.x - sp.vx * 0.03, sp.y - sp.vy * 0.03));
            draw_line(
                h.x,
                h.y,
                t.x,
                t.y,
                0.7 * s,
                Color::new(1.0, 0.88, 0.55, 0.9 * a),
            );
        }
    }
}
