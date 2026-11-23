use super::{Event, Match};

pub const CLASH_HITPAUSE: i32 = 12;
pub const CLASH_PUSHBACK: f32 = -3.8;
pub const CLASH_METER: f32 = 60.0;

impl Match {
    pub(super) fn detect_clashes(&mut self) {
        let len = self.fighters.len();
        if len < 2 {
            return;
        }
        let mut pairs = [(0usize, 0usize, 0.0f32, 0.0f32); 4];
        let mut pair_len = 0;

        for a in 0..len {
            let fa = &self.fighters[a];
            if fa.destroyed
                || fa.is_helper
                || fa.ikemen.standby
                || !fa.hitdef_active
                || fa.move_type != 'A'
                || fa.def.source != crate::chardef::Source::Native
            {
                continue;
            }
            let Some(hda) = fa.hitdef.as_ref() else {
                continue;
            };
            if hda.attr_kind.iter().any(|k| k.contains('T')) {
                continue;
            }
            if fa.frame().is_none_or(|f| f.clsn1.is_empty()) {
                continue;
            }
            fa.fill_boxes(1, &mut self.scratch_c1);
            if self.scratch_c1.is_empty() {
                continue;
            }
            for b in (a + 1)..len {
                let fb = &self.fighters[b];
                if fb.destroyed
                    || fb.is_helper
                    || fb.ikemen.standby
                    || fb.side == fa.side
                    || !fb.hitdef_active
                    || fb.move_type != 'A'
                    || fb.def.source != crate::chardef::Source::Native
                    || !self.same_lane(a, b)
                {
                    continue;
                }
                let Some(hdb) = fb.hitdef.as_ref() else {
                    continue;
                };
                if hdb.attr_kind.iter().any(|k| k.contains('T')) {
                    continue;
                }
                if fb.frame().is_none_or(|f| f.clsn1.is_empty()) {
                    continue;
                }
                fb.fill_boxes(1, &mut self.scratch_c2);
                if self.scratch_c2.is_empty() {
                    continue;
                }
                if let Some((cx, cy)) = Self::overlap(&self.scratch_c1, &self.scratch_c2) {
                    if pair_len < pairs.len() {
                        pairs[pair_len] = (a, b, cx, cy);
                        pair_len += 1;
                    }
                }
            }
        }

        for &(a, b, cx, cy) in &pairs[..pair_len] {
            let aid = self.fighters[a].id;
            let bid = self.fighters[b].id;
            for idx in [a, b] {
                let f = &mut self.fighters[idx];
                f.hitdef_active = false;
                f.hit_pause = f.hit_pause.max(CLASH_HITPAUSE);
                f.vel.x = CLASH_PUSHBACK * f.lscl;
                f.add_power_local(CLASH_METER);
            }
            self.shake_with_dir(8, 12.0, 3.5, 0.0, 0);
            self.sound(a, 5, 0, false, Some("guard"));
            self.events.push(Event::Clash {
                a: aid,
                b: bid,
                x: cx,
                y: cy,
            });
        }
    }
}
