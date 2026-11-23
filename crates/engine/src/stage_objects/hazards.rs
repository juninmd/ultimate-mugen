use super::ObjectEvent;
use crate::matchs::Match;
use crate::fighter::GetHitVars;

const SLAM_SPEED_X: f32 = 2.6;
const SLAM_SPEED_Y: f32 = 3.2;

impl Match {
    pub(super) fn slam_breakables(&mut self) {
        for fi in 0..self.fighters.len() {
            let f = &self.fighters[fi];
            if f.destroyed || f.is_helper || f.ikemen.standby {
                continue;
            }
            let knocked = f.move_type == 'H' || f.ghv.fall;
            let vx = f.vel.x.abs();
            let vy = f.vel.y.abs();
            if !knocked || (vx < SLAM_SPEED_X && vy < SLAM_SPEED_Y) {
                continue;
            }
            let (fx, fy) = (f.pos.x, f.pos.y);
            let damage = (vx * 16.0 + vy * 12.0).clamp(55.0, 150.0);
            let mut hit_any = false;
            for bi in 0..self.objects.breakables.len() {
                let b = &self.objects.breakables[bi];
                if b.broken || b.rehit > 0 || !self.object_in_reach(fi, b.lane, b.depth) {
                    continue;
                }
                if (fx - b.x).abs() <= b.half_width + 16.0 && fy <= 4.0 && fy >= -b.height - 20.0 {
                    self.damage_breakable(bi, damage, (b.x, fy.min(-24.0)), fi);
                    hit_any = true;
                }
            }
            if hit_any {
                self.fighters[fi].vel.x *= 0.72;
            }
        }
    }

    pub(super) fn trigger_blast(
        &mut self,
        x: f32,
        lane: f32,
        radius: f32,
        damage: f32,
        launch: [f32; 2],
    ) {
        self.objects.emit(ObjectEvent::Blast {
            x,
            lane,
            radius,
            damage,
        });
        self.shake_with_dir(12, 10.0, 4.5, 0.0, 0);
        for fi in 0..self.fighters.len() {
            let f = &self.fighters[fi];
            if f.destroyed || f.is_helper || f.ikemen.standby || !f.alive {
                continue;
            }
            let dx = f.pos.x - x;
            if dx.abs() > radius || f.pos.y < -radius {
                continue;
            }
            if !self.object_in_reach(fi, lane, 24.0) {
                continue;
            }
            let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
            let falloff = (1.0 - (dx.abs() / radius.max(1.0)) * 0.45).clamp(0.45, 1.0);
            let f = &mut self.fighters[fi];
            if damage > 0.0 {
                f.life = (f.life - damage * falloff).max(1.0);
            }
            if launch[0] != 0.0 || launch[1] != 0.0 {
                f.vel.x = dir * launch[0].abs() * falloff;
                f.vel.y = launch[1] * falloff;
                f.hit_shake = f.hit_shake.max(6);
                if launch[1] < -0.5 {
                    let def = f.def.clone();
                    f.ghv = GetHitVars::default();
                    f.ghv.fall = true;
                    f.ghv.yaccel = f.c("movement.yaccel", 0.44);
                    f.ghv.xvel = f.vel.x;
                    f.ghv.yvel = f.vel.y;
                    f.ctrl = false;
                    f.move_type = 'H';
                    self.change_state(fi, 5030, def);
                }
            }
        }
    }
}
