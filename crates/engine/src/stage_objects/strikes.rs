use super::{ObjectEvent, Pickup, REHIT_TICKS, SHAKE_TICKS};
use crate::fighter::V2;
use crate::matchs::Match;

const HEAVY_DAMAGE: f32 = 90.0;
const MAX_ATTACKER_PAUSE: i32 = 8;
const DROP_POP_SPEED: f32 = 3.4;

fn bounds(boxes: &[[f32; 4]]) -> Option<[f32; 4]> {
    let first = *boxes.first()?;
    Some(boxes[1..].iter().fold(first, |a, b| {
        [
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]
    }))
}

fn touching(a: [f32; 4], b: [f32; 4]) -> Option<(f32, f32)> {
    let (x0, x1) = (a[0].max(b[0]), a[2].min(b[2]));
    let (y0, y1) = (a[1].max(b[1]), a[3].min(b[3]));
    (x0 < x1 && y0 < y1).then_some(((x0 + x1) * 0.5, (y0 + y1) * 0.5))
}

impl Match {
    pub(super) fn strike_breakables(&mut self) {
        for a in 0..self.fighters.len() {
            let fa = &self.fighters[a];
            if fa.destroyed
                || fa.ikemen.standby
                || !fa.hitdef_active
                || (fa.move_type != 'A' && !fa.is_helper)
            {
                continue;
            }
            let Some(damage) = fa.hitdef.as_ref().map(|hd| hd.damage * fa.attack_mul) else {
                continue;
            };
            let pause = fa.hitdef.as_ref().map_or(0, |hd| hd.pause_p1);
            fa.fill_boxes(1, &mut self.scratch_c1);
            let Some(reach) = bounds(&self.scratch_c1) else {
                continue;
            };
            let mut connected = false;
            for bi in 0..self.objects.breakables.len() {
                let b = &self.objects.breakables[bi];
                if b.broken || b.rehit > 0 || !self.object_in_reach(a, b.lane, b.depth) {
                    continue;
                }
                let Some(at) = touching(reach, b.hurt_box()) else {
                    continue;
                };
                self.damage_breakable(bi, damage, at, a);
                connected = true;
            }
            if connected {
                let fa = &mut self.fighters[a];
                fa.move_contact = 1;
                fa.move_hit = 1;
                fa.hit_pause = fa.hit_pause.max(pause.clamp(0, MAX_ATTACKER_PAUSE));
            }
        }
        for pi in 0..self.projectiles.len() {
            let p = &self.projectiles[pi];
            if p.dying || p.remove || p.hits <= 0 {
                continue;
            }
            let Some(frame) = p.cur.frame() else { continue };
            let Some(local) = bounds(&frame.clsn1) else {
                continue;
            };
            let (sx, sy) = (p.lscl * p.scale[0], p.lscl * p.scale[1]);
            let (near, far) = if p.facing < 0.0 {
                (-local[2] * sx, -local[0] * sx)
            } else {
                (local[0] * sx, local[2] * sx)
            };
            let reach = [
                p.pos.x + near,
                p.pos.y + local[1] * sy,
                p.pos.x + far,
                p.pos.y + local[3] * sy,
            ];
            let (damage, owner, hit_anim) = (p.hitdef.damage, p.owner, p.hit_anim);
            let Some(shooter) = self.idx(owner) else {
                continue;
            };
            let damage = damage * self.fighters[shooter].attack_mul;
            for bi in 0..self.objects.breakables.len() {
                let b = &self.objects.breakables[bi];
                if b.broken || b.rehit > 0 || !self.object_in_reach(shooter, b.lane, b.depth) {
                    continue;
                }
                let Some(at) = touching(reach, b.hurt_box()) else {
                    continue;
                };
                self.damage_breakable(bi, damage, at, shooter);
                let p = &mut self.projectiles[pi];
                p.hits -= 1;
                if p.hits <= 0 {
                    p.vel = V2::default();
                    p.accel = V2::default();
                    if hit_anim >= 0 && p.cur.def.anims.contains_key(&hit_anim) {
                        p.cur.anim = hit_anim;
                        p.cur.elem = 0;
                        p.cur.elem_time = 0;
                        p.cur.elapsed = 0;
                        p.dying = true;
                    } else {
                        p.remove = true;
                    }
                }
                break;
            }
        }
    }

    fn damage_breakable(&mut self, bi: usize, damage: f32, at: (f32, f32), attacker: usize) {
        let damage = damage.max(1.0);
        let b = &mut self.objects.breakables[bi];
        b.life -= damage;
        b.rehit = REHIT_TICKS;
        b.shake = SHAKE_TICKS;
        let (lane, x, height, score, drop) = (b.lane, b.x, b.height, b.score, b.drop);
        let broke = b.life <= 0.0;
        if broke {
            b.life = 0.0;
            b.broken = true;
            b.broken_ticks = 0;
        }
        self.objects.emit(ObjectEvent::Struck {
            index: bi,
            x: at.0,
            y: at.1,
            lane,
            heavy: damage >= HEAVY_DAMAGE,
        });
        if !broke {
            return;
        }
        self.objects.emit(ObjectEvent::Broken { index: bi });
        let root = self.idx(self.fighters[attacker].root).unwrap_or(attacker);
        if self.fighters[root].side == self.objects.collector_side {
            self.fighters[root].score += score as f64;
            self.fighters[root].score_total += score as f64;
        }
        let Some(spec) = drop else { return };
        if self.objects.pickups.len() == self.objects.pickups.capacity() {
            return;
        }
        let toss = (self.random() - 0.5) * 1.6;
        let drop_x = x.clamp(self.stage.bound_left + 8.0, self.stage.bound_right - 8.0);
        self.objects.pickups.push(Pickup {
            spec,
            pos: V2 {
                x: drop_x,
                y: -height * 0.6,
            },
            vel: V2 {
                x: toss,
                y: -DROP_POP_SPEED,
            },
            lane,
            age: 0,
        });
        self.objects.emit(ObjectEvent::Dropped {
            kind: spec.kind,
            x: drop_x,
            lane,
        });
    }
}
