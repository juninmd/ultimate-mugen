use crate::fighter::Fighter;
use crate::input::k;
use crate::matchs::Match;

pub const LANE_REACH: f32 = 14.0;
pub const LANE_ALIGN: f32 = 6.0;
const LANE_SPEED: f32 = 1.1;

#[derive(Clone, Debug)]
pub struct LaneRules {
    pub active: bool,
    pub far: f32,
    pub near: f32,
    pub show_foe_hud: bool,
}

impl Default for LaneRules {
    fn default() -> Self {
        LaneRules { active: false, far: -20.0, near: 10.0, show_foe_hud: true }
    }
}

impl Fighter {
    pub(crate) fn lane_step(&mut self, bits: u32, rules: &LaneRules) {
        if !self.ctrl || self.state_type == 'A' || self.hit_pause > 0 {
            return;
        }
        let toward_viewer = (bits & k::LANE_DOWN != 0) as i32 - (bits & k::LANE_UP != 0) as i32;
        self.lane = (self.lane + toward_viewer as f32 * LANE_SPEED).clamp(rules.far, rules.near);
    }
}

impl Match {
    pub fn lane_of(&self, i: usize) -> f32 {
        let f = &self.fighters[i];
        if !f.is_helper {
            return f.lane;
        }
        self.idx(f.root).map(|r| self.fighters[r].lane).unwrap_or(f.lane)
    }

    pub(crate) fn same_lane(&self, a: usize, b: usize) -> bool {
        !self.lanes.active || (self.lane_of(a) - self.lane_of(b)).abs() <= LANE_REACH
    }
}

pub fn ai_lane_bits(m: &Match, me: usize) -> u32 {
    if !m.lanes.active {
        return 0;
    }
    let Some(target) = m.enemy(me, 0) else { return 0 };
    let gap = m.lane_of(target) - m.lane_of(me);
    if gap.abs() <= LANE_ALIGN {
        0
    } else if gap > 0.0 {
        k::LANE_DOWN
    } else {
        k::LANE_UP
    }
}
