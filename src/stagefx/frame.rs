use super::*;

impl StageReact {
    /// Call once per drawn frame, before drawing the stage.
    pub fn begin_frame(&mut self, m: &Match, gain: f32) {
        if !self.active() {
            return;
        }
        let dt = get_frame_time().clamp(0.0, 0.05);
        self.t += dt;
        if m.round_no != self.round_no {
            self.round_no = m.round_no;
            self.decals.clear();
            self.ripples.clear();
            self.prev.clear();
        }
        let detected = self.observe(m);
        let mut imps: Vec<Impulse> = if self.explicit {
            detected
                .into_iter()
                .filter(|i| i.kind == ImpulseKind::Wall)
                .collect()
        } else {
            detected
        };
        imps.append(&mut self.queue);
        for i in imps {
            self.apply(i);
        }
        self.footsteps(m);
        let k = dt * 60.0;
        self.shake = (self.shake - dt * 1.5).max(0.0);
        self.flicker = (self.flicker - dt * 1.4).max(0.0);
        self.gust = (self.gust - dt * 0.8).max(0.0);
        self.cheer = (self.cheer - dt * 0.35).max(0.0);
        // pendulum: lamps swing back to rest (spring + damping)
        self.swing_v += (-14.0 * self.swing - 1.6 * self.swing_v) * dt;
        self.swing += self.swing_v * dt;
        self.rotor += (0.5 + self.gust * 2.0 + self.shake * 3.0) * dt;
        for s in &mut self.shocks {
            s.age += dt;
        }
        self.shocks.retain(|s| s.age < 1.6);
        for d in &mut self.decals {
            d.age += dt;
        }
        self.decals.retain(|d| d.age < 40.0);
        for r in &mut self.ripples {
            r.age += dt;
        }
        self.ripples.retain(|r| r.age < 1.5);
        let gust = self.gust;
        for b in &mut self.bits {
            b.life -= dt;
            b.x += b.vx * k;
            b.y += b.vy * k;
            b.vy += b.g * k;
            b.rot += b.vrot * k;
            match b.k {
                BitKind::Dust => {
                    b.vx *= 0.95f32.powf(k);
                    b.vy *= 0.97f32.powf(k);
                    b.size += 0.04 * k;
                }
                BitKind::Leaf => {
                    b.vx += (0.02 + gust * 0.03) * k * (self.t * 3.0 + b.rot).sin();
                    b.vy = b.vy.min(0.7);
                    b.vrot += 0.01 * (b.rot * 3.0).sin();
                }
                _ => {}
            }
            if matches!(b.k, BitKind::Shard | BitKind::Drop | BitKind::Leaf) && b.y > b.ground {
                b.y = b.ground;
                b.vy = 0.0;
                b.vx *= 0.5;
                b.vrot = 0.0;
                b.life = b.life.min(0.6);
            }
        }
        // ambient ripples: rain drops on the neon street, fish / drips in the pond, spirit pulses
        let rate = match self.kind {
            StageKind::Neon => 3.5,
            StageKind::Bamboo => 0.8,
            StageKind::Spirit => 0.5,
            _ => 0.0,
        };
        if rate > 0.0 && self.r() < dt * rate {
            let (x, y) = (m.camera.x + self.rr(-200.0, 200.0), self.rr(2.0, 26.0));
            self.ripples.push(Ripple {
                x,
                y,
                age: 0.0,
                power: 0.2,
            });
        }
        // photosensitivity / reduced-effects setting: no flicker or shake, calmer gusts
        let gn = gain.clamp(0.0, 1.3);
        self.flicker = self.flicker.min(gn);
        self.shake = self.shake.min(gn);
        if gn < 0.5 {
            self.gust = self.gust.min(1.0);
        }
        self.bits.retain(|b| b.life > 0.0);
        if self.bits.len() > 260 {
            let n = self.bits.len() - 260;
            self.bits.drain(0..n);
        }
    }
    pub(super) fn observe(&mut self, m: &Match) -> Vec<Impulse> {
        let mut out = vec![];
        let roots: Vec<&ume_engine::fighter::Fighter> = m
            .fighters
            .iter()
            .filter(|f| !f.is_helper && !f.destroyed)
            .collect();
        self.prev.resize(roots.len(), FSnap::default());
        let (l, r) = (
            m.camera.x - m.view_w / 2.0 + 14.0,
            m.camera.x + m.view_w / 2.0 - 14.0,
        );
        for (i, f) in roots.iter().enumerate() {
            let p = self.prev[i];
            if p.id == f.id {
                if f.hit_shake > p.hit_shake && f.hit_shake > 0 {
                    let s = (f.ghv.damage / 100.0).clamp(0.2, 1.0)
                        * if f.ghv.guarded { 0.35 } else { 1.0 }
                        * if f.ghv.fall { 1.25 } else { 1.0 };
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: s,
                        kind: ImpulseKind::Hit,
                    });
                }
                if p.y < -1.5 && f.pos.y >= -0.05 && p.vy > 1.5 {
                    let s = (p.vy / 8.0).clamp(0.15, 1.2)
                        * if f.ghv.fall || f.state_type == 'L' {
                            1.5
                        } else {
                            1.0
                        };
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: s,
                        kind: ImpulseKind::Land,
                    });
                }
                if p.alive && !f.alive {
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: 1.0,
                        kind: ImpulseKind::Ko,
                    });
                }
                let at_edge = f.pos.x <= l
                    || f.pos.x >= r
                    || f.pos.x <= m.stage.bound_left + 4.0
                    || f.pos.x >= m.stage.bound_right - 4.0;
                if f.move_type == 'H'
                    && at_edge
                    && p.vx.abs() > 2.2
                    && f.vel.x.abs() < 0.8
                    && p.cd <= 0.0
                {
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: (p.vx.abs() / 7.0).clamp(0.2, 1.0),
                        kind: ImpulseKind::Wall,
                    });
                    self.prev[i].cd = 0.5;
                }
            }
            let cd = (self.prev[i].cd - 0.016).max(0.0);
            self.prev[i] = FSnap {
                id: f.id,
                hit_shake: f.hit_shake,
                y: f.pos.y,
                vy: f.vel.y,
                vx: f.vel.x,
                x: f.pos.x,
                alive: f.alive,
                cd,
                step_acc: p.step_acc,
            };
        }
        let sup = m.pause.as_ref().filter(|p| p.is_super);
        if let Some(p) = sup {
            if !self.was_super {
                let x = m
                    .fighters
                    .iter()
                    .find(|f| f.id == p.by)
                    .map(|f| f.pos.x)
                    .unwrap_or(m.camera.x);
                out.push(Impulse {
                    x,
                    strength: 1.0,
                    kind: ImpulseKind::Super,
                });
            }
        }
        self.was_super = sup.is_some();
        out
    }
}
