use crate::matchs::Match;

pub const PERFECT_WINDOW: i32 = 6;
const PERFECT_LOCKOUT: i32 = 12;
pub const PERFECT_FLASH: i32 = 12;

impl Match {
    pub(super) fn track_guard_hold(&mut self, i: usize) {
        let held = self.has_command(i, "holdback");
        let r = &mut self.fighters[i].rush;
        r.guard_hold = match (held, r.guard_hold) {
            (true, h) if h < 0 => PERFECT_WINDOW + 1,
            (true, h) => h + 1,
            (false, h) if h > 0 => -PERFECT_LOCKOUT,
            (false, h) if h < 0 => h + 1,
            (false, h) => h,
        };
    }

    pub(crate) fn perfect_guard(&mut self, d: usize) -> bool {
        let f = &self.fighters[d];
        let hold = f.rush.guard_hold;
        let perfect = f.def.rules().perfect_guard
            && !f.is_helper
            && f.state_type != 'A'
            && f.air_height() >= -1.0
            && (1..=PERFECT_WINDOW).contains(&hold);
        if perfect {
            let r = &mut self.fighters[d].rush;
            r.perfect = PERFECT_FLASH;
            r.guard_hold = 0;
        }
        perfect
    }
}
