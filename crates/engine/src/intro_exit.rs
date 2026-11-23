use crate::matchs::Match;

impl Match {
    pub(crate) fn enter_face_off(&mut self, i: usize, taunt: bool) {
        let own = self.fighters[i].def.clone();
        if own.states.contains_key(&190) {
            self.change_state(i, 190, own);
            return;
        }
        let no = if taunt { crate::matchs::ENGINE_STATE_FACE_OFF_TAUNT } else { crate::matchs::ENGINE_STATE_FACE_OFF };
        self.change_engine_state(i, no);
    }

    pub(crate) fn settle_intro_leftovers(&mut self) {
        if self.round_state == 2 && self.round_time == 1 { self.leave_intro_states(); }
    }

    pub(crate) fn leave_intro_states(&mut self) {
        for i in 0..self.fighters.len() {
            let f = &self.fighters[i];
            let intro_leftover = std::sync::Arc::ptr_eq(&f.st_def, &f.def) && !f.ctrl && f.alive && f.move_type == 'I' && (0..5000).contains(&f.state_no);
            if !f.is_helper && ((190..=199).contains(&f.state_no) || intro_leftover) {
                let def = self.fighters[i].def.clone();
                self.change_state(i, 0, def);
                self.fighters[i].ctrl = true;
            }
        }
    }
}
