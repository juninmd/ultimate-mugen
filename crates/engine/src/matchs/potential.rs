use super::*;

pub const POTENTIAL_TICKS: i32 = 900;
pub const FURY_TICKS: i32 = 600;
pub const POTENTIAL_ATTACK: f32 = 1.25;
pub const POTENTIAL_DEFENCE: f32 = 1.2;
pub const FURY_ATTACK: f32 = 1.5;
pub const FURY_DEFENCE: f32 = 1.35;

impl Fighter {
    pub fn potential_active(&self) -> bool {
        self.passive.potential_time > 0
    }

    pub fn potential_attack_mul(&self) -> f32 {
        match (self.potential_active(), self.passive.potential_uses) {
            (false, _) => 1.0,
            (true, 1) => POTENTIAL_ATTACK,
            (true, _) => FURY_ATTACK,
        }
    }

    pub fn potential_defence_mul(&self) -> f32 {
        match (self.potential_active(), self.passive.potential_uses) {
            (false, _) => 1.0,
            (true, 1) => POTENTIAL_DEFENCE,
            (true, _) => FURY_DEFENCE,
        }
    }

    pub fn tick_potential(&mut self, fighting: bool) {
        if fighting && self.alive && self.potential_active() {
            self.passive.potential_time -= 1;
        }
    }
}

impl Match {
    /// Generic bonus transformation: the first use is "Potencial Liberado", the second "Modo Fúria" (stronger, shorter). Needs a full
    /// power bar, spends it, and works only for characters without their own forms. Multipliers are applied at damage time, never stored on attack_mul.
    pub fn try_potential(&mut self, i: usize) -> bool {
        let f = &self.fighters[i];
        if f.is_helper || !f.alive || !f.ctrl || f.state_type == 'A' || f.potential_active() || self.round_state != 2 || self.pause.is_some() { return false; }
        if !f.base_def.forms.is_empty() || f.def.anims.contains_key(&196) || f.power < f.power_max || f.passive.potential_uses >= 2 { return false; }
        let id = f.id;
        self.fighters[i].power = 0.0;
        self.fighters[i].passive.potential_uses += 1;
        self.fighters[i].passive.potential_time = if self.fighters[i].passive.potential_uses >= 2 { FURY_TICKS } else { POTENTIAL_TICKS };
        self.events.push(Event::Transform { fighter: id });
        true
    }
}
