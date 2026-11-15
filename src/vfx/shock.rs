use super::{lerp, Vfx};
use crate::gfx::*;
use crate::poly;

const MIN_DISTORTING_RADIUS: f32 = 24.0;

impl Vfx {
    pub(super) fn draw_rings(&self, to_screen: &impl Fn(f32, f32) -> Vec2, scale: f32, distortion: f32) {
        for r in &self.rings {
            let t = r.t / r.dur;
            let e = 1.0 - (1.0 - t).powi(3);
            let c = to_screen(r.x, r.y);
            let radius = lerp(r.r0, r.r1, e) * scale;
            poly::ring(
                c,
                radius,
                (r.w * (1.0 - t) + 0.4) * scale,
                Color::new(r.col[0], r.col[1], r.col[2], (1.0 - t) * 0.9),
            );
            if distortion > 0.0 && r.r1 >= MIN_DISTORTING_RADIUS {
                let width = (r.w * 1.6 + 5.0) * scale;
                ume_shell::waves::shockwave(c.x, c.y, radius, width, (1.0 - t) * 0.8 * distortion);
            }
        }
    }
}
