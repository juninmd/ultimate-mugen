use super::Fighter;
use crate::chardef::Source;

impl Fighter {
    pub(super) fn anim_time_value(&self) -> i32 {
        if self.anim_total_time >= 0 {
            return self.anim_elapsed - self.anim_total_time;
        }
        if self.anim_def.source != Source::Mugen {
            return -1;
        }
        let finite: i32 = self.anim().map_or(0, |a| {
            a.frames
                .iter()
                .take_while(|f| f.time >= 0)
                .map(|f| f.time)
                .sum()
        });
        (self.anim_elapsed - finite).min(0)
    }
}
