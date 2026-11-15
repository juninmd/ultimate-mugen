use super::{ObjectEvent, PICKUP_ARM_TICKS, PICKUP_LIFETIME, PICKUP_REACH_X, PICKUP_REACH_Y};
use crate::lane::LANE_REACH;
use crate::matchs::Match;

const GRAVITY: f32 = 0.34;
const BOUNCE: f32 = 0.38;
const REST_SPEED: f32 = 1.1;
const LANE_TOLERANCE: f32 = LANE_REACH * 2.0;

impl Match {
    pub(super) fn step_pickups(&mut self) {
        let mut pi = 0;
        while pi < self.objects.pickups.len() {
            let p = &mut self.objects.pickups[pi];
            p.age += 1;
            if p.pos.y < 0.0 || p.vel.y < 0.0 {
                p.vel.y += GRAVITY;
                p.pos.x += p.vel.x;
                p.pos.y += p.vel.y;
                if p.pos.y >= 0.0 {
                    p.pos.y = 0.0;
                    p.vel.x *= 0.5;
                    p.vel.y = if p.vel.y > REST_SPEED {
                        -p.vel.y * BOUNCE
                    } else {
                        0.0
                    };
                }
            }
            if p.age > PICKUP_LIFETIME {
                self.objects.pickups.swap_remove(pi);
                continue;
            }
            if p.age >= PICKUP_ARM_TICKS {
                if let Some(taker) = self.pickup_taker(pi) {
                    self.collect_pickup(pi, taker);
                    continue;
                }
            }
            pi += 1;
        }
    }

    fn pickup_taker(&self, pi: usize) -> Option<usize> {
        let p = &self.objects.pickups[pi];
        (0..self.fighters.len()).find(|&fi| {
            let f = &self.fighters[fi];
            !f.is_helper
                && !f.destroyed
                && !f.ikemen.standby
                && f.alive
                && f.life > 0.0
                && f.side == self.objects.collector_side
                && (f.pos.x - p.pos.x).abs() <= PICKUP_REACH_X
                && (f.pos.y - p.pos.y).abs() <= PICKUP_REACH_Y
                && (!self.lanes.active || (f.lane - p.lane).abs() <= LANE_TOLERANCE)
        })
    }

    fn collect_pickup(&mut self, pi: usize, taker: usize) {
        let p = self.objects.pickups.swap_remove(pi);
        let f = &mut self.fighters[taker];
        let before = f.life;
        f.life = (f.life + p.spec.heal * f.life_max).min(f.life_max);
        f.power = (f.power + p.spec.power * f.power_max).min(f.power_max);
        f.score += p.spec.score as f64;
        f.score_total += p.spec.score as f64;
        let event = ObjectEvent::Collected {
            spec: p.spec,
            x: p.pos.x,
            lane: p.lane,
            fighter: f.id,
            healed: f.life - before,
        };
        self.objects.emit(event);
    }
}
