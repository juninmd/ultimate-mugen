use crate::fighter::Fighter;
use crate::passive_time::TimeRule;

pub const TOTAL_ATTACK_CAP: f32 = 8.0;
pub const RAGE_MULTIPLIER_CAP: f32 = 6.0;
pub const STACK_DEFENCE_CAP: f32 = 3.0;
pub const STACK_SPEED_CAP: f32 = 1.5;
pub const STACK_LIMIT: i32 = 30;
pub const AURA_FLOOR: f32 = 0.08;
const SPECIAL_STATES: (i32, i32) = (1000, 2000);
const SUPER_STATES: (i32, i32) = (3000, 3900);
const AIR_VARIANT_OFFSET: i32 = 20000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RageRule {
    pub max_multiplier: f32,
    pub exponent: f32,
    pub start_ratio: f32,
    pub full_ratio: f32,
}

impl RageRule {
    pub fn progress(&self, life_ratio: f32) -> f32 {
        let span = (self.start_ratio - self.full_ratio).max(0.0001);
        ((self.start_ratio - life_ratio) / span).clamp(0.0, 1.0)
    }

    pub fn multiplier(&self, life_ratio: f32) -> f32 {
        let shaped = if self.exponent == 1.0 {
            self.progress(life_ratio)
        } else {
            self.progress(life_ratio).powf(self.exponent)
        };
        1.0 + (self.max_multiplier - 1.0) * shaped
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StackRule {
    pub damage_gain: f32,
    pub defence_gain: f32,
    pub speed_gain: f32,
    pub max_stacks: i32,
    pub decay_ticks: i32,
    pub reset_on_round: bool,
    pub count_specials: bool,
    pub count_supers: bool,
}

impl StackRule {
    pub fn counts(&self, state_no: i32) -> bool {
        let state = if state_no >= AIR_VARIANT_OFFSET {
            state_no - AIR_VARIANT_OFFSET
        } else {
            state_no
        };
        let within = |range: (i32, i32)| state >= range.0 && state < range.1;
        (self.count_specials && within(SPECIAL_STATES))
            || (self.count_supers && within(SUPER_STATES))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PassiveRules {
    pub rage: Option<RageRule>,
    pub stacks: Option<StackRule>,
    pub time: Option<TimeRule>,
}

impl PassiveRules {
    pub fn is_empty(&self) -> bool {
        self.rage.is_none() && self.stacks.is_none() && self.time.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PassiveState {
    pub stacks: i32,
    pub idle_ticks: i32,
    pub last_state: i32,
    pub fight_ticks: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PassiveView {
    pub rage: Option<f32>,
    pub rage_multiplier: f32,
    pub time: Option<f32>,
    pub time_multiplier: f32,
    pub stacks: i32,
    pub max_stacks: i32,
    pub aura: f32,
}

impl Fighter {
    fn life_ratio(&self) -> f32 {
        if self.life_max > 0.0 {
            (self.life / self.life_max).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    pub fn rage_multiplier(&self) -> f32 {
        match &self.base_def.passives.rage {
            Some(rule) if self.alive => rule.multiplier(self.life_ratio()),
            _ => 1.0,
        }
    }

    pub fn stack_damage_multiplier(&self) -> f32 {
        match &self.base_def.passives.stacks {
            Some(rule) => 1.0 + rule.damage_gain * self.passive.stacks as f32,
            None => 1.0,
        }
    }

    pub fn time_multiplier(&self) -> f32 {
        match &self.base_def.passives.time {
            Some(rule) if self.alive => rule.multiplier(self.passive.fight_ticks),
            _ => 1.0,
        }
    }

    pub fn effective_attack(&self) -> f32 {
        self.passive_attack() * self.rush.attack_gain()
    }

    fn passive_attack(&self) -> f32 {
        if self.base_def.passives.is_empty() {
            return self.attack_mul;
        }
        let boosted = self.attack_mul
            * (self.rage_multiplier() * self.stack_damage_multiplier() * self.time_multiplier())
                .min(RAGE_MULTIPLIER_CAP * 2.0);
        boosted.min(TOTAL_ATTACK_CAP.max(self.attack_mul))
    }

    pub fn effective_defence(&self) -> f32 {
        let base = self.defence_mul * self.super_defence_mul;
        match &self.base_def.passives.stacks {
            Some(rule) => {
                base * (1.0 + rule.defence_gain * self.passive.stacks as f32).min(STACK_DEFENCE_CAP)
            }
            None => base,
        }
    }

    pub fn passive_speed(&self) -> f32 {
        match &self.base_def.passives.stacks {
            Some(rule) if self.move_type != 'H' => {
                (1.0 + rule.speed_gain * self.passive.stacks as f32).min(STACK_SPEED_CAP)
            }
            _ => 1.0,
        }
    }

    pub fn tick_passive(&mut self, fighting: bool) {
        if fighting && self.alive && self.base_def.passives.time.is_some() {
            self.passive.fight_ticks += 1;
        }
        let Some(rule) = self.base_def.passives.stacks else {
            return;
        };
        if self.passive.last_state != self.state_no {
            self.passive.last_state = self.state_no;
            if rule.counts(self.state_no) && self.alive {
                self.passive.stacks = (self.passive.stacks + 1).min(rule.max_stacks);
                self.passive.idle_ticks = 0;
            }
        }
        if rule.decay_ticks > 0 && self.passive.stacks > 0 {
            self.passive.idle_ticks += 1;
            if self.passive.idle_ticks >= rule.decay_ticks {
                self.passive.stacks -= 1;
                self.passive.idle_ticks = 0;
            }
        }
    }

    pub fn reset_passive_for_round(&mut self) {
        self.passive.fight_ticks = 0;
        let keeps = self
            .base_def
            .passives
            .stacks
            .is_some_and(|rule| !rule.reset_on_round);
        if !keeps {
            self.passive = Default::default();
        }
        self.passive.last_state = self.state_no;
    }

    pub fn passive_view(&self) -> Option<PassiveView> {
        let rules = &self.base_def.passives;
        if rules.is_empty() {
            return None;
        }
        let rage = rules
            .rage
            .filter(|_| self.alive)
            .map(|rule| rule.progress(self.life_ratio()));
        let (stacks, max_stacks) = (
            self.passive.stacks,
            rules.stacks.map_or(0, |rule| rule.max_stacks),
        );
        let stack_share = if max_stacks > 0 {
            stacks as f32 / max_stacks as f32
        } else {
            0.0
        };
        let time = rules
            .time
            .filter(|_| self.alive)
            .map(|rule| rule.progress(self.passive.fight_ticks));
        let aura = rage.unwrap_or(0.0).max(stack_share).max(time.unwrap_or(0.0));
        Some(PassiveView {
            rage,
            rage_multiplier: self.rage_multiplier(),
            time,
            time_multiplier: self.time_multiplier(),
            stacks,
            max_stacks,
            aura: if aura < AURA_FLOOR { 0.0 } else { aura },
        })
    }
}
