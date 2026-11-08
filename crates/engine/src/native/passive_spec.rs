use crate::passive_time::TimeRule;
use crate::passives::{PassiveRules, RageRule, StackRule, RAGE_MULTIPLIER_CAP, STACK_LIMIT};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RageSpec {
    #[serde(default, rename = "maxMultiplier")]
    pub max_multiplier: Option<f32>,
    #[serde(default)]
    pub exponent: Option<f32>,
    #[serde(default, rename = "startRatio")]
    pub start_ratio: Option<f32>,
    #[serde(default, rename = "fullRatio")]
    pub full_ratio: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StackSpec {
    #[serde(default, rename = "damageGain")]
    pub damage_gain: Option<f32>,
    #[serde(default, rename = "defenceGain")]
    pub defence_gain: Option<f32>,
    #[serde(default, rename = "speedGain")]
    pub speed_gain: Option<f32>,
    #[serde(default, rename = "maxStacks")]
    pub max_stacks: Option<i32>,
    #[serde(default, rename = "decayTicks")]
    pub decay_ticks: Option<i32>,
    #[serde(default, rename = "resetOnRound")]
    pub reset_on_round: Option<bool>,
    #[serde(default)]
    pub specials: Option<bool>,
    #[serde(default)]
    pub supers: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TimeSpec {
    #[serde(default, rename = "maxMultiplier")]
    pub max_multiplier: Option<f32>,
    #[serde(default, rename = "startSeconds")]
    pub start_seconds: Option<f32>,
    #[serde(default, rename = "rampSeconds")]
    pub ramp_seconds: Option<f32>,
    #[serde(default)]
    pub exponent: Option<f32>,
}

impl TimeSpec {
    pub fn rule(&self) -> TimeRule {
        let ticks = |seconds: f32| (seconds.max(0.0) * 60.0).round() as i32;
        TimeRule {
            max_multiplier: self
                .max_multiplier
                .unwrap_or(5.0)
                .clamp(1.0, RAGE_MULTIPLIER_CAP),
            start_ticks: ticks(self.start_seconds.unwrap_or(0.0)),
            ramp_ticks: ticks(self.ramp_seconds.unwrap_or(60.0)).max(1),
            exponent: self.exponent.unwrap_or(1.0).clamp(0.25, 4.0),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PassiveSpec {
    #[serde(default, rename = "meterRegen")]
    pub meter_regen: Option<f32>,
    #[serde(default, rename = "lifeRegen")]
    pub life_regen: Option<f32>,
    #[serde(default)]
    pub rage: Option<RageSpec>,
    #[serde(default)]
    pub stacks: Option<StackSpec>,
    #[serde(default, rename = "enrageTime")]
    pub enrage_time: Option<TimeSpec>,
}

impl RageSpec {
    pub fn rule(&self) -> RageRule {
        let start_ratio = self.start_ratio.unwrap_or(1.0).clamp(0.05, 1.0);
        let full_ratio = self
            .full_ratio
            .unwrap_or(0.01)
            .clamp(0.0, start_ratio - 0.01);
        RageRule {
            max_multiplier: self
                .max_multiplier
                .unwrap_or(4.0)
                .clamp(1.0, RAGE_MULTIPLIER_CAP),
            exponent: self.exponent.unwrap_or(1.0).clamp(0.25, 4.0),
            start_ratio,
            full_ratio,
        }
    }
}

impl StackSpec {
    pub fn rule(&self) -> StackRule {
        StackRule {
            damage_gain: self.damage_gain.unwrap_or(0.05).max(0.0),
            defence_gain: self.defence_gain.unwrap_or(0.0).max(0.0),
            speed_gain: self.speed_gain.unwrap_or(0.0).max(0.0),
            max_stacks: self.max_stacks.unwrap_or(10).clamp(1, STACK_LIMIT),
            decay_ticks: self.decay_ticks.unwrap_or(0).max(0),
            reset_on_round: self.reset_on_round.unwrap_or(true),
            count_specials: self.specials.unwrap_or(true),
            count_supers: self.supers.unwrap_or(true),
        }
    }
}

impl PassiveSpec {
    pub fn rules(&self) -> PassiveRules {
        PassiveRules {
            rage: self.rage.as_ref().map(RageSpec::rule),
            stacks: self.stacks.as_ref().map(StackSpec::rule),
            time: self.enrage_time.as_ref().map(TimeSpec::rule),
        }
    }
}
