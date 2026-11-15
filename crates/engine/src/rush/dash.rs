use crate::fighter::Fighter;
use crate::matchs::Match;

pub const DASH_COST: f32 = 250.0;
pub const DASH_WINDOW: i32 = 20;
const FORWARD_DASH_STATE: i32 = 100;
const BACK_DASH_STATE: i32 = 105;
const FIRST_NORMAL: i32 = 200;
const LAST_NORMAL: i32 = 699;

fn connected_normal(f: &Fighter) -> bool {
    !f.is_helper
        && !f.destroyed
        && f.alive
        && !f.ikemen.standby
        && f.move_type == 'A'
        && matches!(f.state_type, 'S' | 'C')
        && (FIRST_NORMAL..=LAST_NORMAL).contains(&f.state_no)
        && f.power >= DASH_COST
        && (f.move_hit > 0 || f.move_guarded > 0)
        && (1..=DASH_WINDOW).contains(&f.move_contact)
}

impl Match {
    pub(super) fn dash_cancel_target(&self, i: usize) -> Option<i32> {
        let f = &self.fighters[i];
        if self.round_state != 2 || self.pause.is_some() || !connected_normal(f) {
            return None;
        }
        if f.cmd.active.contains("FF") {
            Some(FORWARD_DASH_STATE)
        } else if f.cmd.active.contains("BB") {
            Some(BACK_DASH_STATE)
        } else {
            None
        }
    }

    pub(super) fn dash_cancel(&mut self, i: usize, target: i32) {
        self.add_power(i, -DASH_COST);
        let own = self.fighters[i].def.clone();
        self.change_state(i, target, own);
        let f = &mut self.fighters[i];
        f.hit_pause = 0;
        f.hit_shake = 0;
    }
}
