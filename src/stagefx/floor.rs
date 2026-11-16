use super::*;

impl StageReact {
    /// Everything on the floor plane: decals, ripples, light pools, ambient gloss streak.
    pub(super) fn floor_common(&self, cx: &Ctx, g: &Gfx, m: &Match) {
        let s = cx.s;
        // glossy sheen band along the floor top (wet / polished surfaces)
        if self.look.gloss > 0.1 {
            let a = self.look.gloss * 0.22;
            qd(
                [
                    cx.w(cx.cam() - 260.0, 0.0),
                    cx.w(cx.cam() + 260.0, 0.0),
                    cx.w(cx.cam() + 260.0, 22.0),
                    cx.w(cx.cam() - 260.0, 22.0),
                ],
                [
                    ca(self.look.ripple, a),
                    ca(self.look.ripple, a),
                    ca(self.look.ripple, 0.0),
                    ca(self.look.ripple, 0.0),
                ],
            );
        }
        // decals
        for d in &self.decals {
            let fade = if d.age > 30.0 {
                1.0 - (d.age - 30.0) / 10.0
            } else {
                1.0
            };
            let pop = (d.age * 8.0).min(1.0);
            let c0 = cx.w(d.x, d.y);
            match d.kind {
                DecalKind::Scorch => {
                    let r = d.size * pop;
                    fan(
                        c0,
                        r * s,
                        r * 0.28 * s,
                        18,
                        ca(self.look.crack, 0.55 * fade),
                        ca(self.look.crack, 0.0),
                    );
                }
                DecalKind::Crack => {
                    for arm in 0..7u32 {
                        let a =
                            (arm as f32 + hash(d.seed, arm) * 0.8) / 7.0 * std::f32::consts::TAU;
                        let mut p = (0.0f32, 0.0f32);
                        let len = d.size * (0.45 + hash(d.seed, arm + 20) * 0.55) * pop;
                        for k in 0..3 {
                            let a2 = a + (hash(d.seed, arm * 7 + k + 40) - 0.5) * 0.8;
                            let q = (p.0 + a2.cos() * len / 3.0, p.1 + a2.sin() * len / 3.0 * 0.3);
                            let w = (1.3 - k as f32 * 0.35) * s;
                            seg(
                                cx.w(d.x + p.0, d.y + p.1),
                                cx.w(d.x + q.0, d.y + q.1),
                                w,
                                ca(self.look.crack, 0.85 * fade),
                                ca(self.look.crack, 0.85 * fade),
                            );
                            if k == 1 {
                                let b2 = a2 + 0.9 * if arm % 2 == 0 { 1.0 } else { -1.0 };
                                seg(
                                    cx.w(d.x + q.0, d.y + q.1),
                                    cx.w(
                                        d.x + q.0 + b2.cos() * len * 0.2,
                                        d.y + q.1 + b2.sin() * len * 0.06,
                                    ),
                                    0.8 * s,
                                    ca(self.look.crack, 0.7 * fade),
                                    ca(self.look.crack, 0.0),
                                );
                            }
                            p = q;
                        }
                    }
                    if let Some(gc) = self.look.glow {
                        let hot = (1.0 - d.age / 12.0).max(0.25) * fade;
                        additive(g, || {
                            for arm in 0..7u32 {
                                let a = (arm as f32 + hash(d.seed, arm) * 0.8) / 7.0
                                    * std::f32::consts::TAU;
                                let len = d.size * (0.45 + hash(d.seed, arm + 20) * 0.55) * pop;
                                seg(
                                    c0,
                                    cx.w(d.x + a.cos() * len * 0.9, d.y + a.sin() * len * 0.27),
                                    0.9 * s,
                                    ca(gc, 0.6 * hot),
                                    ca(gc, 0.0),
                                );
                            }
                        });
                    }
                }
            }
        }
        // ripples
        for r in &self.ripples {
            let t = r.age / 1.5;
            let e = 1.0 - (1.0 - t).powi(2);
            let rx = (4.0 + e * (26.0 + 36.0 * r.power)) * s;
            let a = (1.0 - t) * (0.25 + 0.4 * r.power.min(1.2));
            let c = cx.w(r.x, r.y + 3.0 + e * 4.0);
            additive(g, || {
                ring_e(
                    c,
                    rx,
                    rx * 0.22,
                    (0.9 + r.power) * s * (1.0 - t * 0.6),
                    ca(self.look.ripple, a),
                );
                if r.power > 0.5 && t < 0.7 {
                    ring_e(
                        c,
                        rx * 0.62,
                        rx * 0.14,
                        0.7 * s,
                        ca(self.look.ripple, a * 0.6),
                    );
                }
            });
        }
        // light pools under the fighters
        let pool = self.look.pool;
        additive(g, || {
            for (i, f) in m
                .fighters
                .iter()
                .filter(|f| !f.is_helper && !f.destroyed && f.alive)
                .enumerate()
            {
                let fl = self.flick(300 + i as u32);
                let c = cx.w(f.pos.x, 6.0);
                let pc = if self.kind == StageKind::Neon {
                    if i % 2 == 0 {
                        hx(0xff40d8, 1.0)
                    } else {
                        hx(0x30e8ff, 1.0)
                    }
                } else {
                    pool
                };
                fan(c, 48.0 * s, 9.0 * s, 20, ca(pc, 0.17 * fl), ca(pc, 0.0));
            }
        });
    }
}
