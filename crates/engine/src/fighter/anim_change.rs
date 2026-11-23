use super::*;

impl Fighter {
    /// Change animation; `from` is the CharDef providing the animation (own, or state owner for ChangeAnim2).
    pub fn change_anim(&mut self, no: i32, from: &Arc<CharDef>, elem: i32) {
        let own = self.def.clone();
        let from = if !Arc::ptr_eq(from, &own) && from.id == own.id { &own } else { from };
        if no != self.anim_no { self.prev_anim = self.anim_no; }
        let mut real = if from.anims.contains_key(&no) { Some(no) } else { None };
        if real.is_none() { real = crate::anim_fallback::anim_fallback(no).iter().copied().find(|a| from.anims.contains_key(a)); }
        let real = match real {
            Some(r) => r,
            None => {
                if self.anim().is_some() && Arc::ptr_eq(from, &self.anim_def) { self.anim_no = no; return; }
                if Arc::ptr_eq(from, &self.def) && from.anims.contains_key(&0) { 0 } else { return }
            }
        };
        if !Arc::ptr_eq(from, &self.anim_def) {
            self.anim_def = from.clone();
        }
        self.anim_no = no;
        self.anim_real = real;
        let (tot, ls) = from.anims.get(&real).map(|a| (a.total_time, a.loop_start)).unwrap_or((0, 0));
        self.anim_total_time = tot;
        self.anim_loop_start = ls;
        self.elem = 0; self.elem_time = 0; self.anim_elapsed = 0;
        if elem > 1 {
            self.anim_elapsed = self.elem_start(elem as usize);
            let n = self.anim().map(|a| a.frames.len()).unwrap_or(1);
            self.elem = ((elem - 1) as usize).min(n - 1);
        }
        self.anim_changed = true;
    }
    pub fn change_anim_own(&mut self, no: i32, elem: i32) {
        if no != self.anim_no { self.prev_anim = self.anim_no; }
        let from = &self.def;
        let mut real = if from.anims.contains_key(&no) { Some(no) } else { None };
        if real.is_none() { real = crate::anim_fallback::anim_fallback(no).iter().copied().find(|a| from.anims.contains_key(a)); }
        let real = match real {
            Some(r) => r,
            None => {
                if self.anim().is_some() && Arc::ptr_eq(from, &self.anim_def) { self.anim_no = no; return; }
                if from.anims.contains_key(&0) { 0 } else { return }
            }
        };
        if !Arc::ptr_eq(from, &self.anim_def) {
            self.anim_def = from.clone();
        }
        self.anim_no = no;
        self.anim_real = real;
        let (tot, ls) = from.anims.get(&real).map(|a| (a.total_time, a.loop_start)).unwrap_or((0, 0));
        self.anim_total_time = tot;
        self.anim_loop_start = ls;
        self.elem = 0; self.elem_time = 0; self.anim_elapsed = 0;
        if elem > 1 {
            self.anim_elapsed = self.elem_start(elem as usize);
            let n = self.anim().map(|a| a.frames.len()).unwrap_or(1);
            self.elem = ((elem - 1) as usize).min(n - 1);
        }
        self.anim_changed = true;
    }
}
