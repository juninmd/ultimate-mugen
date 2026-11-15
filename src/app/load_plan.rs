use super::*;

impl App {
    /// Fighters to load for the current bout as (side, slot, registry index): leaders only, or whole teams in parallel mode.
    pub(super) fn load_plan(&self) -> Vec<(usize, usize, usize)> {
        let mut v = vec![];
        match &self.team {
            Some(t) if t.whole_squad() => {
                for k in 0..t.members[0].len().max(t.members[1].len()) {
                    for side in 0..2 {
                        if let Some(&r) = t.members[side].get(k) {
                            v.push((side, k, r));
                        }
                    }
                }
            }
            _ => {
                v.push((0, 0, self.sel.0));
                v.push((1, 0, self.sel.1));
            }
        }
        v
    }

    /// One unit of fight preparation (runs behind the VS screen, one per frame, so the bar animates and the UI never freezes).
    /// Palette for a planned fighter: explicit team/player choice, with the classic duplicate-fighter fallback.
    pub(super) fn pal_for(&self, plan: &[(usize, usize, usize)], st: usize) -> i32 {
        let (side, k, r) = plan[st];
        if let Some(p) = self
            .team_pal
            .get(k)
            .copied()
            .filter(|_| side == 0 && self.team.is_some())
        {
            return p.clamp(1, crate::settings::PALETTE_COUNT as i32);
        }
        let mut pal = self.side_pal(side);
        let dup = (side == 1 && plan.iter().any(|p| p.0 == 0 && p.2 == r))
            || plan.iter().any(|p| p.0 == side && p.1 < k && p.2 == r);
        if dup && pal == self.side_pal(0) {
            pal = (pal % crate::settings::PALETTE_COUNT as i32) + 1;
        }
        pal
    }

    pub(crate) fn side_pal(&self, side: usize) -> i32 {
        self.pal[side.min(1)].clamp(1, crate::settings::PALETTE_COUNT as i32)
    }

    /// Palette shown for a side on the select screen (duplicate fighters get a different palette by default).
    pub(crate) fn pal_for_side(&self, side: usize, picked: [Option<usize>; 2]) -> i32 {
        let pal = self.side_pal(side);
        if side == 1 && picked[0].is_some() && picked[0] == picked[1] && pal == self.side_pal(0) {
            (pal % crate::settings::PALETTE_COUNT as i32) + 1
        } else {
            pal
        }
    }
}
