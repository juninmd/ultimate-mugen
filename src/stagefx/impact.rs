use super::*;

impl StageReact {
    /// Turn an impulse into stage motion, debris and marks.
    pub(super) fn apply(&mut self, i: Impulse) {
        self.applied += 1;
        let s = i.strength.clamp(0.0, 1.5);
        let x = i.x;
        let kind = self.kind;
        let (dust, ripple) = (self.look.dust, self.look.ripple);
        match i.kind {
            ImpulseKind::Hit => {
                self.shake = self.shake.max(s * 0.7);
                if s > 0.3 {
                    self.flicker = self.flicker.max(s);
                }
                self.gust = (self.gust + s * 0.7).min(3.0);
                let sg = if self.r() < 0.5 { -1.0 } else { 1.0 };
                self.swing_v += sg * s * 1.6;
                self.cheer = self.cheer.max(s * 0.3);
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: s * 0.6,
                });
                self.dust_at(x, 2 + (s * 8.0) as usize, 1.2, 0.35, dust);
                if s > 0.45 {
                    self.shards(x, 3 + (s * 8.0) as usize, s);
                    self.ripples.push(Ripple {
                        x,
                        y: 3.0,
                        age: 0.0,
                        power: s * 0.8,
                    });
                    self.decal(x, 16.0 + s * 26.0, DecalKind::Crack);
                }
                if s > 0.8 {
                    self.decal(x, 20.0 + s * 20.0, DecalKind::Scorch);
                }
                self.special(kind, x, s * 0.7);
            }
            ImpulseKind::Land => {
                self.shake = self.shake.max(s * 0.28);
                self.dust_at(x, 3 + (s * 10.0) as usize, 1.6, 0.4, dust);
                self.ripples.push(Ripple {
                    x,
                    y: 2.0,
                    age: 0.0,
                    power: s * 0.7,
                });
                if self.look.wet {
                    self.drops(x, 3 + (s * 6.0) as usize, s);
                }
                if s > 0.7 {
                    self.shards(x, 2 + (s * 5.0) as usize, s * 0.7);
                    self.shocks.push(Shock {
                        x,
                        age: 0.0,
                        power: s * 0.5,
                    });
                    self.gust = (self.gust + s * 0.4).min(3.0);
                }
                if s > 0.95 {
                    self.decal(x, 14.0 + s * 18.0, DecalKind::Crack);
                }
                if s > 0.45 {
                    self.special(kind, x, s * 0.45);
                }
            }
            ImpulseKind::Super => {
                self.shake = self.shake.max(0.9);
                self.flicker = 1.0;
                self.gust = (self.gust + 1.8).min(3.0);
                self.cheer = self.cheer.max(0.85);
                self.swing_v += 2.2;
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: 1.4,
                });
                self.ripples.push(Ripple {
                    x,
                    y: 3.0,
                    age: 0.0,
                    power: 1.4,
                });
                self.dust_at(x, 12, 2.4, 0.5, dust);
                self.decal(x, 38.0, DecalKind::Scorch);
                self.special(kind, x, 1.4);
            }
            ImpulseKind::Ko => {
                self.shake = 1.0;
                self.flicker = 1.0;
                self.gust = 3.0;
                self.cheer = 1.0;
                self.swing_v += 3.0;
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: 1.8,
                });
                self.ripples.push(Ripple {
                    x,
                    y: 3.0,
                    age: 0.0,
                    power: 1.6,
                });
                self.dust_at(x, 16, 2.8, 0.6, dust);
                self.shards(x, 16, 1.2);
                self.decal(x, 44.0, DecalKind::Crack);
                self.decal(x, 36.0, DecalKind::Scorch);
                self.special(kind, x, 1.8);
            }
            ImpulseKind::Wall => {
                self.shake = self.shake.max(0.6 * s + 0.2);
                self.flicker = self.flicker.max(0.5 * s);
                self.gust = (self.gust + s).min(3.0);
                self.dust_at(x, 8, 1.8, 0.4, dust);
                self.shards(x, 6 + (s * 8.0) as usize, s + 0.3);
                self.decal(x, 18.0 + s * 20.0, DecalKind::Crack);
                for q in 0..5 {
                    let life = self.rr(0.8, 1.5);
                    let (vx, sz) = (self.rr(-0.5, 0.5), self.rr(0.8, 1.8));
                    let yy = -self.rr(40.0, 150.0);
                    {
                        let b = Bit {
                            k: BitKind::Shard,
                            x: x + self.rr(-8.0, 8.0),
                            y: yy,
                            vx,
                            vy: 0.0,
                            g: 0.2,
                            life,
                            max: life,
                            size: sz,
                            rot: q as f32,
                            vrot: 0.2,
                            col: self.look.shard,
                            ground: self.rr(0.0, 14.0),
                        };
                        self.push_bit(b);
                    }
                }
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: s * 0.8,
                });
                self.ripples.push(Ripple {
                    x,
                    y: 3.0,
                    age: 0.0,
                    power: s,
                });
                self.special(kind, x, s);
            }
        }
        let _ = ripple;
    }
    /// Lateral push (−1..1+) at world x from passing shock waves.
    pub(super) fn push_at(&self, wx: f32) -> f32 {
        let mut p = 0.0;
        for s in &self.shocks {
            let d = wx - s.x;
            let front = s.age * 260.0;
            let e = (d.abs() - front) / 38.0;
            p += d.signum() * s.power * (-e * e).exp() * (1.0 - s.age / 1.6);
        }
        p
    }
    /// Smooth wind value around 0.35 (steady breeze) with gusts; `ph` desynchronises elements.
    pub(super) fn wind(&self, x: f32, ph: f32) -> f32 {
        let base = 0.35
            + 0.28 * (self.t * 0.9 + x * 0.011 + ph).sin()
            + 0.12 * (self.t * 2.1 + ph * 2.0 + x * 0.02).sin();
        base * (1.0 + self.gust * 1.3) + self.shake * 0.12 * (self.t * 31.0 + ph * 5.0).sin()
    }
    /// Light multiplier with random dropouts while `flicker` is high.
    pub(super) fn flick(&self, i: u32) -> f32 {
        let n = hash(i, (self.t * 16.0) as u32);
        let calm = 0.93 + 0.07 * (self.t * 3.0 + i as f32 * 1.7).sin();
        if n < self.flicker * 0.7 {
            0.2 + n
        } else {
            calm * (1.0 + self.flicker * 0.3)
        }
    }
}
