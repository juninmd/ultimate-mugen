use crate::fighter::Fighter;
use crate::matchs::Match;

pub const CHASE_COST: f32 = 500.0;
pub const CHASE_WINDOW: i32 = 30;
pub const CHASE_GAP: f32 = 38.0;
const RANGE: f32 = 420.0;
const FIRST_MOVE: i32 = 200;
const LAST_MOVE: i32 = 2999;
const AIR_STATE: i32 = 50;

fn juggled(f: &Fighter) -> bool {
    f.alive && !f.destroyed && !f.is_helper && f.move_type == 'H' && (f.state_type == 'A' || f.air_height() < -1.0)
}

fn striker(f: &Fighter) -> bool {
    !f.is_helper
        && !f.destroyed
        && f.alive
        && !f.ikemen.standby
        && f.move_type == 'A'
        && (FIRST_MOVE..=LAST_MOVE).contains(&f.state_no)
        && f.power >= CHASE_COST
        && f.move_hit > 0
        && (1..=CHASE_WINDOW).contains(&f.move_contact)
}

impl Match {
    pub(super) fn chase_target(&self, i: usize) -> Option<usize> {
        let f = &self.fighters[i];
        if self.round_state != 2 || self.pause.is_some() || !striker(f) || !f.cmd.active.contains("UU") {
            return None;
        }
        self.fighters.iter().enumerate()
            .filter(|(_, e)| e.side != f.side && juggled(e) && (e.pos.x - f.pos.x).abs() <= RANGE)
            .min_by(|a, b| (a.1.pos.x - f.pos.x).abs().total_cmp(&(b.1.pos.x - f.pos.x).abs()))
            .map(|(v, _)| v)
    }

    pub(super) fn chase(&mut self, i: usize, victim: usize) {
        let (vx, vy) = (self.fighters[victim].pos.x, self.fighters[victim].pos.y);
        let side = if self.fighters[i].pos.x <= vx { -1.0 } else { 1.0 };
        self.add_power(i, -CHASE_COST);
        let own = self.fighters[i].def.clone();
        self.change_state(i, AIR_STATE, own);
        let f = &mut self.fighters[i];
        f.pos.x = vx + side * CHASE_GAP;
        f.pos.y = vy.min(-1.0);
        f.facing = -side;
        f.vel.x = 0.0;
        f.vel.y = 0.0;
        f.ctrl = true;
        f.hit_pause = 0;
        f.hit_shake = 0;
        f.move_type = 'I';
    }
}
