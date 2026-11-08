use super::Match;
use crate::fighter::asf;

impl Match {
    pub(super) fn builtin_walk(&mut self, i: usize, s: i32) {
        let held = |m: &Match, n: &str| m.fighters[i].cmd.active.contains(n);
        let moving = held(self, "holdfwd") || held(self, "holdback");
        let enter =
            moving && s != 20 && (s == 0 || s == 52) && !self.fighters[i].has_assert(asf::NOWALK);
        let leave = !moving && s == 20;
        if enter || leave {
            let own = self.fighters[i].def.clone();
            self.change_state(i, if enter { 20 } else { 0 }, own);
        }
    }
}

impl Match {
    pub(super) fn builtin_guard_exit(&mut self, i: usize, s: i32, guard_input: bool) -> bool {
        if !matches!(s, 130 | 131) || guard_input && self.in_guard_dist(i) {
            return false;
        }
        let own = self.fighters[i].def.clone();
        self.change_state(i, 140, own);
        true
    }
}
