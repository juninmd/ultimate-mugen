use super::EggSprite;
use crate::audio::Audio;
use ume_engine::matchs::Match;

pub(crate) const MAX_ACTIVE: usize = 3;
const GLOBAL_COOLDOWN_TICKS: i64 = 55;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EggStyle {
    CornerPop,
    FloorRoll,
    Flyby,
    TopRibbon,
    SpeechBubble,
}

pub(crate) struct ActiveEgg {
    pub style: EggStyle,
    pub sprite: EggSprite,
    pub title: String,
    pub sub: String,
    pub speaker: String,
    pub color: [f32; 3],
    pub side: usize,
    pub t: f32,
    pub dur: f32,
    pub flip: bool,
}

#[derive(Default)]
pub(crate) struct FightTracker {
    pub last_tick: i64,
    pub last_round: i32,
    pub fired_match: u128,
    pub fired_round: u128,
    pub cooldown_until: i64,
    pub active: Vec<ActiveEgg>,
    pub prev_life: [f32; 2],
    pub prev_power: [f32; 2],
    pub prev_state: [i32; 2],
    pub prev_type: [char; 2],
    pub prev_form: [u8; 2],
    pub last_hit_tick: [i64; 2],
    pub taunt_tick: [i64; 2],
    pub super_tick: [i64; 2],
    pub jump_tick: [i64; 2],
    pub sync_jumps: u8,
    pub crouch_count: [u8; 2],
    pub crouch_window: [i64; 2],
    pub guard_streak: [u8; 2],
    pub still_ticks: i32,
    pub corner_ticks: [i32; 2],
    pub awakened: [bool; 2],
}

impl FightTracker {
    pub(crate) fn sync_match(&mut self, m: &Match) {
        if m.game_time < self.last_tick || (m.game_time <= 2 && self.last_tick > 10) {
            *self = Self::default();
        }
        if m.round_no != self.last_round {
            self.last_round = m.round_no;
            self.fired_round = 0;
            self.guard_streak = [0; 2];
            self.crouch_count = [0; 2];
            self.still_ticks = 0;
            self.corner_ticks = [0; 2];
            self.sync_jumps = 0;
        }
        self.last_tick = m.game_time;
    }

    pub(crate) fn record_snapshot(&mut self, m: &Match) {
        for side in 0..2 {
            if let Some(f) = m.roots().find(|r| r.side == side) {
                self.prev_life[side] = f.life;
                self.prev_power[side] = f.power;
                self.prev_state[side] = f.state_no;
                self.prev_type[side] = f.state_type;
                self.prev_form[side] = f.form;
            }
        }
    }

    pub(crate) fn advance(&mut self, dt: f32) {
        for egg in &mut self.active {
            egg.t += dt;
        }
        self.active.retain(|egg| egg.t < egg.dur);
    }

    pub(crate) fn has_fired(&self, bit: u8, per_round: bool) -> bool {
        let mask = 1u128 << (bit as u32);
        if per_round {
            self.fired_round & mask != 0
        } else {
            self.fired_match & mask != 0
        }
    }

    pub(crate) fn trigger(&mut self, bit: u8, per_round: bool, egg: ActiveEgg, audio: &mut Audio, sfx: &'static str) -> bool {
        if self.has_fired(bit, per_round) || self.last_tick < self.cooldown_until {
            return false;
        }
        let mask = 1u128 << (bit as u32);
        if per_round {
            self.fired_round |= mask;
        } else {
            self.fired_match |= mask;
        }
        self.cooldown_until = self.last_tick + GLOBAL_COOLDOWN_TICKS;
        if self.active.len() >= MAX_ACTIVE {
            self.active.remove(0);
        }
        self.active.push(egg);
        if !sfx.is_empty() {
            audio.sfx(sfx);
        }
        true
    }
}
