use crate::fighter::Fighter;

impl Fighter {
    pub(crate) fn lands_by_script(&self) -> bool {
        self.physics == 'N'
            && matches!(self.st_def.source, crate::chardef::Source::Mugen)
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
        self.move_type == 'H'
            || !matches!(self.def.source, crate::chardef::Source::Mugen)
            || self.state_time > 200
    }

    pub(crate) fn hit_state_overstay(&self, free: bool) -> bool {
        free && self.move_type == 'H'
            && (5000..5200).contains(&self.state_no)
            && self.state_no != 5110
            && self.state_time > 900
            && self.hit_pause <= 0
            && self.hit_shake <= 0
    }
}
