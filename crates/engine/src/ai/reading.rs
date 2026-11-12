use super::Ai;
use crate::fighter::Fighter;
use crate::input::k;
use crate::matchs::Match;

const GUARD_STATES: std::ops::RangeInclusive<i32> = 120..=155;
const JUMP_DECAY: f32 = 0.998;
const JUMP_GAIN: f32 = 0.12;
const GUARD_DECAY: f32 = 0.995;
const GUARD_GAIN: f32 = 0.005;
const FULL_READ_LEVEL: i32 = 8;
const READ_DELAY_PER_LEVEL: i32 = 2;
const PUNISH_REACH: f32 = 1.3;
const PUNISH_COOLDOWN: i32 = 20;
const PUNISH_CHANCE: f32 = 0.45;
const TURTLE_RATE: f32 = 0.04;
const JUMPY_RATE: f32 = 0.15;
const JUMPY_ANTI_AIR_BONUS: f32 = 0.25;
const TURTLE_PRESSURE_BONUS: f32 = 0.15;

#[derive(Default)]
pub(super) struct Profile {
    jump: f32,
    guard: f32,
    was_airborne: bool,
}

impl Profile {
    pub(super) fn observe(&mut self, e: &Fighter) {
        let airborne = e.state_type == 'A' && e.move_type == 'I';
        self.jump *= JUMP_DECAY;
        if airborne && !self.was_airborne {
            self.jump += JUMP_GAIN;
        }
        self.was_airborne = airborne;
        self.guard = self.guard * GUARD_DECAY + if GUARD_STATES.contains(&e.state_no) { GUARD_GAIN } else { 0.0 };
    }

    fn turtles(&self) -> bool {
        self.guard > TURTLE_RATE
    }

    pub(super) fn pressure_bonus(&self) -> f32 {
        if self.turtles() { TURTLE_PRESSURE_BONUS } else { 0.0 }
    }

    pub(super) fn anti_air_bonus(&self) -> f32 {
        if self.jumps_a_lot() { JUMPY_ANTI_AIR_BONUS } else { 0.0 }
    }

    fn jumps_a_lot(&self) -> bool {
        self.jump > JUMPY_RATE
    }
}

pub(super) fn read_delay(level: i32) -> i32 {
    (FULL_READ_LEVEL - level).clamp(0, FULL_READ_LEVEL) * READ_DELAY_PER_LEVEL
}

fn guard_direction(e: &Fighter) -> Option<u32> {
    let flags = &e.hitdef.as_ref()?.guardflag;
    let (low, high) = (flags.contains('L'), flags.contains('H'));
    match (low, high) {
        (true, false) => Some(k::D),
        (false, true) => Some(0),
        _ => None,
    }
}

impl Ai {
    pub(super) fn block_hold(&mut self, e: &Fighter, back: u32) -> u32 {
        let direction = guard_direction(e).unwrap_or(if e.state_type == 'C' || self.r() < 0.3 { k::D } else { 0 });
        back | direction
    }

    pub(super) fn sees_attack(&self, e: &Fighter) -> bool {
        e.move_type == 'A' && e.state_time >= read_delay(self.level)
    }

    pub(super) fn punish_guarded(&mut self, m: &Match, me: usize, ei: usize, near: f32) -> Option<usize> {
        let (f, e) = (&m.fighters[me], &m.fighters[ei]);
        let dist = (e.pos.x - f.pos.x).abs();
        let open = e.move_type == 'A' && e.move_guarded > 0 && !e.ctrl;
        if !open || !f.ctrl || f.state_type == 'A' || dist > near * PUNISH_REACH || self.r() >= PUNISH_CHANCE + self.level as f32 * 0.04 {
            return None;
        }
        let pool = self.attack.clone();
        let picked = self.choose(m, me, ei, &pool, |c| c.steps.len() == 1);
        if picked.is_some() {
            self.cooldown = PUNISH_COOLDOWN;
        }
        picked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_levels_read_attacks_later() {
        assert!(read_delay(3) > read_delay(6));
        assert_eq!(read_delay(FULL_READ_LEVEL), 0);
        assert_eq!(read_delay(12), 0);
    }

    #[test]
    fn profile_notices_jumps_and_turtling() {
        let nc: crate::native::NativeChar = serde_json::from_str(&std::fs::read_to_string("../../assets/chars/kaito.json").unwrap()).unwrap();
        let def = std::sync::Arc::new(crate::native::compile_native(&nc, 1, true));
        let mut f = Fighter::new(0, def, 0, 1);
        let mut p = Profile::default();
        assert!(!p.turtles() && !p.jumps_a_lot());
        for tick in 0..600 {
            f.state_no = 130;
            f.state_type = if tick % 20 < 10 { 'A' } else { 'S' };
            f.move_type = 'I';
            p.observe(&f);
        }
        assert!(p.turtles());
        assert!(p.jumps_a_lot());
    }
}
