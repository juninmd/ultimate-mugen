use crate::fighter::Fighter;

pub const POTENTIAL_TICKS: i32 = 60 * 12;
pub const FURY_TICKS: i32 = 60 * 9;
pub const POTENTIAL_COST: f32 = 0.5;
pub const FURY_COST: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PotentialLevel {
    #[default]
    Idle,
    Potential,
    Fury,
}

impl PotentialLevel {
    pub fn factors(self) -> (f32, f32) {
        match self {
            Self::Idle => (1.0, 1.0),
            Self::Potential => (1.2, 0.9),
            Self::Fury => (1.4, 0.75),
        }
    }

    pub fn duration(self) -> i32 {
        match self {
            Self::Idle => 0,
            Self::Potential => POTENTIAL_TICKS,
            Self::Fury => FURY_TICKS,
        }
    }

    pub fn intensity(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::Potential => 0.6,
            Self::Fury => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PotentialState {
    pub level: PotentialLevel,
    pub time: i32,
    pub potential_used: bool,
    pub fury_used: bool,
}

impl PotentialState {
    pub fn remaining_ratio(&self) -> f32 {
        let total = self.level.duration();
        if total == 0 {
            0.0
        } else {
            (self.time.max(0) as f32 / total as f32).clamp(0.0, 1.0)
        }
    }
}

impl Fighter {
    pub fn set_potential_level(&mut self, level: PotentialLevel) {
        let (old_a, old_d) = self.potential.level.factors();
        let (new_a, new_d) = level.factors();
        self.attack_mul = self.attack_mul / old_a * new_a;
        self.defence_mul = self.defence_mul / old_d * new_d;
        self.potential.level = level;
        self.potential.time = level.duration();
    }

    pub fn tick_potential(&mut self) {
        if self.potential.level == PotentialLevel::Idle {
            return;
        }
        self.potential.time -= 1;
        if self.potential.time <= 0 {
            self.set_potential_level(PotentialLevel::Idle);
        }
    }

    pub fn reset_potential(&mut self) {
        self.set_potential_level(PotentialLevel::Idle);
        self.potential.potential_used = false;
        self.potential.fury_used = false;
    }
}
