use super::*;
use crate::potential::{PotentialLevel, FURY_COST, GIFT_TRANSFORMS, MAX_TRANSFORMS, POTENTIAL_COST};

impl Match {
    /// Generic transformation for characters without their own forms (imported MUGEN/UME): first "Potencial Liberado"
    /// (half a bar, once per round), then "Modo Fúria" while it lasts (a full bar, once per round). Only the stats change. A character has at most MAX_TRANSFORMS transformations in total, own forms included, so gifts are capped by the room left.
    pub fn try_potential(&mut self, i: usize) -> bool {
        let f = &self.fighters[i];
        if f.is_helper || !f.alive || !f.ctrl || f.state_type == 'A' || (f.form as usize) < f.form_count() || self.round_state != 2 || self.pause.is_some() { return false; }
        let budget = MAX_TRANSFORMS.saturating_sub(f.form_count()).min(GIFT_TRANSFORMS);
        let (level, cost) = match f.potential.level {
            PotentialLevel::Idle if budget >= 1 && !f.potential.potential_used => (PotentialLevel::Potential, f.power_max * POTENTIAL_COST),
            PotentialLevel::Potential if budget >= 2 && !f.potential.fury_used => (PotentialLevel::Fury, f.power_max * FURY_COST),
            _ => return false,
        };
        if f.power < cost { return false; }
        let id = f.id;
        {
            let f = &mut self.fighters[i];
            f.power = (f.power - cost).max(0.0);
            match level {
                PotentialLevel::Fury => f.potential.fury_used = true,
                _ => f.potential.potential_used = true,
            }
            f.set_potential_level(level);
        }
        self.events.push(Event::Transform { fighter: id });
        true
    }
}
