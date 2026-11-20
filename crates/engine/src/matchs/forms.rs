use super::*;

const LEGACY_COST: f32 = 1000.0;
const LEGACY_ATTACK: f32 = 1.3;


fn form_defence(def: &CharDef, entered: u8) -> f32 {
    def.forms.iter().take(entered as usize).fold(1.0, |defence, form| defence * form.defense)
}

fn stored_form_attack(def: &CharDef, entered: u8) -> f32 {
    if def.source == Source::Mugen && def.forms.is_empty() { return 1.0; }
    let minimum = 1.0 + (LEGACY_ATTACK - 1.0) * f32::from(entered.min(6));
    def.forms.iter().take(entered as usize).fold(1.0, |attack, form| attack * form.attack).max(minimum)
}

impl Fighter {
    pub fn next_form_cost(&self) -> Option<f32> {
        let forms = &self.base_def.forms;
        let next = self.form as usize;
        if forms.is_empty() { return (next < self.form_count()).then_some(LEGACY_COST); }
        forms.get(next).map(|form| form.cost)
    }

    pub fn form_count(&self) -> usize {
        let forms = &self.base_def.forms;
        if !forms.is_empty() { return forms.len(); }
        usize::from(self.base_def.anims.contains_key(&196) || self.base_def.source == Source::Mugen && self.power_max >= LEGACY_COST)
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

    pub(crate) fn legacy_awakening_attack(&self) -> f32 {
        if self.form > 0 && self.base_def.source == Source::Mugen && self.base_def.forms.is_empty() {
            LEGACY_ATTACK
        } else {
            1.0
        }
    }

    pub fn form_speed_multiplier(&self) -> f32 {
        1.0 + 0.08 * f32::from(self.form.min(6))
    }

    pub fn exposed_damage_multiplier(&self) -> f32 {
        1.0 + 0.15 * f32::from(self.form.min(6))
    }

    pub(crate) fn form_exposure(&self) -> f32 {
        form_defence(&self.base_def, self.form) * self.exposed_damage_multiplier()
    }

    pub(super) fn exposed_defence(&self) -> f32 {
        self.effective_defence() / self.form_exposure()
    }

    pub fn revert_form(&mut self) {
        if self.form == 0 { return; }
        self.attack_mul /= stored_form_attack(&self.base_def, self.form);
        self.defence_mul /= form_defence(&self.base_def, self.form);
        self.form = 0;
        self.form_time = 0;
        self.def = self.base_def.clone();
        self.anim_def = self.def.clone();
    }
}

impl Match {
    pub fn try_transform(&mut self, i: usize) -> bool {
        let f = &self.fighters[i];
        let Some(cost) = f.next_form_cost() else { return false };
        if f.is_helper || !f.alive || !f.ctrl || f.state_type == 'A' || f.power < cost || self.round_state != 2 || self.pause.is_some() { return false; }
        let id = f.id;
        let target_form = (f.form + 1).min(f.base_def.forms.len().max(1) as u8);
        let old_a = stored_form_attack(&f.base_def, f.form);
        let new_a = stored_form_attack(&f.base_def, target_form);
        let defence_gain = form_defence(&f.base_def, target_form) / form_defence(&f.base_def, f.form);
        let target = f.base_def.forms.get((target_form as usize).saturating_sub(1)).and_then(|fm| fm.target.clone());
        {
            let f = &mut self.fighters[i];
            f.power = (f.power - cost).max(0.0);
            f.form = target_form;
            f.form_time = 0;
            f.attack_mul = f.attack_mul / old_a * new_a;
            f.defence_mul *= defence_gain;
            if let Some(target) = target { f.def = target.clone(); f.anim_def = target; }
        }
        self.change_engine_state(i, ENGINE_STATE_AWAKENING);
        let f = &mut self.fighters[i];
        f.ctrl = false;
        if f.def.source == Source::Native && f.def.anims.contains_key(&196) {
            f.change_anim_own(196, 1);
        }
        self.pause = Some(PauseState { by: id, time: 60, move_time: 60, is_super: true, darken: true, unhittable: true, p2_def_mul: 1.0, pause_bg: true, ..Default::default() });
        self.do_shake(40, 5.0);
        self.events.push(Event::Transform { fighter: id });
        true
    }

    pub fn awaken_defeated_boss(&mut self) {
        let (buf, len) = self.root_index_buf();
        for &i in &buf[..len] {
            let f = &self.fighters[i];
            if f.def.boss && f.alive && f.ctrl && f.state_type != 'A' && f.form == 0 && self.wins[1 - f.side.min(1)] >= 1 && f.form_count() > 0 {
                self.fighters[i].power = self.fighters[i].power_max;
                self.try_transform(i);
            }
        }
    }

    pub fn awaken_boss_phase(&mut self, phase: u8) {
        for f in self.fighters.iter_mut().filter(|f| !f.is_helper && f.def.boss) {
            let max_form = f.base_def.forms.len().max(1) as u8;
            let target_form = phase.clamp(1, max_form);
            let old_a = stored_form_attack(&f.base_def, f.form);
            let new_a = stored_form_attack(&f.base_def, target_form);
            let defence_gain = form_defence(&f.base_def, target_form) / form_defence(&f.base_def, f.form);
            let base_life = f.base_def.c("data.life", 1000.0) as f32;
            f.form = target_form;
            f.form_time = 1;
            f.attack_mul = f.attack_mul / old_a * new_a;
            f.defence_mul *= defence_gain;
            let mult = if target_form >= 3 { 1.30 } else { 1.15 };
            f.life_max = f.life_max.max((base_life * mult).round());
            f.life = f.life_max;
            f.power = f.power_max;
        }
    }

    pub fn awaken_boss_final(&mut self) {
        self.awaken_boss_phase(2);
    }
}
