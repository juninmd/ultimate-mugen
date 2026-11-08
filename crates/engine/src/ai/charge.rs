use crate::matchs::Match;
use crate::native::{CHARGE_COMMAND, CHARGE_STATE};

const MINIMUM_LEVEL: i32 = 3;
const CHARGE_BELOW_POWER: f32 = 700.0;
const SAFE_DISTANCE: f32 = 190.0;

pub(super) fn wants_to_charge(m: &Match, me: usize, enemy: usize, level: i32) -> bool {
    let (f, e) = (&m.fighters[me], &m.fighters[enemy]);
    let ready = f.ctrl && f.state_type == 'S' || f.state_no == CHARGE_STATE;
    level >= MINIMUM_LEVEL
        && ready
        && f.power < CHARGE_BELOW_POWER
        && (e.pos.x - f.pos.x).abs() > SAFE_DISTANCE
        && e.move_type != 'A'
        && f.def.commands.iter().any(|c| c.name == CHARGE_COMMAND)
}
