use super::*;

impl StageReact {
    pub(super) fn footsteps(&mut self, m: &Match) {
        let roots: Vec<&ume_engine::fighter::Fighter> = m
            .fighters
            .iter()
            .filter(|f| !f.is_helper && !f.destroyed)
            .collect();
        for (i, f) in roots.iter().enumerate() {
            let Some(p) = self.prev.get(i).copied() else {
                continue;
            };
            if p.id != f.id {
                continue;
            }
            if f.pos.y >= -0.05 && f.state_type != 'L' && (f.pos.x - p.x).abs() < 20.0 {
                let acc = p.step_acc + (f.pos.x - p.x).abs();
                if acc > 13.0 && self.look.wet {
                    self.ripples.push(Ripple {
                        x: f.pos.x,
                        y: 2.0,
                        age: 0.0,
                        power: 0.35,
                    });
                    if self.r() < 0.5 {
                        self.drops(f.pos.x, 2, 0.7);
                    }
                    self.prev[i].step_acc = 0.0;
                } else {
                    self.prev[i].step_acc = if acc > 13.0 { 0.0 } else { acc };
                }
            }
        }
    }
    pub(super) fn push_bit(&mut self, b: Bit) {
        self.bits.push(b);
    }
    pub(super) fn dust_at(&mut self, x: f32, n: usize, spread: f32, up: f32, col: Color) {
        for _ in 0..n {
            let (vx, vy, life, size, jx) = (
                self.rr(-spread, spread),
                -self.rr(0.1, up),
                self.rr(0.45, 0.9),
                self.rr(2.0, 4.5),
                self.rr(-6.0, 6.0),
            );
            {
                let b = Bit {
                    k: BitKind::Dust,
                    x: x + jx,
                    y: self.rr(-1.0, 4.0),
                    vx,
                    vy,
                    g: -0.004,
                    life,
                    max: life,
                    size,
                    rot: 0.0,
                    vrot: 0.0,
                    col,
                    ground: 99.0,
                };
                self.push_bit(b);
            }
        }
    }
    pub(super) fn shards(&mut self, x: f32, n: usize, power: f32) {
        for _ in 0..n {
            let (vx, vy, life, size, vr, gr) = (
                self.rr(-1.0, 1.0) * (1.2 + power * 2.2),
                -self.rr(1.4, 3.6) * (0.6 + power * 0.6),
                self.rr(0.8, 1.5),
                self.rr(0.8, 1.7),
                self.rr(-0.4, 0.4),
                self.rr(0.0, 14.0),
            );
            let col = lerpc(self.look.shard, WHITE, self.rr(0.0, 0.25));
            {
                let b = Bit {
                    k: BitKind::Shard,
                    x,
                    y: -4.0,
                    vx,
                    vy,
                    g: 0.2,
                    life,
                    max: life,
                    size,
                    rot: 0.0,
                    vrot: vr,
                    col,
                    ground: gr,
                };
                self.push_bit(b);
            }
        }
    }
    pub(super) fn drops(&mut self, x: f32, n: usize, power: f32) {
        for _ in 0..n {
            let (vx, vy, life, gr) = (
                self.rr(-1.0, 1.0) * (0.8 + power),
                -self.rr(1.6, 3.4) * (0.6 + power * 0.5),
                self.rr(0.6, 1.1),
                self.rr(0.0, 10.0),
            );
            let c = self.look.ripple;
            {
                let b = Bit {
                    k: BitKind::Drop,
                    x,
                    y: -1.0,
                    vx,
                    vy,
                    g: 0.17,
                    life,
                    max: life,
                    size: self.rr(0.6, 1.1),
                    rot: 0.0,
                    vrot: 0.0,
                    col: ca(c, 0.9),
                    ground: gr,
                };
                self.push_bit(b);
            }
        }
    }
    pub(super) fn leaves(&mut self, x: f32, n: usize, power: f32, from_top: bool) {
        for _ in 0..n {
            let dirx = if self.r() < 0.5 { -1.0 } else { 1.0 };
            let (vx, vy, life, size, vr) = (
                dirx * self.rr(0.8, 2.6) * (0.5 + power),
                if from_top {
                    self.rr(0.1, 0.5)
                } else {
                    -self.rr(0.8, 2.6) * (0.5 + power * 0.5)
                },
                self.rr(2.2, 4.2),
                self.rr(1.4, 2.4),
                self.rr(-0.25, 0.25),
            );
            let col = if self.r() < 0.5 {
                self.look.leaf
            } else {
                self.look.leaf2
            };
            let (y, gr) = (
                if from_top {
                    self.rr(-170.0, -60.0)
                } else {
                    self.rr(-6.0, -1.0)
                },
                self.rr(0.0, 22.0),
            );
            {
                let b = Bit {
                    k: BitKind::Leaf,
                    x: x + self.rr(-14.0, 14.0),
                    y,
                    vx,
                    vy,
                    g: 0.022,
                    life,
                    max: life,
                    size,
                    rot: self.rr(0.0, std::f32::consts::TAU),
                    vrot: vr,
                    col,
                    ground: gr,
                };
                self.push_bit(b);
            }
        }
    }
}
