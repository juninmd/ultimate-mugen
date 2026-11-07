use super::*;

const LEGACY_COST: f32 = 1000.0;
const LEGACY_ATTACK: f32 = 1.3;
const LEGACY_DEFENCE: f32 = 1.25;

fn form_multipliers(def: &CharDef, entered: u8) -> (f32, f32) {
    if def.forms.is_empty() {
        return if entered > 0 { (LEGACY_ATTACK, LEGACY_DEFENCE) } else { (1.0, 1.0) };
    }
    def.forms.iter().take(entered as usize).fold((1.0, 1.0), |(a, d), form| (a * form.attack, d * form.defense))
}

impl Fighter {
    pub fn next_form_cost(&self) -> Option<f32> {
        let forms = &self.base_def.forms;
        let next = self.form as usize;
        if forms.is_empty() { return (next == 0).then_some(LEGACY_COST); }
        forms.get(next).map(|form| form.cost)
    }

    pub fn revert_form(&mut self) {
        if self.form == 0 { return; }
        let (attack, defence) = form_multipliers(&self.base_def, self.form);
        self.attack_mul /= attack;
        self.defence_mul /= defence;
        self.form = 0;
        self.form_time = 0;
        self.def = self.base_def.clone();
        self.anim_def = self.def.clone();
    }
}

impl Match {
    /// Transformation: enters the next form (`CharDef::forms`, or the legacy single awakening). Costs the form's meter, lasts until the round
    /// ends, grants its attack/defence multipliers and a small life refill, and may swap the whole character. Native characters only.
    pub fn try_transform(&mut self, i: usize) -> bool {
        let f = &self.fighters[i];
        let Some(cost) = f.next_form_cost() else { return false };
        if f.is_helper || !f.alive || !f.ctrl || f.state_type == 'A' || f.power < cost || self.round_state != 2 || self.pause.is_some() || !f.def.anims.contains_key(&196) { return false; }
        let id = f.id;
        let (attack, defence, target) = match f.base_def.forms.get(f.form as usize) {
            Some(form) => (form.attack, form.defense, form.target.clone()),
            None => (LEGACY_ATTACK, LEGACY_DEFENCE, None),
        };
        {
            let f = &mut self.fighters[i];
            f.power -= cost; f.form += 1; f.form_time = 0; f.attack_mul *= attack; f.defence_mul *= defence;
            f.life = (f.life + f.life_max * 0.1).min(f.life_max);
            if let Some(target) = target { f.def = target.clone(); f.anim_def = target; }
        }
        self.change_engine_state(i, ENGINE_STATE_AWAKENING);
        self.fighters[i].ctrl = false;
        self.pause = Some(PauseState { by: id, time: 80, move_time: 80, is_super: true, darken: true, unhittable: true, p2_def_mul: 1.0, pause_bg: true, ..Default::default() });
        self.do_shake(40, 5.0);
        self.events.push(Event::Transform { fighter: id });
        true
    }
}
