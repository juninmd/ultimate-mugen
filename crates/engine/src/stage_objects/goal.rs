use super::{Goal, ObjectEvent};
use crate::matchs::Match;

const BODY_HALF_WIDTH: f32 = 12.0;

impl Match {
    pub(super) fn block_on_solid_breakables(&mut self) {
        for bi in 0..self.objects.breakables.len() {
            let b = &self.objects.breakables[bi];
            if !b.solid {
                continue;
            }
            let (x, reach, lane, depth, top) = (
                b.x,
                b.half_width + BODY_HALF_WIDTH,
                b.lane,
                b.depth,
                -b.height,
            );
            for fi in 0..self.fighters.len() {
                let f = &self.fighters[fi];
                if f.is_helper
                    || f.destroyed
                    || f.ikemen.standby
                    || f.pos.y < top
                    || !self.object_in_reach(fi, lane, depth)
                {
                    continue;
                }
                let gap = f.pos.x - x;
                if gap.abs() >= reach {
                    continue;
                }
                let side = if gap == 0.0 { -f.facing } else { gap.signum() };
                self.fighters[fi].pos.x = x + side * reach;
            }
        }
    }

    pub(super) fn step_object_goal(&mut self) {
        let Goal::BreakAll { ticks_left, total } = self.objects.goal else {
            return;
        };
        if self.round_state != 2 {
            return;
        }
        if self.objects.intact_count() == 0 {
            self.objects.goal = Goal::Cleared { ticks_left };
            self.objects.emit(ObjectEvent::GoalCleared { ticks_left });
            self.timer = 0;
            return;
        }
        let left = ticks_left - 1;
        if left <= 0 {
            self.objects.goal = Goal::TimedOut;
            self.objects.emit(ObjectEvent::GoalTimedOut);
            self.timer = 0;
            return;
        }
        self.objects.goal = Goal::BreakAll {
            ticks_left: left,
            total,
        };
        self.timer = left + 1;
    }
}
