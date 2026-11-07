use crate::matchs::Match;

impl Match {
    pub(crate) fn leave_intro_states(&mut self) {
        for i in 0..self.fighters.len() {
            if !self.fighters[i].is_helper && (190..=199).contains(&self.fighters[i].state_no) {
                let def = self.fighters[i].def.clone();
                self.change_state(i, 0, def);
                self.fighters[i].ctrl = true;
            }
        }
    }
}
