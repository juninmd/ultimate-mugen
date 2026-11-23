use crate::fighter::Fighter;

const AIRBORNE_HIT_LIMIT: i32 = 360;

impl Fighter {
    pub(crate) fn lands_by_script(&self) -> bool {
        self.physics == 'N'
            && self.st_def.rules().script_landing
            && self.st_def.states.get(&self.state_no).is_some_and(|s| {
                s.controllers.iter().any(|c| {
                    matches!(c.kind.as_str(), "selfstate" | "changestate")
                        && c.trigger_all
                            .iter()
                            .chain(c.triggers.iter().flatten())
                            .any(|e| {
                                let d = format!("{:?}", e.node).to_ascii_lowercase();
                                d.contains("\"pos\"") && d.contains("comp: some('y')")
                            })
                })
            })
    }

    pub(crate) fn counts_as_stuck(&self) -> bool {
        let scripted_attack = self.move_type == 'A' && self.anim_total_time > 0 && self.state_time <= self.anim_total_time + 60;
        self.move_type == 'H' || !scripted_attack && (!self.def.rules().script_landing || self.state_time > 200)
    }

    pub(crate) fn hit_state_overstay(&self, free: bool) -> bool {
        free && self.move_type == 'H'
            && (5000..5200).contains(&self.state_no)
            && self.state_no != 5110
            && self.state_time > if self.pos.y < self.ground_level - 1.0 { AIRBORNE_HIT_LIMIT } else { 900 }
            && self.hit_pause <= 0
            && self.hit_shake <= 0
    }
}
