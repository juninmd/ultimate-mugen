use super::cancel::{flash, landing_state, swallow_start};
use crate::fighter::Fighter;
use crate::matchs::Match;

pub const WHIFF_COST: f32 = 1000.0;
pub const WHIFF_MIN_TICKS: i32 = 10;
pub const WHIFF_MAX_TICKS: i32 = 60;

fn whiffing(f: &Fighter) -> bool {
    !f.is_helper
        && !f.destroyed
        && f.alive
        && !f.ikemen.standby
        && f.move_type == 'A'
        && f.state_type != 'L'
        && f.rush.cooldown == 0
        && f.power >= WHIFF_COST
        && f.move_contact == 0
        && f.move_hit == 0
        && f.move_guarded == 0
        && (WHIFF_MIN_TICKS..=WHIFF_MAX_TICKS).contains(&f.state_time)
}

impl Match {
    pub fn whiff_ready(&self, i: usize) -> bool {
        let f = &self.fighters[i];
        f.rush.requested() && self.round_state == 2 && self.pause.is_none() && whiffing(f)
    }

    pub(super) fn whiff_cancel(&mut self, i: usize) {
        self.add_power(i, -WHIFF_COST);
        let target = landing_state(&self.fighters[i]);
        let own = self.fighters[i].def.clone();
        self.change_state(i, target, own);
        let f = &mut self.fighters[i];
        f.ctrl = true;
        f.hit_pause = 0;
        f.hit_shake = 0;
        if f.state_type != 'A' {
            f.vel.x = 0.0;
        }
        swallow_start(f);
        f.palfx = Some(flash());
        f.rush.begin_whiff();
    }
}
