mod goal;
mod hazards;
mod pickups;
mod strikes;
mod types;

use crate::matchs::Match;
pub use types::{Breakable, Goal, ObjectEvent, Pickup, PickupSpec};

pub const REHIT_TICKS: i32 = 10;
pub const SHAKE_TICKS: i32 = 9;
pub const PICKUP_ARM_TICKS: i32 = 22;
pub const PICKUP_LIFETIME: i32 = 900;
pub const PICKUP_REACH_X: f32 = 18.0;
pub const PICKUP_REACH_Y: f32 = 26.0;
pub const MAX_EVENTS: usize = 48;
pub const STANDING_STRIKE_HEIGHT: f32 = 100.0;

#[derive(Clone, Debug)]
pub struct StageObjects {
    pub breakables: Vec<Breakable>,
    pub pickups: Vec<Pickup>,
    pub events: Vec<ObjectEvent>,
    pub goal: Goal,
    pub collector_side: usize,
    pub any_collector: bool,
}

impl Default for StageObjects {
    fn default() -> Self {
        StageObjects {
            breakables: Vec::new(),
            pickups: Vec::new(),
            events: Vec::with_capacity(MAX_EVENTS),
            goal: Goal::None,
            collector_side: 0,
            any_collector: false,
        }
    }
}

impl StageObjects {
    pub fn is_idle(&self) -> bool {
        self.breakables.is_empty() && self.pickups.is_empty() && self.goal == Goal::None
    }

    pub fn clear(&mut self) {
        self.breakables.clear();
        self.pickups.clear();
        self.events.clear();
        self.goal = Goal::None;
        self.any_collector = false;
    }

    pub fn add(&mut self, breakable: Breakable) -> usize {
        self.breakables.push(breakable);
        let drops = self.breakables.iter().filter(|b| b.drop.is_some()).count();
        self.pickups
            .reserve(drops.saturating_sub(self.pickups.len()));
        self.breakables.len() - 1
    }

    pub fn start_break_all_goal(&mut self, ticks: i32) {
        self.goal = Goal::BreakAll {
            ticks_left: ticks.max(1),
            total: ticks.max(1),
        };
    }

    pub fn intact_count(&self) -> usize {
        self.breakables.iter().filter(|b| !b.broken).count()
    }

    pub fn goal_ticks_left(&self) -> Option<i32> {
        match self.goal {
            Goal::BreakAll { ticks_left, .. } => Some(ticks_left),
            _ => None,
        }
    }

    pub(crate) fn emit(&mut self, event: ObjectEvent) {
        if self.events.len() < MAX_EVENTS {
            self.events.push(event);
        }
    }
}

impl Match {
    pub(crate) fn stage_objects_step(&mut self) {
        if self.objects.is_idle() {
            return;
        }
        for b in self.objects.breakables.iter_mut() {
            b.rehit = (b.rehit - 1).max(0);
            b.shake = (b.shake - 1).max(0);
            if b.broken {
                b.broken_ticks = b.broken_ticks.saturating_add(1);
            }
        }
        self.strike_breakables();
        self.slam_breakables();
        self.block_on_solid_breakables();
        self.step_pickups();
        self.step_object_goal();
    }

    pub(crate) fn object_in_reach(&self, fighter: usize, lane: f32, depth: f32) -> bool {
        !self.lanes.active
            || (self.lane_of(fighter) - lane).abs() <= crate::lane::LANE_REACH + depth
    }
}
