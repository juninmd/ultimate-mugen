use super::Fighter;
use crate::chardef::{AnimFrame, Animation, CharDef};
use std::sync::Arc;

impl Fighter {
    pub fn anim(&self) -> Option<&Animation> {
        self.anim_def.anims.get(&self.anim_real)
    }

    pub fn frame(&self) -> Option<&AnimFrame> {
        self.anim().and_then(|a| a.frames.get(self.elem))
    }

    pub fn anim_time_trig(&self) -> i32 {
        self.anim_time_value()
    }

    pub fn elem_start(&self, n: usize) -> i32 {
        if n <= 1 {
            return 0;
        }
        let Some(a) = self.anim() else { return 0 };
        a.frames.iter().take(n - 1).map(|f| f.time.max(0)).sum()
    }

    pub fn anim_elem_started(&self, n: i32) -> bool {
        self.elem as i32 + 1 == n
            && self.elem_time == 0
            && (n > 1
                || self.anim_elapsed == 0
                || (self.anim_loop_start == 0 && self.anim_elapsed != self.anim_total_time))
    }

    pub fn anim_elem_time(&self, n: i32) -> Option<i32> {
        let a = self.anim()?;
        if n < 1 || n as usize > a.frames.len() {
            return None;
        }
        let start = if n == 1 {
            0
        } else {
            a.frames
                .iter()
                .take(n as usize - 1)
                .map(|f| f.time.max(0))
                .sum()
        };
        Some(self.anim_elapsed - start)
    }

    pub fn anim_elem_no_at(&self, off: i32) -> i32 {
        let Some(a) = self.anim() else { return 0 };
        let mut t = self.anim_elapsed + off;
        let tot = a.total_time;
        if tot > 0 && t > tot {
            let ls = self.elem_start(self.anim_loop_start + 1);
            let ll = tot - ls;
            t = if ll > 0 { ls + (t - ls) % ll } else { ls };
        }
        let mut acc = 0;
        for (i, f) in a.frames.iter().enumerate() {
            if f.time < 0 || t < acc + f.time {
                return i as i32 + 1;
            }
            acc += f.time;
        }
        a.frames.len() as i32
    }

    pub fn change_anim(&mut self, no: i32, from: &Arc<CharDef>, elem: i32) {
        let own = self.def.clone();
        let from = if !Arc::ptr_eq(from, &own) && from.id == own.id {
            &own
        } else {
            from
        };
        if no != self.anim_no {
            self.prev_anim = self.anim_no;
        }
        let mut real = if from.anims.contains_key(&no) {
            Some(no)
        } else {
            None
        };
        if real.is_none() {
            real = crate::anim_fallback::anim_fallback(no)
                .iter()
                .copied()
                .find(|a| from.anims.contains_key(a));
        }
        let real = match real {
            Some(r) => r,
            None => {
                if self.anim().is_some() && Arc::ptr_eq(from, &self.anim_def) {
                    self.anim_no = no;
                    return;
                }
                if Arc::ptr_eq(from, &self.def) && from.anims.contains_key(&0) {
                    0
                } else {
                    return;
                }
            }
        };
        if !Arc::ptr_eq(from, &self.anim_def) {
            self.anim_def = from.clone();
        }
        self.apply_anim_reset(no, real, from, elem);
    }

    pub fn change_anim_own(&mut self, no: i32, elem: i32) {
        if no != self.anim_no {
            self.prev_anim = self.anim_no;
        }
        let from = self.def.clone();
        let mut real = if from.anims.contains_key(&no) {
            Some(no)
        } else {
            None
        };
        if real.is_none() {
            real = crate::anim_fallback::anim_fallback(no)
                .iter()
                .copied()
                .find(|a| from.anims.contains_key(a));
        }
        let real = match real {
            Some(r) => r,
            None => {
                if self.anim().is_some() && Arc::ptr_eq(&from, &self.anim_def) {
                    self.anim_no = no;
                    return;
                }
                if from.anims.contains_key(&0) {
                    0
                } else {
                    return;
                }
            }
        };
        if !Arc::ptr_eq(&from, &self.anim_def) {
            self.anim_def = from.clone();
        }
        self.apply_anim_reset(no, real, &from, elem);
    }

    fn apply_anim_reset(&mut self, no: i32, real: i32, from: &Arc<CharDef>, elem: i32) {
        self.anim_no = no;
        self.anim_real = real;
        let (tot, ls) = from
            .anims
            .get(&real)
            .map(|a| (a.total_time, a.loop_start))
            .unwrap_or((0, 0));
        self.anim_total_time = tot;
        self.anim_loop_start = ls;
        self.elem = 0;
        self.elem_time = 0;
        self.anim_elapsed = 0;
        if elem > 1 {
            self.anim_elapsed = self.elem_start(elem as usize);
            let n = self.anim().map(|a| a.frames.len()).unwrap_or(1);
            self.elem = ((elem - 1) as usize).min(n - 1);
        }
        self.anim_changed = true;
    }

    pub fn advance_anim(&mut self) {
        if self.anim_changed {
            self.anim_changed = false;
            return;
        }
        let Some(a) = self.anim_def.anims.get(&self.anim_real) else {
            return;
        };
        if a.frames.is_empty() {
            return;
        }
        let len = a.frames.len();
        let ft = a.frames[self.elem].time;
        if self.anim_total_time >= 0 && self.anim_elapsed >= self.anim_total_time {
            self.anim_elapsed = if self.anim_loop_start == 0 {
                0
            } else {
                a.frames
                    .iter()
                    .take(self.anim_loop_start)
                    .map(|f| f.time.max(0))
                    .sum()
            };
        }
        self.anim_elapsed += 1;
        if ft < 0 {
            return;
        }
        self.elem_time += 1;
        if self.elem_time >= ft {
            self.elem_time = 0;
            self.elem += 1;
            if self.elem >= len {
                self.elem = self.anim_loop_start;
            }
            let mut guard = 0;
            while a.frames[self.elem].time == 0 && guard < len {
                guard += 1;
                self.elem += 1;
                if self.elem >= len {
                    self.elem = self.anim_loop_start;
                }
            }
        }
    }
}
