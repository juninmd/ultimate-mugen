use super::Match;
use crate::flags::asf;

impl Match {
    pub(super) fn check_air_dash(&mut self, i: usize) -> bool {
        let f = &self.fighters[i];
        if f.state_type != 'A' || !f.ctrl || f.has_assert(asf::NOHARDCODEDKEYS) {
            return false;
        }
        let max_dashes = f.c("movement.airdash.num", 1.0) as i32;
        if f.air_dashes >= max_dashes || f.pos.y >= -15.0 * f.lscl {
            return false;
        }
        let is_ff = f.cmd.active.contains("FF");
        let is_bb = f.cmd.active.contains("BB");
        if is_ff {
            self.fighters[i].air_dashes += 1;
            let own = self.fighters[i].def.clone();
            self.change_state(i, 110, own);
            return true;
        }
        if is_bb {
            self.fighters[i].air_dashes += 1;
            let own = self.fighters[i].def.clone();
            self.change_state(i, 115, own);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{common_def, flat_stage};
    use crate::matchs::Match;

    #[test]
    fn air_dash_and_retreat_transitions_and_limits() {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(common_def(), 0, 0, 1);
        m.round_state = 2;

        let f = &mut m.fighters[0];
        f.state_type = 'A';
        f.state_no = 50;
        f.ctrl = true;
        f.pos.y = -60.0;
        f.air_dashes = 0;
        f.cmd.active.insert("FF".into());

        let transitioned = m.check_air_dash(0);
        assert!(transitioned);
        assert_eq!(m.fighters[0].state_no, 110);
        assert_eq!(m.fighters[0].air_dashes, 1);

        let second_dash = m.check_air_dash(0);
        assert!(!second_dash);
        assert_eq!(m.fighters[0].state_no, 110);

        m.fighters[0].air_dashes = 0;
        m.fighters[0].pos.y = -50.0;
        m.fighters[0].state_type = 'A';
        m.fighters[0].ctrl = true;
        m.fighters[0].cmd.active.clear();
        m.fighters[0].cmd.active.insert("BB".into());

        let retreat = m.check_air_dash(0);
        assert!(retreat);
        assert_eq!(m.fighters[0].state_no, 115);
        assert_eq!(m.fighters[0].air_dashes, 1);
    }
}
