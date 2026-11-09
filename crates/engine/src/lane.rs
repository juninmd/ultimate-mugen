use crate::fighter::{Fighter, V2};
use crate::input::k;
use crate::matchs::{Event, Match, StageInfo};

pub const LANE_REACH: f32 = 14.0;
pub const LANE_ALIGN: f32 = 6.0;
pub const SCROLL_SPEED: f32 = 12.0;
const LANE_SPEED: f32 = 1.1;

#[derive(Clone, Debug)]
pub struct LaneRules {
    pub active: bool,
    pub far: f32,
    pub near: f32,
    pub show_foe_hud: bool,
    pub target_x: f32,
    pub bound_span: f32,
}

impl Default for LaneRules {
    fn default() -> Self {
        LaneRules { active: false, far: -20.0, near: 10.0, show_foe_hud: true, target_x: 0.0, bound_span: 0.0 }
    }
}

impl LaneRules {
    pub fn is_scrolling(&self, stage: &StageInfo) -> bool {
        self.active && self.bound_span > 0.0 && stage.bound_left < self.target_x - self.bound_span - 0.5
    }
}

impl Fighter {
    pub(crate) fn lane_step(&mut self, bits: &mut u32, rules: &LaneRules, stage: &StageInfo) {
        if rules.is_scrolling(stage) {
            if self.side == 0 {
                *bits = k::R;
                self.facing = 1.0;
            } else {
                *bits = 0;
                self.ctrl = false;
            }
            return;
        }
        if !self.ctrl || self.state_type == 'A' || self.hit_pause > 0 {
            return;
        }
        let toward_viewer = (*bits & k::LANE_DOWN != 0) as i32 - (*bits & k::LANE_UP != 0) as i32;
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

    pub fn is_lane_scrolling(&self) -> bool {
        self.lanes.is_scrolling(&self.stage)
    }

    pub(crate) fn lane_scroll_step(&mut self) -> bool {
        if self.lanes.active && !self.lanes.show_foe_hud && self.round_state == 3 && self.winner_side == 0 {
            self.announce = None;
            self.slow_mo = self.slow_mo.min(18);
            let settled = self.roots().all(|f| {
                f.ikemen.standby
                    || if f.alive {
                        f.move_type != 'H' && (f.ctrl || f.state_no == 0)
                    } else {
                        f.state_no == 5150 || f.state_no == 5110
                    }
            });
            if (settled && self.round_time > 25) || self.round_time > 240 {
                if !self.match_over {
                    self.match_over = true;
                    self.events.push(Event::MatchOver(0));
                }
            }
        }
        if !self.is_lane_scrolling() {
            return false;
        }
        self.round_time = 0;
        self.timer = if self.round_ticks > 0 { self.round_ticks } else { 99 * 60 };
        let center = self.lanes.target_x;
        let target_left = center - self.lanes.bound_span;
        let hero_target_x = self.stage.p1x;
        let center_lane = 0.0f32.clamp(self.lanes.far, self.lanes.near);
        let next_left = (self.stage.bound_left + SCROLL_SPEED).min(target_left);
        let next_cam = (self.camera.x + SCROLL_SPEED).min(center);
        let mut hero_arrived = true;
        for f in self.fighters.iter_mut().filter(|f| !f.is_helper && !f.destroyed) {
            if f.side == 0 {
                f.pos.x = (f.pos.x + SCROLL_SPEED * 0.75).min(hero_target_x);
                f.pos.y = 0.0;
                f.vel.y = 0.0;
                f.facing = 1.0;
                let dl = center_lane - f.lane;
                f.lane = if dl.abs() <= LANE_SPEED { center_lane } else { f.lane + dl.signum() * LANE_SPEED };
                if f.pos.x < hero_target_x - 0.5 {
                    hero_arrived = false;
                }
            } else {
                f.vel = V2::default();
                f.facing = -1.0;
                f.ctrl = false;
            }
        }
        let arrived = next_left >= target_left - 0.5 && next_cam >= center - 0.5 && hero_arrived;
        if arrived {
            self.stage.bound_left = target_left;
            self.camera.x = center;
            for i in 0..self.fighters.len() {
                if self.fighters[i].is_helper || self.fighters[i].destroyed {
                    continue;
                }
                if self.fighters[i].side == 0 {
                    self.fighters[i].pos.x = hero_target_x;
                    self.fighters[i].lane = center_lane;
                    let d = self.fighters[i].def.clone();
                    self.change_state(i, 0, d);
                }
                self.fighters[i].cmd.active.clear();
                self.fighters[i].vel = V2::default();
                self.fighters[i].ctrl = true;
            }
        } else {
            self.stage.bound_left = next_left.min(target_left - 1.0);
            self.camera.x = next_cam;
        }
        true
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

