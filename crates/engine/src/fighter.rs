//! Per-player (and per-helper) runtime state.
use crate::chardef::{AnimFrame, Animation, Box4, CharDef};
use crate::commands::CommandState;
use crate::input::InputHistory;
use std::sync::Arc;

mod data;
pub use data::*;
use crate::palette::Trans;
pub use crate::flags::asf;
pub use crate::anim_fallback;
pub use crate::anim_fallback::anim_fallback;

#[derive(Clone)]
pub struct Fighter {
    pub id: u32,
    pub def: Arc<CharDef>,
    pub st_def: Arc<CharDef>,
    pub anim_def: Arc<CharDef>,
    pub side: usize,
    pub player_no: usize,
    pub is_helper: bool,
    pub helper_id: i32,
    pub helper_player: bool,
    pub helper_projectile: bool,
    pub own_projectile: bool,
    pub own_clsn_scale: bool,
    pub size_overrides: Vec<(String, f32)>,
    pub key_ctrl: bool,
    pub parent: Option<u32>,
    pub root: u32,
    pub destroyed: bool,
    pub pal_no: i32,
    pub ai_level: i32,
    pub pos: V2, pub vel: V2, pub facing: f32, pub lscl: f32,
    pub state_no: i32, pub prev_state_no: i32, pub state_time: i32,
    pub state_type: char, pub move_type: char, pub physics: char, pub ctrl: bool,
    pub life: f32, pub life_max: f32, pub power: f32, pub power_max: f32, pub alive: bool,
    pub vars: [i32; 60], pub fvars: [f64; 40], pub sysvars: [i32; 5], pub sysfvars: [f64; 5],
    pub anim_no: i32, pub anim_real: i32, pub elem: usize, pub elem_time: i32, pub anim_elapsed: i32, pub anim_changed: bool, pub anim_total_time: i32, pub anim_loop_start: usize,
    pub hitdef: Option<HitDefData>, pub hitdef_active: bool, pub hitdef_targets: Vec<u32>,
    pub move_contact: i32, pub move_hit: i32, pub move_guarded: i32, pub move_reversed: i32, pub hit_count: i32, pub uniq_hit_count: i32, pub uniq_hit_keys: Vec<(u32, i32)>,
    pub move_countered: i32, pub move_hit_var: MoveHitVars, pub hit_overridden_at: i64, pub prev_state_type: char, pub prev_move_type: char, pub prev_anim: i32,
    pub ghv: GetHitVars, /** juggle points already spent on this fighter in the current air combo */ pub juggle_used: i32, /** consecutive ticks without control during the fight (stuck-state watchdog) */ pub no_ctrl_ticks: i32, pub hit_pause: i32, pub hit_shake: i32, pub hit_over_time: i32, pub slide_time: i32, pub fall_time: i32,
pub asserts: u64,
    pub width_override: Option<(f32, f32)>, pub height_override: Option<(f32, f32)>, pub clsn_mods: Vec<crate::sctrl::clsn::ClsnMod>, pub run_state_depth: u8, pub player_push: bool, pub screen_bound: bool, pub pos_freeze: bool, pub spr_priority: i32,
    pub targets: Vec<(u32, i32)>,
    pub not_hit_by: Vec<(String, i32)>, pub hit_by: Vec<(String, i32)>,
    pub attack_mul: f32, pub defence_mul: f32,
    pub persist: Vec<(u32, i32)>,
    pub air_jumps: i32,
    pub ground_level: f32,
    pub bind: Option<Bind>,
    pub palfx: Option<PalFx>,
    pub trans: Option<Trans>,
    pub remap: Vec<((i32, i32), (i32, i32))>,
    pub angle: f32, pub angle_draw: bool, pub draw_scale: [f32; 2],
    pub after_image: Option<AfterImage>,
    pub proj_contacts: Vec<(char, i32, i64)>,
    pub last_hit_by: Option<u32>,
    pub history: InputHistory,
    pub cmd: CommandState,
    pub combo_count: i32, pub combo_damage: f32, pub combo_target: Option<u32>, pub combo_timer: i32,
    pub received_damage: f32, pub received_hits: i32, pub score: f64, pub score_total: f64,
    pub asserted_commands: Vec<(String, i32)>, pub assert_input: (u32, i32), pub input_shift: i32,
    pub sprite_remap: Vec<((i32, i32), (i32, i32), i32)>, pub sprite_transform: Option<(f32, f32, f32, i32)>,
    /// 0 = base form, n = nth transformation (CharDef::forms, legacy awakening = 1) until the round ends
    pub form: u8, pub form_time: i32, pub base_def: Arc<CharDef>,
    pub draw_offset: V2, pub victory_quote: i32, pub clipboard: Vec<String>, pub unhittable: i32, pub super_defence_mul: f32,
    pub reversal: Option<(String, HitDefData)>, pub hit_overrides: [Option<HitOverride>; 8],
    pub ikemen: crate::ikemen::IkemenState,
    pub pause_move_time: i32, pub super_move_time: i32, pub pause_bool: bool, pub preserve: bool, pub own_pal: bool, pub state_giver: Option<u32>,
}

impl Fighter {
    pub fn new(id: u32, def: Arc<CharDef>, side: usize, player_no: usize) -> Self {
        let lscl = 320.0 / if def.local_w > 0.0 { def.local_w } else { 320.0 };
        let life = def.c("data.life", 1000.0) as f32;
        Fighter {
            id, st_def: def.clone(), anim_def: def.clone(), side, player_no, is_helper: false, helper_id: 0, helper_player: false, helper_projectile: false, own_projectile: false, own_clsn_scale: false, size_overrides: vec![], key_ctrl: false,
            parent: None, root: id, destroyed: false, pal_no: 1, ai_level: 0,
            pos: V2::default(), vel: V2::default(), facing: 1.0, lscl,
            state_no: 0, prev_state_no: 0, state_time: 0, state_type: 'S', move_type: 'I', physics: 'S', ctrl: true,
            life, life_max: life, power: 0.0, power_max: def.c("data.power", 3000.0) as f32, alive: true,
            vars: [0; 60], fvars: [0.0; 40], sysvars: [0; 5], sysfvars: [0.0; 5],
            anim_no: -1, anim_real: -1, elem: 0, elem_time: 0, anim_elapsed: 0, anim_changed: false, anim_total_time: 0, anim_loop_start: 0,
            hitdef: None, hitdef_active: false, hitdef_targets: vec![], move_contact: 0, move_hit: 0, move_guarded: 0, move_reversed: 0, hit_count: 0, uniq_hit_count: 0, uniq_hit_keys: vec![],
            move_countered: 0, move_hit_var: MoveHitVars::default(), hit_overridden_at: 0, prev_state_type: 'S', prev_move_type: 'I', prev_anim: -1,
            ghv: GetHitVars::default(), juggle_used: 0, no_ctrl_ticks: 0, hit_pause: 0, hit_shake: 0, hit_over_time: 0, slide_time: 0, fall_time: 0, asserts: 0,
            width_override: None, height_override: None, clsn_mods: vec![], run_state_depth: 0, player_push: true, screen_bound: true, pos_freeze: false, spr_priority: 0, targets: vec![],
            not_hit_by: vec![], hit_by: vec![], attack_mul: def.c("data.attack", 100.0) as f32 / 100.0, defence_mul: 1.0,
            persist: Vec::new(), air_jumps: 0, ground_level: 0.0, bind: None, palfx: None, trans: None, remap: vec![], angle: 0.0, angle_draw: false, draw_scale: [1.0, 1.0],
            after_image: None, proj_contacts: vec![], last_hit_by: None, history: InputHistory::default(), cmd: CommandState::default(),
            combo_count: 0, combo_damage: 0.0, combo_target: None, combo_timer: 0, received_damage: 0.0, received_hits: 0, score: 0.0, score_total: 0.0, asserted_commands: vec![], assert_input: (0, 0), input_shift: 0, sprite_remap: vec![], sprite_transform: None, form: 0, form_time: 0, base_def: def.clone(), reversal: None, hit_overrides: Default::default(), ikemen: crate::ikemen::IkemenState::new(&def),
            pause_move_time: 0, super_move_time: 0, pause_bool: false, preserve: false, own_pal: true, state_giver: None,
            draw_offset: V2::default(), victory_quote: -1, clipboard: vec![], unhittable: 0, super_defence_mul: 1.0, def,
        }
    }
    pub fn c(&self, k: &str, d: f64) -> f32 {
        let fallback = |def: &CharDef| -> f32 {
            match k {
                "size.xscale" => def.scale_x,
                "size.yscale" => def.scale_y,
                "size.ground.back" => def.size_ground_back,
                "size.ground.front" => def.size_ground_front,
                "size.air.back" => def.size_air_back,
                "size.air.front" => def.size_air_front,
                "size.height" => def.size_height,
                "size.attack.dist" => def.size_attack_dist,
                _ => def.c(k, d) as f32,
            }
        };
        if self.size_overrides.is_empty() {
            return fallback(&self.def);
        }
        match self.size_overrides.iter().find(|(n, _)| n == k) {
            Some((_, v)) => *v,
            None => fallback(&self.def),
        }
    }
    pub fn projectile_tally_id(&self) -> u32 { if self.own_projectile { self.id } else { self.root } }

    pub fn anim(&self) -> Option<&Animation> { self.anim_def.anims.get(&self.anim_real) }
    pub fn frame(&self) -> Option<&AnimFrame> { self.anim().and_then(|a| a.frames.get(self.elem)) }

    pub fn anim_time_trig(&self) -> i32 {
        if self.anim_total_time < 0 { -1 } else { self.anim_elapsed - self.anim_total_time }
    }
    pub fn elem_start(&self, n: usize) -> i32 {
        if n <= 1 { return 0; }
        let Some(a) = self.anim() else { return 0 };
        a.frames.iter().take(n - 1).map(|f| f.time.max(0)).sum()
    }
    pub fn anim_elem_started(&self, n: i32) -> bool {
        self.elem as i32 + 1 == n && self.elem_time == 0 && (n > 1 || self.anim_elapsed == 0 || self.anim_loop_start == 0)
    }
    pub fn anim_elem_time(&self, n: i32) -> Option<i32> {
        let a = self.anim()?;
        if n < 1 || n as usize > a.frames.len() { return None; }
        let start = if n == 1 { 0 } else { a.frames.iter().take(n as usize - 1).map(|f| f.time.max(0)).sum() };
        Some(self.anim_elapsed - start)
    }
    pub fn anim_elem_no_at(&self, off: i32) -> i32 {
        let Some(a) = self.anim() else { return 0 };
        let mut t = self.anim_elapsed + off;
        let tot = a.total_time;
        if tot > 0 && t >= tot {
            let ls = self.elem_start(a.loop_start + 1);
            let ll = tot - ls;
            t = if ll > 0 { ls + (t - ls) % ll } else { ls };
        }
        let mut acc = 0;
        for (i, f) in a.frames.iter().enumerate() {
            if f.time < 0 || t < acc + f.time { return i as i32 + 1; }
            acc += f.time;
        }
        a.frames.len() as i32
    }
    /// Change animation; `from` is the CharDef providing the animation (own, or state owner for ChangeAnim2).
    pub fn change_anim(&mut self, no: i32, from: &Arc<CharDef>, elem: i32) {
        if no != self.anim_no { self.prev_anim = self.anim_no; }
        let mut real = if from.anims.contains_key(&no) { Some(no) } else { None };
        if real.is_none() { real = crate::anim_fallback::anim_fallback(no).iter().copied().find(|a| from.anims.contains_key(a)); }
        let real = match real {
            Some(r) => r,
            None => {
                if self.anim().is_some() && Arc::ptr_eq(from, &self.anim_def) { self.anim_no = no; return; }
                if from.anims.contains_key(&0) { 0 } else { return }
            }
        };
        if !Arc::ptr_eq(from, &self.anim_def) {
            self.anim_def = from.clone();
        }
        self.anim_no = no;
        self.anim_real = real;
        let (tot, ls) = from.anims.get(&real).map(|a| (a.total_time, a.loop_start)).unwrap_or((0, 0));
        self.anim_total_time = tot;
        self.anim_loop_start = ls;
        self.elem = 0; self.elem_time = 0; self.anim_elapsed = 0;
        if elem > 1 {
            self.anim_elapsed = self.elem_start(elem as usize);
            let n = self.anim().map(|a| a.frames.len()).unwrap_or(1);
            self.elem = ((elem - 1) as usize).min(n - 1);
        }
        self.anim_changed = true;
    }
    pub fn change_anim_own(&mut self, no: i32, elem: i32) {
        if no != self.anim_no { self.prev_anim = self.anim_no; }
        let from = &self.def;
        let mut real = if from.anims.contains_key(&no) { Some(no) } else { None };
        if real.is_none() { real = crate::anim_fallback::anim_fallback(no).iter().copied().find(|a| from.anims.contains_key(a)); }
        let real = match real {
            Some(r) => r,
            None => {
                if self.anim().is_some() && Arc::ptr_eq(from, &self.anim_def) { self.anim_no = no; return; }
                if from.anims.contains_key(&0) { 0 } else { return }
            }
        };
        if !Arc::ptr_eq(from, &self.anim_def) {
            self.anim_def = from.clone();
        }
        self.anim_no = no;
        self.anim_real = real;
        let (tot, ls) = from.anims.get(&real).map(|a| (a.total_time, a.loop_start)).unwrap_or((0, 0));
        self.anim_total_time = tot;
        self.anim_loop_start = ls;
        self.elem = 0; self.elem_time = 0; self.anim_elapsed = 0;
        if elem > 1 {
            self.anim_elapsed = self.elem_start(elem as usize);
            let n = self.anim().map(|a| a.frames.len()).unwrap_or(1);
            self.elem = ((elem - 1) as usize).min(n - 1);
        }
        self.anim_changed = true;
    }
    pub fn advance_anim(&mut self) {
        if self.anim_changed { self.anim_changed = false; return; }
        let Some(a) = self.anim_def.anims.get(&self.anim_real) else { return; };
        if a.frames.is_empty() { return; }
        let len = a.frames.len();
        let ft = a.frames[self.elem].time;
        if self.anim_total_time >= 0 && self.anim_elapsed >= self.anim_total_time {
            self.anim_elapsed = if self.anim_loop_start == 0 { 0 } else { a.frames.iter().take(self.anim_loop_start).map(|f| f.time.max(0)).sum() };
        }
        self.anim_elapsed += 1;
        if ft < 0 { return; }
        self.elem_time += 1;
        if self.elem_time >= ft {
            self.elem_time = 0;
            self.elem += 1;
            if self.elem >= len { self.elem = self.anim_loop_start; }
            let mut guard = 0;
            while a.frames[self.elem].time == 0 && guard < len {
                guard += 1;
                self.elem += 1;
                if self.elem >= len { self.elem = self.anim_loop_start; }
            }
        }
    }

    pub fn air_height(&self) -> f32 { self.pos.y - self.ground_level }
    pub fn front_width(&self) -> f32 {
        if let Some((f, _)) = self.width_override { return f; }
        (if self.state_type == 'A' { self.c("size.air.front", 12.0) } else { self.c("size.ground.front", 16.0) }) * self.lscl
    }
    pub fn back_width(&self) -> f32 {
        if let Some((_, b)) = self.width_override { return b; }
        (if self.state_type == 'A' { self.c("size.air.back", 12.0) } else { self.c("size.ground.back", 15.0) }) * self.lscl
    }
    pub fn height_top(&self) -> f32 { self.height_override.map(|h| h.0).unwrap_or_else(|| self.c("size.height", 60.0) * self.lscl) }
    pub fn height_bottom(&self) -> f32 { self.height_override.map(|h| h.1).unwrap_or(0.0) }
    pub fn add_power_local(&mut self, v: f32) { self.power = (self.power + v).clamp(0.0, self.power_max); }

    pub fn fill_boxes(&self, kind: u8, out: &mut Vec<Box4>) {
        out.clear();
        let Some(fr) = self.frame() else { return; };
        let list = if kind == 1 { &fr.clsn1 } else { &fr.clsn2 };
        let modified;
        let list: &[Box4] = if self.clsn_mods.iter().any(|c| c.kind == kind) {
            modified = crate::sctrl::clsn::apply(&self.clsn_mods, kind, list);
            &modified
        } else {
            list
        };
        if list.is_empty() {
            if kind == 2 {
                if let Some((top, bottom)) = self.height_override {
                    let (a, b) = (self.pos.x - self.back_width() * self.facing, self.pos.x + self.front_width() * self.facing);
                    out.push([a.min(b), self.pos.y - top, a.max(b), self.pos.y + bottom]);
                }
            }
            return;
        }
        let s = self.lscl * self.draw_scale[0];
        let (xscale, yscale) = if self.size_overrides.is_empty() {
            (self.def.scale_x, self.def.scale_y)
        } else {
            (self.c("size.xscale", 1.0), self.c("size.yscale", 1.0))
        };
        let sx = xscale * s;
        let sy = yscale * self.lscl * self.draw_scale[1];
        let flip = fr.flip_h != (self.facing < 0.0);
        out.reserve(list.len());
        for b in list {
            let (mut x1, mut x2) = (b[0] * sx, b[2] * sx);
            if flip { let t = -x2; x2 = -x1; x1 = t; }
            out.push([self.pos.x + x1, self.pos.y + b[1] * sy, self.pos.x + x2, self.pos.y + b[3] * sy]);
        }
    }

    pub fn boxes(&self, kind: u8) -> Vec<Box4> {
        let mut out = Vec::new();
        self.fill_boxes(kind, &mut out);
        out
    }
    pub fn has_assert(&self, f: u64) -> bool { self.asserts & f != 0 }
    /// True while nothing legitimately freezes this fighter (round running, no super pause, no hit pause/shake, not bound).
    pub fn round_free(&self, fighting: bool, paused: bool) -> bool { fighting && !paused && self.hit_pause == 0 && self.hit_shake == 0 && self.bind.is_none() }
}
