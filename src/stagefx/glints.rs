use super::*;

impl StageReact {
    pub(super) fn sparks(&mut self, x: f32, y: f32, n: usize, col: Color, power: f32) {
        for _ in 0..n {
            let a = self.rr(-3.1, 0.0);
            let s = self.rr(0.8, 3.6) * (0.6 + power);
            let life = self.rr(0.3, 0.7);
            {
                let b = Bit {
                    k: BitKind::Spark,
                    x,
                    y,
                    vx: a.cos() * s,
                    vy: a.sin() * s * 0.8,
                    g: 0.14,
                    life,
                    max: life,
                    size: self.rr(0.6, 1.2),
                    rot: 0.0,
                    vrot: 0.0,
                    col,
                    ground: 99.0,
                };
                self.push_bit(b);
            }
        }
    }
    pub(super) fn embers(&mut self, x: f32, n: usize, col: Color, up: f32) {
        for _ in 0..n {
            let life = self.rr(0.8, 1.8);
            {
                let b = Bit {
                    k: BitKind::Ember,
                    x: x + self.rr(-10.0, 10.0),
                    y: self.rr(-3.0, 3.0),
                    vx: self.rr(-0.5, 0.5),
                    vy: -self.rr(0.6, 1.8) * up,
                    g: -0.004,
                    life,
                    max: life,
                    size: self.rr(0.5, 1.3),
                    rot: 0.0,
                    vrot: 0.0,
                    col,
                    ground: 99.0,
                };
                self.push_bit(b);
            }
        }
    }
    pub(super) fn decal(&mut self, x: f32, size: f32, kind: DecalKind) {
        if self.decals.len() >= 9 {
            self.decals.remove(0);
        }
        let seed = (self.r() * 1e6) as u32;
        let y = self.rr(3.0, 14.0);
        self.decals.push(Decal {
            x,
            y,
            age: 0.0,
            size,
            seed,
            kind,
        });
    }
    /// Stage-specific flavour of an impact: petals, sparks, leaves, sand, wisps, lava.
    pub(super) fn special(&mut self, kind: StageKind, x: f32, s: f32) {
        let n = (4.0 + s * 12.0) as usize;
        match kind {
            StageKind::Dojo => self.leaves(x, n, s, false),
            StageKind::Neon => {
                let jx = self.rr(-60.0, 60.0);
                self.sparks(x + jx, -140.0, 3 + (s * 6.0) as usize, hx(0xffe8a0, 1.0), s);
                self.drops(x, n / 2, s);
            }
            StageKind::Bamboo => {
                self.leaves(x, n, s, true);
                self.drops(x, n, s);
            }
            StageKind::Plains => {
                self.dust_at(x, n, 3.2 + s, 0.5, self.look.dust);
                self.leaves(x, n / 2, s * 1.4, false);
            }
            StageKind::Spirit => {
                self.embers(x, n, hx(0x7affee, 1.0), 1.0);
                self.leaves(x, n / 2, s * 0.6, false);
            }
            StageKind::Volcano => {
                self.embers(x, n * 2, hx(0xff9a30, 1.0), 1.4);
                self.sparks(x, -2.0, n, hx(0xffb040, 1.0), s * 1.2);
                for _ in 0..(s * 3.0) as usize {
                    let jx = self.rr(-60.0, 60.0);
                    self.shards(x + jx, 1, s);
                }
            }
            StageKind::Arena => {
                self.dust_at(x, n, 2.8 + s, 0.45, self.look.dust);
                self.sparks(x, -6.0, (n / 2).max(2), hx(0xffe098, 1.0), s * 0.9);
            }
            StageKind::Custom => {}
        }
    }
}
