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

    pub(crate) fn skip_intro_on_request(&mut self, inputs: &[u32]) {
        use crate::input::k;
        if self.round_state != 1 || self.round_no != 1 || self.intro_ticks <= 0 || self.intro_skipped { return; }
        let pressed = self.round_time > 8 && self.global_asserts & crate::flags::gsf::ROUNDNOTSKIP == 0 && inputs.iter().any(|b| b & (k::A | k::B | k::C | k::X | k::Y | k::Z | k::S) != 0);
        if !pressed { return; }
        self.intro_skipped = true;
        self.round_time = self.round_time.max(60 + self.intro_ticks);
        self.explods.clear();
    }

    pub(crate) fn settle_intro_leftovers(&mut self) {
        if self.round_state == 2 && self.round_time == 1 { self.leave_intro_states(); }
    }

    pub(crate) fn leave_intro_states(&mut self) {
        for i in 0..self.fighters.len() {
            let f = &self.fighters[i];
            let forced = self.intro_skipped && !f.ctrl && f.alive && f.state_no < 5000;
            let intro_leftover = forced || f.def.states.contains_key(&190) && std::sync::Arc::ptr_eq(&f.st_def, &f.def) && !f.ctrl && f.alive && f.move_type == 'I' && (0..5000).contains(&f.state_no);
            if !f.is_helper && ((190..=199).contains(&f.state_no) || intro_leftover) {
                let def = self.fighters[i].def.clone();
                self.change_state(i, 0, def);
                self.fighters[i].ctrl = true;
            }
        }
    }
}
