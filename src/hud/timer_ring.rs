use crate::gfx::*;
use crate::poly;

pub(super) struct Ring {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub frac: f32,
    pub urgent: bool,
}

fn arc(c: Vec2, r: f32, th: f32, a0: f32, a1: f32, col: Color) {
    let steps = ((poly::facets(r) as f32) * ((a1 - a0).abs() / std::f32::consts::TAU))
        .ceil()
        .max(2.0) as usize;
    for k in 0..steps {
        let (t0, t1) = (
            a0 + (a1 - a0) * k as f32 / steps as f32,
            a0 + (a1 - a0) * (k + 1) as f32 / steps as f32,
        );
        let (u0, u1) = (vec2(t0.cos(), t0.sin()), vec2(t1.cos(), t1.sin()));
        let (ro, ri) = (r + th / 2.0, (r - th / 2.0).max(0.0));
        poly::solid(&[c + u0 * ro, c + u1 * ro, c + u1 * ri, c + u0 * ri], col);
    }
}

pub(super) fn draw<P: Fn(f32, f32) -> Vec2>(p: &P, s: f32, gt: f32, ring: Ring) {
    let c = p(ring.cx, ring.cy);
    let r = ring.r * s;
    if !super::art::draw_timer_crest(c, r) {
        poly::circle(c, r + 3.2 * s, Color::new(0.03, 0.02, 0.08, 0.55));
        poly::ring(c, r + 3.2 * s, 0.35 * s, Color::new(1.0, 1.0, 1.0, 0.35));
    }
    poly::ring(c, r, 1.4 * s, Color::new(1.0, 1.0, 1.0, 0.12));
    let frac = ring.frac.clamp(0.0, 1.0);
    if frac <= 0.0 {
        return;
    }
    let a0 = -std::f32::consts::FRAC_PI_2;
    let a1 = a0 + std::f32::consts::TAU * frac;
    let (core, glow) = if ring.urgent {
        let pulse = 0.6 + 0.4 * (gt * 9.0).sin().abs();
        (
            Color::new(1.0, 0.22, 0.3, 1.0),
            Color::new(1.0, 0.15, 0.25, 0.28 * pulse),
        )
    } else {
        (
            Color::new(1.0, 0.78, 0.22, 1.0),
            Color::new(1.0, 0.7, 0.15, 0.22),
        )
    };
    arc(c, r, 3.4 * s, a0, a1, glow);
    arc(c, r, 1.4 * s, a0, a1, core);
    let tip = c + vec2(a1.cos(), a1.sin()) * r;
    poly::circle(tip, 1.1 * s, Color::new(1.0, 1.0, 0.92, 0.95));
}
