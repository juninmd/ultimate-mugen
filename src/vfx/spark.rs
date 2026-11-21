use super::{Vfx, P};
use crate::gfx::*;
use crate::poly;

impl Vfx {
    pub(super) fn draw_spark(&self, particle: &P, center: Vec2, scale: f32) {
        let alpha = (particle.life / particle.max).clamp(0.0, 1.0);
        let direction = vec2(-particle.vx, -particle.vy).normalize_or_zero();
        let rotation = direction.y.atan2(direction.x);
        let size = (particle.size * scale * 7.0).max(3.0);
        if self.sheets.draw_spark(
            center,
            size,
            rotation,
            1.0 - alpha,
            Color::new(particle.col[0], particle.col[1], particle.col[2], alpha),
        ) {
            return;
        }
        let tail = center
            + direction
                * (particle.vx * particle.vx + particle.vy * particle.vy).sqrt()
                * 2.8
                * scale;
        let normal = vec2(-direction.y, direction.x) * (particle.size * scale).max(1.0) * 0.9;
        let head = Color::new(1.0, 1.0, 0.92, alpha);
        let fade = Color::new(particle.col[0], particle.col[1], particle.col[2], 0.0);
        poly::fill(
            &[
                center - direction * (particle.size * scale * 0.6),
                center + normal,
                tail,
                center - normal,
            ],
            |point| {
                if (point - tail).length() < 0.001 {
                    fade
                } else if (point - center).length() < 0.001 {
                    head
                } else {
                    Color::new(particle.col[0], particle.col[1], particle.col[2], alpha)
                }
            },
        );
    }
}
