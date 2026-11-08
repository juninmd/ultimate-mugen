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

    pub fn form_count(&self) -> usize {
        let forms = &self.base_def.forms;
        if !forms.is_empty() { return forms.len(); }
        usize::from(self.base_def.anims.contains_key(&196))
    }

    pub fn active_form_name(&self) -> &str {
        if self.form == 0 {
            return &self.def.form_name;
        }
        self.base_def
            .forms
            .get((self.form as usize).saturating_sub(1))
            .map(|f| f.name.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.def.form_name)
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
        let defeated_once = f.def.boss && self.wins[1 - f.side.min(1)] >= 1;
        let target_form = if defeated_once {
            f.base_def.forms.len().max(1) as u8
        } else {
            f.form + 1
        };
        let (old_a, old_d) = form_multipliers(&f.base_def, f.form);
        let (new_a, new_d) = form_multipliers(&f.base_def, target_form);
        let target = f.base_def.forms.get((target_form as usize).saturating_sub(1)).and_then(|fm| fm.target.clone());
        {
            let f = &mut self.fighters[i];
            f.power = (f.power - cost).max(0.0);
            f.form = target_form;
            f.form_time = 0;
            f.attack_mul = f.attack_mul / old_a * new_a;
            f.defence_mul = f.defence_mul / old_d * new_d;
            let heal = if defeated_once { 0.2 } else { 0.1 };
            f.life = (f.life + f.life_max * heal).min(f.life_max);
            if let Some(target) = target { f.def = target.clone(); f.anim_def = target; }
        }
        self.change_engine_state(i, ENGINE_STATE_AWAKENING);
        self.fighters[i].ctrl = false;
        self.pause = Some(PauseState { by: id, time: 80, move_time: 80, is_super: true, darken: true, unhittable: true, p2_def_mul: 1.0, pause_bg: true, ..Default::default() });
        self.do_shake(40, 5.0);
        self.events.push(Event::Transform { fighter: id });
        true
    }

    pub fn awaken_defeated_boss(&mut self) {
        let (buf, len) = self.root_index_buf();
        for &i in &buf[..len] {
            let f = &self.fighters[i];
            let max_form = f.base_def.forms.len().max(1) as u8;
            if f.def.boss && f.alive && self.wins[1 - f.side.min(1)] >= 1 && f.form < max_form {
                self.fighters[i].power = self.fighters[i].power_max;
                self.try_transform(i);
            }
        }
    }

    pub fn awaken_boss_final(&mut self) {
        for f in self.fighters.iter_mut().filter(|f| !f.is_helper && f.def.boss) {
            let max_form = f.base_def.forms.len().max(1) as u8;
            let (old_a, old_d) = form_multipliers(&f.base_def, f.form);
            let (new_a, new_d) = form_multipliers(&f.base_def, max_form);
            let base_life = f.base_def.c("data.life", 1000.0) as f32;
            f.form = max_form;
            f.form_time = 1;
            f.attack_mul = f.attack_mul / old_a * new_a;
            f.defence_mul = f.defence_mul / old_d * new_d;
            f.life_max = f.life_max.max((base_life * 1.15).round());
            f.life = f.life_max;
            f.power = f.power_max;
        }
    }
}
