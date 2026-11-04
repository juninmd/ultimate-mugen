use crate::fighter::{asf, HitDefData};
use crate::matchs::Match;
use std::sync::Arc;

pub const DIZZY_STATE: i32 = 5300;
pub const GUARD_BREAK_STATE: i32 = 5310;

fn state_exists(m: &Match, i: usize, state_no: i32) -> bool {
    let f = &m.fighters[i];
    f.def.states.contains_key(&state_no) || f.st_def.states.contains_key(&state_no) || m.common.states.contains_key(&state_no)
}

fn scaled(points: f32, mul: f32) -> f32 {
    (points * mul).round()
}

fn keep_red_life_between_life_and_max(m: &mut Match, i: usize) {
    let f = &mut m.fighters[i];
    f.ikemen.red_life = if f.life <= 0.0 { 0.0 } else { f.ikemen.red_life.clamp(f.life, f.life_max) };
}

pub fn apply_hit_points(m: &mut Match, d: usize, hd: &HitDefData, guarded: bool, mul: f32) {
    let f = &mut m.fighters[d];
    let red = if guarded { hd.red_life[1] } else { hd.red_life[0] };
    if !f.has_assert(asf::NOREDLIFEDAMAGE) {
        f.ikemen.red_life -= scaled(red, mul);
    }
    if guarded {
        if !f.has_assert(asf::NOGUARDPOINTSDAMAGE) {
            f.ikemen.guard_points = (f.ikemen.guard_points - scaled(hd.guard_points, mul)).clamp(0.0, f.ikemen.guard_max);
        }
    } else if !f.ikemen.dizzy && !f.has_assert(asf::NODIZZYPOINTSDAMAGE) {
        f.ikemen.dizzy_points = (f.ikemen.dizzy_points - scaled(hd.dizzy_points, mul)).clamp(0.0, f.ikemen.dizzy_max);
    }
    keep_red_life_between_life_and_max(m, d);
}

pub fn enter_stun_states(m: &mut Match, d: usize, guarded: bool) {
    let f = &m.fighters[d];
    if !f.alive || f.life <= 0.0 { return; }
    let airborne = f.state_type == 'A' || f.pos.y < -1.0;
    let (target, due) = if guarded {
        (GUARD_BREAK_STATE, f.ikemen.guard_max > 0.0 && f.ikemen.guard_points <= 0.0 && !f.ikemen.guard_break)
    } else {
        (DIZZY_STATE, f.ikemen.dizzy_max > 0.0 && f.ikemen.dizzy_points <= 0.0 && !f.ikemen.dizzy && !airborne && !f.ghv.fall)
    };
    if !due || !state_exists(m, d, target) { return; }
    if guarded { m.fighters[d].ikemen.guard_break = true; } else { m.fighters[d].ikemen.dizzy = true; }
    m.fighters[d].ctrl = false;
    m.fighters[d].ikemen.stun_by_hit = true;
    let def: Arc<_> = m.fighters[d].def.clone();
    m.change_state(d, target, def);
}

pub fn maintain(m: &mut Match) {
    for i in 0..m.fighters.len() {
        if m.fighters[i].is_helper || m.fighters[i].destroyed { continue; }
        keep_red_life_between_life_and_max(m, i);
        let f = &mut m.fighters[i];
        let recovered = f.move_type != 'H' && f.ikemen.stun_by_hit;
        if recovered && f.state_no != DIZZY_STATE && f.state_no != GUARD_BREAK_STATE { f.ikemen.stun_by_hit = false; }
        if f.ikemen.dizzy && f.state_no != DIZZY_STATE && recovered {
            f.ikemen.dizzy = false;
            f.ikemen.dizzy_points = f.ikemen.dizzy_max;
        }
        if f.ikemen.guard_break && f.state_no != GUARD_BREAK_STATE && recovered {
            f.ikemen.guard_break = false;
            f.ikemen.guard_points = f.ikemen.guard_max;
        }
    }
}

pub fn reset_for_round(m: &mut Match, i: usize) {
    let f = &mut m.fighters[i];
    f.ikemen.red_life = f.life_max;
    f.ikemen.dizzy = false;
    f.ikemen.guard_break = false;
    f.ikemen.stun_by_hit = false;
    f.ikemen.dizzy_points = f.ikemen.dizzy_max;
    f.ikemen.guard_points = f.ikemen.guard_max;
}
