use super::Ai;
use crate::fighter::Fighter;
use crate::input::k;
use crate::matchs::{tile_local, Match, ARENA_PLATFORMS};

const FIRST_TIER: f32 = -50.0;

impl Ai {
    pub(super) fn arena_opening(&mut self, m: &Match, me: usize) -> Option<u32> {
        if !m.power_arena {
            return None;
        }
        if m.round_state != 2 {
            self.arena_live = false;
            self.queue.clear();
            self.hold = 0;
            self.hold_ticks = 0;
            self.cooldown = 0;
            return Some(0);
        }
        if self.arena_live {
            return None;
        }
        let foe = m.enemy(me, 0);
        let climbed = foe.is_some_and(|ei| on_platform(&m.fighters[ei]));
        let struck = m.fighters[me].move_type == 'H';
        if foe.is_none() || climbed || struck {
            self.arena_live = true;
            return None;
        }
        self.queue.clear();
        self.hold = 0;
        self.hold_ticks = 0;
        self.cooldown = 0;
        Some(climb_input(&m.fighters[me]))
    }
}

fn on_platform(f: &Fighter) -> bool {
    f.ground_level < -1.0 && (f.pos.y - f.ground_level).abs() <= 0.5
}

fn climb_input(f: &Fighter) -> u32 {
    if !f.ctrl || f.state_type == 'A' || f.ground_level < -1.0 {
        return 0;
    }
    let local = tile_local(f.pos.x);
    if under_first_tier(local) {
        return k::U;
    }
    let center = nearest_first_tier_center(local);
    if center < local {
        k::L
    } else {
        k::R
    }
}

fn under_first_tier(local: f32) -> bool {
    ARENA_PLATFORMS
        .iter()
        .any(|&(left, right, top)| top == FIRST_TIER && local >= left && local <= right)
}

fn nearest_first_tier_center(local: f32) -> f32 {
    ARENA_PLATFORMS
        .iter()
        .filter(|platform| platform.2 == FIRST_TIER)
        .map(|(left, right, _)| (left + right) * 0.5)
        .min_by(|a, b| (*a - local).abs().total_cmp(&(*b - local).abs()))
        .unwrap_or(0.0)
}
