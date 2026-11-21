use super::*;

impl Match {
    pub(super) fn run_minus4_paused(&mut self, i: usize) {
        if self.power_arena && self.fighters[i].state_no == ENGINE_STATE_ARENA_EXIT { return; }
        let own = self.fighters[i].def.clone();
        if own.states.contains_key(&-4) {
            self.run_state(i, &own, -4, true);
        }
    }

    pub(super) fn run_states(&mut self, i: usize) {
        if self.power_arena && self.fighters[i].state_no == ENGINE_STATE_ARENA_EXIT { return; }
        let in_pause = self.fighters[i].hit_pause > 0 || self.fighters[i].hit_shake > 0;
        {
            let f = &mut self.fighters[i];
            f.asserts = if in_pause {
                f.asserts & !asf::ANIMATEHITPAUSE
            } else {
                0
            };
            if f.ikemen.standby {
                f.asserts |= asf::INVISIBLE;
            }
        }
        let own = self.fighters[i].def.clone();
        let custom = !Arc::ptr_eq(&self.fighters[i].st_def, &own)
            && !Arc::ptr_eq(&self.fighters[i].st_def, &self.common);
        let helper = self.fighters[i].is_helper;
        let id = self.fighters[i].id;
        if own.states.contains_key(&-4) {
            self.run_state(i, &own, -4, in_pause);
        }
        let Some(_i) = self.idx(id) else { return };
        if !helper && !custom {
            self.run_state(i, &own, -3, in_pause);
        }
        let Some(_i) = self.idx(id) else { return };
        if !helper {
            self.run_state(i, &own, -2, in_pause);
            let Some(_i) = self.idx(id) else { return };
        }
        if (!helper || self.fighters[i].key_ctrl) && !custom {
            self.run_state(i, &own, -1, in_pause);
            let Some(_i) = self.idx(id) else { return };
            if !in_pause {
                self.builtin_transitions(i);
            }
        }
        for _ in 0..16 {
            let Some(_i) = self.idx(id) else { return };
            let (same, no) = {
                let f = &self.fighters[i];
                (Arc::ptr_eq(&f.st_def, &own), f.state_no)
            };
            let owner;
            let st_ref = if same {
                &own
            } else {
                owner = self.fighters[i].st_def.clone();
                &owner
            };
            if !self.run_state(i, st_ref, no, in_pause) {
                break;
            }
        }
    }
}
