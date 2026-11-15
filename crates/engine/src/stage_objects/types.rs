use super::STANDING_STRIKE_HEIGHT;
use crate::fighter::V2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickupSpec {
    pub kind: u16,
    pub heal: f32,
    pub power: f32,
    pub score: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Breakable {
    pub kind: u16,
    pub x: f32,
    pub lane: f32,
    pub half_width: f32,
    pub height: f32,
    pub depth: f32,
    pub life: f32,
    pub life_max: f32,
    pub score: f32,
    pub solid: bool,
    pub drop: Option<PickupSpec>,
    pub rehit: i32,
    pub shake: i32,
    pub broken: bool,
    pub broken_ticks: i32,
}

impl Breakable {
    pub fn new(kind: u16, x: f32, lane: f32, half_width: f32, height: f32, life: f32) -> Self {
        Breakable {
            kind,
            x,
            lane,
            half_width,
            height,
            depth: 8.0,
            life,
            life_max: life,
            score: 0.0,
            solid: false,
            drop: None,
            rehit: 0,
            shake: 0,
            broken: false,
            broken_ticks: 0,
        }
    }

    pub fn damage_ratio(&self) -> f32 {
        if self.life_max <= 0.0 {
            return 1.0;
        }
        (1.0 - self.life / self.life_max).clamp(0.0, 1.0)
    }

    pub(crate) fn hurt_box(&self) -> [f32; 4] {
        [
            self.x - self.half_width,
            -self.height.max(STANDING_STRIKE_HEIGHT),
            self.x + self.half_width,
            0.0,
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pickup {
    pub spec: PickupSpec,
    pub pos: V2,
    pub vel: V2,
    pub lane: f32,
    pub age: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObjectEvent {
    Struck {
        index: usize,
        x: f32,
        y: f32,
        lane: f32,
        heavy: bool,
    },
    Broken {
        index: usize,
    },
    Dropped {
        kind: u16,
        x: f32,
        lane: f32,
    },
    Collected {
        spec: PickupSpec,
        x: f32,
        lane: f32,
        fighter: u32,
        healed: f32,
    },
    GoalCleared {
        ticks_left: i32,
    },
    GoalTimedOut,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Goal {
    #[default]
    None,
    BreakAll {
        ticks_left: i32,
        total: i32,
    },
    Cleared {
        ticks_left: i32,
    },
    TimedOut,
}
