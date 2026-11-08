use super::{Kind, Vfx};
use crate::gfx::*;
use crate::poly;

impl Vfx {
    pub(super) fn draw_cracks(&self, to_screen: &impl Fn(f32, f32) -> Vec2, scale: f32) {
        for p in self.parts.iter().filter(|p| p.k == Kind::Crack) {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let glow = (a * 3.0 - 2.0).max(0.0);
            let (sn, cs) = p.rot.sin_cos();
            let steps = 4;
            let mut prev = to_screen(p.x, p.y);
            for i in 1..=steps {
                let t = i as f32 / steps as f32;
                let jitter = ((p.rot * 31.0 + i as f32 * 12.9898).sin() * 43758.5).fract() - 0.5;
                let q = to_screen(
                    p.x + cs * p.size * t + sn * jitter * 5.0,
                    p.y + sn * p.size * t * 0.3 + jitter * 2.4,
                );
                let w = (1.6 - t) * scale * 1.5;
                let n = vec2(0.0, w);
                poly::solid(
                    &[prev - n, prev + n, q + n * 0.7, q - n * 0.7],
                    Color::new(0.06, 0.04, 0.05, 0.7 * a),
                );
                if glow > 0.0 {
                    let core = vec2(0.0, w * 0.35);
                    poly::solid(
                        &[prev - core, prev + core, q + core, q - core],
                        Color::new(p.col[0], p.col[1], p.col[2], glow * (1.0 - t * 0.6)),
                    );
                }
                prev = q;
            }
        }
    }
}
