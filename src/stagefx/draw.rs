use super::*;

impl StageReact {
    /// Animated content attached to back layer `i` (called right after the baked layer is drawn).
    pub fn draw_layer(&self, i: usize, cx: &Ctx, g: &Gfx, m: &Match) {
        match self.kind {
            StageKind::Dojo => self.dojo_layer(i, cx, g),
            StageKind::Neon => self.neon_layer(i, cx, g),
            StageKind::Bamboo => self.bamboo_layer(i, cx, g),
            StageKind::Plains => self.plains_layer(i, cx, g),
            StageKind::Spirit => self.spirit_layer(i, cx, g),
            StageKind::Volcano => self.volcano_layer(i, cx, g),
            StageKind::Custom => {}
        }
        if i == 4 {
            self.floor_common(cx, g, m);
        }
    }
    /// Foreground pass (after the fighters): front layers' animated parts, debris, fog, vignette.
    pub fn draw_front(&self, cx: &Ctx, g: &Gfx) {
        match self.kind {
            StageKind::Dojo => self.dojo_front(cx, g),
            StageKind::Neon => self.neon_front(cx, g),
            StageKind::Bamboo => self.bamboo_front(cx, g),
            StageKind::Plains => self.plains_front(cx, g),
            StageKind::Spirit => self.spirit_front(cx, g),
            StageKind::Volcano => self.volcano_front(cx, g),
            StageKind::Custom => {}
        }
        self.draw_bits(cx, g);
    }
    /// Per-stage vignette and colour grade, applied through the post-processing parameters (no extra geometry): darker, tinted
    /// edges per stage, pulsing with the flicker of lights.
    pub fn grade(&self, p: &mut crate::vfx::FxParams, gain: f32) {
        if !self.active() {
            return;
        }
        let (v, tint) = match self.kind {
            StageKind::Dojo => (0.14, [1.03, 0.99, 0.95]),
            StageKind::Neon => (0.16, [0.99, 0.97, 1.04]),
            StageKind::Bamboo => (0.16, [0.97, 1.0, 1.02]),
            StageKind::Plains => (0.05, [1.03, 1.01, 0.97]),
            StageKind::Spirit => (0.15, [0.97, 1.02, 1.02]),
            StageKind::Volcano => (0.2, [1.05, 0.98, 0.94]),
            StageKind::Custom => (0.0, [1.0; 3]),
        };
        p.vignette += v + self.flicker * 0.07 * gain.min(1.0);
        for k in 0..3 {
            p.tint[k] *= tint[k];
        }
    }
    pub(super) fn draw_bits(&self, cx: &Ctx, g: &Gfx) {
        let s = cx.s;
        for b in self.bits.iter().filter(|b| {
            matches!(
                b.k,
                BitKind::Dust | BitKind::Shard | BitKind::Leaf | BitKind::Drop
            )
        }) {
            let a = (b.life / b.max).clamp(0.0, 1.0);
            let c = cx.w(b.x, b.y);
            match b.k {
                BitKind::Dust => fan(
                    c,
                    b.size * s,
                    b.size * s * 0.85,
                    12,
                    ca(b.col, 0.32 * a),
                    ca(b.col, 0.0),
                ),
                BitKind::Shard => {
                    let r = b.size * s * 1.5;
                    let (sn, cs) = b.rot.sin_cos();
                    let pts: Vec<Vec2> = [
                        vec2(-r, -r * 0.4),
                        vec2(-r * 0.1, -r * 0.9),
                        vec2(r, -r * 0.2),
                        vec2(r * 0.5, r * 0.8),
                        vec2(-r * 0.6, r * 0.5),
                    ]
                    .iter()
                    .map(|q| c + vec2(q.x * cs - q.y * sn, q.x * sn + q.y * cs))
                    .collect();
                    let base = ca(b.col, a.min(1.0));
                    crate::poly::fill(&pts, |q| {
                        if q.x + q.y < c.x + c.y {
                            crate::poly::shade(base, 1.3)
                        } else {
                            crate::poly::shade(base, 0.7)
                        }
                    });
                }
                BitKind::Leaf => {
                    let l = b.size * s * 1.7;
                    let flap = (self.t * 9.0 + b.rot * 4.0).sin().abs() * 0.7 + 0.3;
                    let (d1, d2) = (
                        rot(vec2(l, 0.0), b.rot),
                        rot(vec2(0.0, l * 0.55 * flap), b.rot),
                    );
                    let c1 = ca(b.col, a.min(1.0) * 0.95);
                    let c2 = ca(crate::poly::shade(b.col, 0.78), a.min(1.0) * 0.95);
                    tr([c - d1, c + d2, c + d1], [c1, c1, c2]);
                    tr([c - d1, c - d2, c + d1], [c2, c2, c1]);
                }
                BitKind::Drop => {
                    crate::poly::circle(c, b.size * s, ca(b.col, 0.85 * a));
                }
                _ => {}
            }
        }
        additive(g, || {
            for b in self
                .bits
                .iter()
                .filter(|b| matches!(b.k, BitKind::Spark | BitKind::Ember))
            {
                let a = (b.life / b.max).clamp(0.0, 1.0);
                let c = cx.w(b.x, b.y);
                match b.k {
                    BitKind::Spark => {
                        let t = cx.w(b.x - b.vx * 2.6, b.y - b.vy * 2.6);
                        seg(c, t, b.size * s, ca(hx(0xffffff, 1.0), a), ca(b.col, 0.0));
                    }
                    _ => {
                        let r = b.size * s * (0.7 + 0.5 * a);
                        fan(c, r * 2.6, r * 2.6, 8, ca(b.col, 0.55 * a), ca(b.col, 0.0));
                        crate::poly::circle(c, r * 0.8, ca(hx(0xfff0c0, 1.0), a));
                    }
                }
            }
        });
    }
}
