use super::{Event, Match};

pub const WALL_SHATTER_SPEED: f32 = 7.5;
pub const WALL_SHATTER_DAMAGE: f32 = 35.0;
pub const WALL_SHATTER_METER: f32 = 150.0;

impl Match {
    pub(super) fn bound_fighters(&mut self) {
        if self.power_arena || self.is_lane_scrolling() {
            return;
        }
        let left = self.camera.x - self.view_w / 2.0;
        let right = self.camera.x + self.view_w / 2.0;
        let (bl, br, vw) = (self.stage.bound_left, self.stage.bound_right, self.view_w);
        let stage_left = bl - vw / 2.0;
        let stage_right = br + vw / 2.0;
        let mut bounced = None;
        let mut shattered: Option<(usize, u32, usize, f32, f32, bool, f32, Option<u32>)> = None;
        let gated = self.lanes.gate;
        for i in 0..self.fighters.len() {
            let f = &mut self.fighters[i];
            if f.destroyed || (f.is_helper && !f.helper_player) {
                continue;
            }
            if gated && f.side != 0 {
                continue;
            }
            if !f.screen_bound {
                f.screen_bound = true;
                continue;
            }
            let lw = if f.facing > 0.0 {
                f.back_width()
            } else {
                f.front_width()
            };
            let rw = if f.facing > 0.0 {
                f.front_width()
            } else {
                f.back_width()
            };
            let minx = left.max(stage_left) + lw;
            let maxx = right.min(stage_right) - rw;
            let world_vx = f.vel.x * f.facing;
            let hit_wall_left = f.pos.x <= minx && world_vx <= 0.0;
            let hit_wall_right = f.pos.x >= maxx && world_vx >= 0.0;
            let at_stage_left =
                f.pos.x <= stage_left + lw + 1.0 && world_vx <= -WALL_SHATTER_SPEED * f.lscl;
            let at_stage_right =
                f.pos.x >= stage_right - rw - 1.0 && world_vx >= WALL_SHATTER_SPEED * f.lscl;
            f.pos.x = f.pos.x.clamp(minx, maxx.max(minx));
            let in_hit = f.move_type == 'H' || f.ghv.fall;
            if (at_stage_left || at_stage_right) && !f.ghv.wall_bounced && in_hit {
                f.ghv.wall_bounced = true;
                let before = f.life;
                f.life = (f.life - WALL_SHATTER_DAMAGE).max(1.0);
                let dealt = (before - f.life).max(0.0);
                let rebound = if at_stage_left {
                    5.8 * f.lscl
                } else {
                    -5.8 * f.lscl
                };
                f.vel.x = rebound * f.facing;
                f.vel.y = -7.2 * f.lscl;
                f.state_type = 'A';
                shattered = Some((
                    i,
                    f.id,
                    f.side,
                    f.pos.x,
                    f.pos.y,
                    at_stage_right,
                    dealt,
                    f.last_hit_by,
                ));
            } else if (hit_wall_left || hit_wall_right)
                && f.ghv.wall_bounce
                && !f.ghv.wall_bounced
                && in_hit
            {
                f.ghv.wall_bounced = true;
                f.vel.x = if hit_wall_left {
                    5.2 * f.lscl
                } else {
                    -5.2 * f.lscl
                };
                f.vel.y = -6.5 * f.lscl;
                f.state_type = 'A';
                bounced = Some(i);
            }
        }
        if let Some((i, fighter, side, x, y, right_wall, bonus_damage, attacker_id)) = shattered {
            if let Some(aid) = attacker_id {
                if let Some(att) = self.fighters.iter_mut().find(|f| f.id == aid) {
                    att.add_power_local(WALL_SHATTER_METER);
                }
            }
            self.shake_with_dir(14, 14.0, 5.5, 0.0, 0);
            self.sound(i, 5, 0, false, Some("hitheavy"));
            self.events.push(Event::WallShatter {
                fighter,
                side,
                x,
                y,
                right_wall,
                bonus_damage,
            });
        } else if let Some(i) = bounced {
            self.shake_with_dir(10, 12.0, 4.5, 0.0, 0);
            self.sound(i, 5, 0, false, Some("hitheavy"));
        }
    }
}
