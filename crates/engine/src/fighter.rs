//! Per-player (and per-helper) runtime state.
use crate::chardef::{AnimFrame, Animation, Box4, CharDef};
use crate::commands::CommandState;
use crate::input::InputHistory;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 { pub x: f32, pub y: f32 }

#[derive(Clone, Debug, Default)]
pub struct HitDefData {
    pub attr_state: String, pub attr_kind: Vec<String>, pub hitflag: String, pub guardflag: String,
    pub animtype: i32, pub air_animtype: i32, pub fall_animtype: i32, pub priority: i32,
    pub damage: f32, pub guard_damage: f32, pub pause_p1: i32, pub pause_p2: i32, pub guard_pause_p1: i32, pub guard_pause_p2: i32,
    pub spark_no: i32, pub spark_own: bool, pub guard_spark_no: i32, pub guard_spark_own: bool, pub spark_x: f32, pub spark_y: f32,
    pub hit_sound: Option<(i32, i32, bool)>, pub guard_sound: Option<(i32, i32, bool)>,
    pub ground_type: i32, pub air_type: i32, pub ground_slide_time: i32, pub guard_slide_time: i32,
    pub ground_hit_time: i32, pub guard_hit_time: i32, pub air_hit_time: i32, pub guard_ctrl_time: i32, pub guard_dist: f32,
    pub yaccel: f32, pub ground_vel: [f32; 2], pub guard_vel: f32, pub air_vel: [f32; 2], pub air_guard_vel: [f32; 2],
    pub p1sprpriority: i32, pub p2sprpriority: i32, pub p1facing: i32, pub p2facing: i32,
    pub p1stateno: i32, pub p2stateno: i32, pub p2getp1state: bool, pub forcestand: bool,
    pub fall: bool, pub air_fall: bool, pub fall_xvel: Option<f32>, pub fall_yvel: f32, pub fall_recover: bool, pub fall_recover_time: i32,
    pub fall_damage: f32, pub fall_kill: bool, pub kill: bool, pub guard_kill: bool,
    pub get_power: f32, pub give_power: f32, pub guard_get_power: f32, pub guard_give_power: f32,
    pub id: i32, pub chain_id: Option<i32>, pub envshake_time: i32, pub num_hits: i32,
    /** juggle points this move costs on an airborne, falling target (its Statedef `juggle`) and the attacker's `data.airjuggle` budget */
    pub juggle: i32, pub airjuggle: i32,
    pub no_chain_id: [Option<i32>; 2], pub hit_once: bool, pub snap: Option<V2>, pub min_dist: [Option<f32>; 2], pub max_dist: [Option<f32>; 2], pub affect_team: char,
    pub down_vel: [f32; 2], pub down_hit_time: i32, pub down_bounce: bool, pub fall_env_shake: Shake, pub env_shake: Shake,
    pub ground_corner_push: f32, pub air_corner_push: f32, pub guard_corner_push: f32, pub air_guard_corner_push: f32,
    pub target_palfx: Option<PalFx>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shake { pub time: i32, pub freq: f32, pub ampl: f32, pub phase: f32 }

#[derive(Clone, Debug)]
pub struct HitOverride { pub attr: String, pub state_no: i32, pub time: i32, pub force_air: bool }

#[derive(Clone, Debug)]
pub struct GetHitVars {
    pub kind: i32, pub animtype: i32, pub airtype: i32, pub groundtype: i32, pub damage: f32, pub hitcount: i32, pub guardcount: i32,
    pub hittime: i32, pub slidetime: i32, pub ctrltime: i32, pub xvel: f32, pub yvel: f32, pub yaccel: f32, pub chainid: i32,
    pub guarded: bool, pub fall: bool, pub fall_damage: f32, pub fall_xvel: f32, pub fall_yvel: f32, pub fall_recover: bool,
    pub fall_recover_time: i32, pub fall_kill: bool, pub kill: bool, pub attacker_id: u32, pub attacker_no: usize,
    pub hit_id: i32, pub fall_env_shake: Shake, pub down_bounce: bool,
}
impl Default for GetHitVars {
    fn default() -> Self {
        GetHitVars { kind: 0, animtype: 0, airtype: 0, groundtype: 0, damage: 0.0, hitcount: 0, guardcount: 0, hittime: 0, slidetime: 0, ctrltime: 0,
            xvel: 0.0, yvel: 0.0, yaccel: 0.44, chainid: -1, guarded: false, fall: false, fall_damage: 0.0, fall_xvel: 0.0, fall_yvel: -4.5,
            fall_recover: true, fall_recover_time: 4, fall_kill: true, kill: true, attacker_id: 0, attacker_no: 0,
            hit_id: -1, fall_env_shake: Shake::default(), down_bounce: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BindKind { Target, Parent, Root }
#[derive(Clone, Debug)]
pub struct Bind { pub to: u32, pub time: i32, pub x: f32, pub y: f32, pub kind: BindKind }

pub use crate::palette::PalFx;
use crate::palette::{AfterImageParams, Trans};

#[derive(Clone, Debug)]
pub struct TrailPoint { pub x: f32, pub y: f32, pub group: i32, pub index: i32, pub fx: f32, pub fy: f32, pub flip_h: bool, pub facing: f32 }
#[derive(Clone, Debug)]
pub struct AfterImage { pub time: i32, pub length: usize, pub trail: Vec<TrailPoint>, pub params: AfterImageParams, pub ticks: i32 }

/// AssertSpecial flags
pub mod asf {
    pub const NOAUTOTURN: u32 = 1; pub const NOWALK: u32 = 2; pub const NOJUMP: u32 = 4; pub const INVISIBLE: u32 = 8; pub const NOSHADOW: u32 = 16;
    pub const UNGUARDABLE: u32 = 32; pub const NOSTANDGUARD: u32 = 64; pub const NOCROUCHGUARD: u32 = 128; pub const NOAIRGUARD: u32 = 256;
    pub const NOKOSND: u32 = 512; pub const NOJUGGLE: u32 = 1024;
    pub fn from_name(s: &str) -> u32 {
        match s { "noautoturn" => NOAUTOTURN, "nowalk" => NOWALK, "nojump" => NOJUMP, "invisible" => INVISIBLE, "noshadow" => NOSHADOW, "unguardable" => UNGUARDABLE,
            "nostandguard" => NOSTANDGUARD, "nocrouchguard" => NOCROUCHGUARD, "noairguard" => NOAIRGUARD, "nokosnd" => NOKOSND, "nojugglecheck" => NOJUGGLE, _ => 0 }
    }
}

/// fallback chains for missing animations
pub fn anim_fallback(no: i32) -> &'static [i32] {
    match no {
        5 => &[0], 6 => &[11, 0], 10 => &[11, 0], 11 => &[0], 12 => &[0], 20 => &[0], 21 => &[20, 0], 40 => &[41, 0], 41 => &[40, 0], 42 => &[41, 40, 0], 43 => &[41, 40, 0],
        44 => &[41, 40, 0], 45 => &[42, 41, 40, 0], 46 => &[43, 41, 40, 0], 47 => &[40, 0], 100 => &[20, 0], 105 => &[43, 41, 0], 106 => &[47, 0],
        120 => &[130, 0], 121 => &[131, 11, 0], 122 => &[132, 41, 0], 130 => &[120, 0], 131 => &[121, 11], 132 => &[122, 41],
        140 => &[130, 0], 141 => &[131, 11], 142 => &[132, 41], 150 => &[130, 5000, 0], 151 => &[150, 130, 0], 152 => &[131, 5010, 11], 153 => &[152, 131, 11], 154 => &[132, 5030, 41], 155 => &[154, 132, 41],
        170 | 175 | 181 | 190 | 191 | 195 | 196 => &[0], 197..=200 => &[5150, 5110], 180 => &[181, 0],
        5000 => &[5001, 5002, 0], 5001 => &[5000, 0], 5002 => &[5001, 5000, 0], 5005 => &[5000, 0], 5006 => &[5005, 5001, 5000], 5007 => &[5006, 5005, 5002, 5000],
        5010 => &[5000, 0], 5011 => &[5010, 5001, 5000], 5012 => &[5011, 5010, 5002, 5000], 5020 => &[5010, 5000, 11], 5021 => &[5020, 5011, 5001], 5022 => &[5021, 5012, 5002],
        5030 => &[5000, 0], 5035 => &[5030, 5000], 5040 => &[5030, 41, 0], 5050 => &[5030, 5000], 5051 | 5052 => &[5050], 5060 => &[5050, 5030], 5061 => &[5060, 5050],
        5070 | 5071 => &[5050, 5030], 5080 | 5090 => &[5110, 5050], 5100 => &[5110, 5050], 5101 => &[5100, 5050], 5110 => &[5100, 5050],
        5120 => &[5110, 0], 5140 | 5150 | 5170 => &[5110], 5160 => &[5101, 5100], 5200 | 5210 => &[5040, 41],
        _ => &[],
    }
}

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
    pub anim_no: i32, pub anim_real: i32, pub elem: usize, pub elem_time: i32, pub anim_elapsed: i32, pub anim_changed: bool,
    pub hitdef: Option<HitDefData>, pub hitdef_active: bool, pub hitdef_targets: Vec<u32>,
    pub move_contact: i32, pub move_hit: i32, pub move_guarded: i32, pub move_reversed: i32, pub hit_count: i32,
    pub ghv: GetHitVars, /** juggle points already spent on this fighter in the current air combo */ pub juggle_used: i32, /** consecutive ticks without control during the fight (stuck-state watchdog) */ pub no_ctrl_ticks: i32, pub hit_pause: i32, pub hit_shake: i32, pub hit_over_time: i32, pub slide_time: i32, pub fall_time: i32,
    pub asserts: u32,
    pub width_override: Option<(f32, f32)>, pub player_push: bool, pub screen_bound: bool, pub pos_freeze: bool, pub spr_priority: i32,
    pub targets: Vec<(u32, i32)>,
    pub not_hit_by: Vec<(String, i32)>, pub hit_by: Vec<(String, i32)>,
    pub attack_mul: f32, pub defence_mul: f32,
    pub persist: HashMap<u32, i32>,
    pub air_jumps: i32,
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
    pub combo_count: i32, pub combo_damage: f32,
    /// 0 = base form, 1 = transformed (awakened) until the round ends
    pub form: u8, pub form_time: i32,
    pub draw_offset: V2, pub victory_quote: i32, pub clipboard: Vec<String>, pub unhittable: i32, pub super_defence_mul: f32,
    pub reversal: Option<(String, HitDefData)>, pub hit_overrides: [Option<HitOverride>; 8],
    pub ikemen: crate::ikemen::IkemenState,
    pub pause_move_time: i32, pub super_move_time: i32, pub preserve: bool, pub own_pal: bool, pub state_giver: Option<u32>,
}

impl Fighter {
    pub fn new(id: u32, def: Arc<CharDef>, side: usize, player_no: usize) -> Self {
        let lscl = 320.0 / if def.local_w > 0.0 { def.local_w } else { 320.0 };
        let life = def.c("data.life", 1000.0) as f32;
        Fighter {
            id, st_def: def.clone(), anim_def: def.clone(), side, player_no, is_helper: false, helper_id: 0, helper_player: false, key_ctrl: false,
            parent: None, root: id, destroyed: false, pal_no: 1, ai_level: 0,
            pos: V2::default(), vel: V2::default(), facing: 1.0, lscl,
            state_no: 0, prev_state_no: 0, state_time: 0, state_type: 'S', move_type: 'I', physics: 'S', ctrl: true,
            life, life_max: life, power: 0.0, power_max: def.c("data.power", 3000.0) as f32, alive: true,
            vars: [0; 60], fvars: [0.0; 40], sysvars: [0; 5], sysfvars: [0.0; 5],
            anim_no: -1, anim_real: -1, elem: 0, elem_time: 0, anim_elapsed: 0, anim_changed: false,
            hitdef: None, hitdef_active: false, hitdef_targets: vec![], move_contact: 0, move_hit: 0, move_guarded: 0, move_reversed: 0, hit_count: 0,
            ghv: GetHitVars::default(), juggle_used: 0, no_ctrl_ticks: 0, hit_pause: 0, hit_shake: 0, hit_over_time: 0, slide_time: 0, fall_time: 0, asserts: 0,
            width_override: None, player_push: true, screen_bound: true, pos_freeze: false, spr_priority: 0, targets: vec![],
            not_hit_by: vec![], hit_by: vec![], attack_mul: def.c("data.attack", 100.0) as f32 / 100.0, defence_mul: 1.0,
            persist: HashMap::new(), air_jumps: 0, bind: None, palfx: None, trans: None, remap: vec![], angle: 0.0, angle_draw: false, draw_scale: [1.0, 1.0],
            after_image: None, proj_contacts: vec![], last_hit_by: None, history: InputHistory::default(), cmd: CommandState::default(),
            combo_count: 0, combo_damage: 0.0, form: 0, form_time: 0, reversal: None, hit_overrides: Default::default(), ikemen: crate::ikemen::IkemenState::new(&def),
            pause_move_time: 0, super_move_time: 0, preserve: false, own_pal: true, state_giver: None,
            draw_offset: V2::default(), victory_quote: -1, clipboard: vec![], unhittable: 0, super_defence_mul: 1.0, def,
        }
    }
    pub fn c(&self, k: &str, d: f64) -> f32 { self.def.c(k, d) as f32 }

    pub fn anim(&self) -> Option<&Animation> { self.anim_def.anims.get(&self.anim_real) }
    pub fn frame(&self) -> Option<&AnimFrame> { self.anim().and_then(|a| a.frames.get(self.elem)) }

    pub fn anim_time_trig(&self) -> i32 {
        match self.anim() { None => 0, Some(a) if a.total_time < 0 => -1, Some(a) => self.anim_elapsed - a.total_time }
    }
    pub fn elem_start(&self, n: usize) -> i32 {
        let Some(a) = self.anim() else { return 0 };
        a.frames.iter().take(n.saturating_sub(1)).map(|f| f.time.max(0)).sum()
    }
    pub fn anim_elem_started(&self, n: i32) -> bool {
        self.elem as i32 + 1 == n && self.elem_time == 0 && (n > 1 || self.anim_elapsed == 0 || self.anim().is_some_and(|a| a.loop_start == 0))
    }
    pub fn anim_elem_time(&self, n: i32) -> Option<i32> {
        let a = self.anim()?;
        if n < 1 || n as usize > a.frames.len() { return None; }
        Some(self.anim_elapsed - self.elem_start(n as usize))
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
        let mut real = if from.anims.contains_key(&no) { Some(no) } else { None };
        if real.is_none() { real = anim_fallback(no).iter().copied().find(|a| from.anims.contains_key(a)); }
        let real = match real {
            Some(r) => r,
            None => {
                if self.anim().is_some() && Arc::ptr_eq(from, &self.anim_def) { self.anim_no = no; return; }
                if from.anims.contains_key(&0) { 0 } else { return }
            }
        };
        self.anim_def = from.clone();
        self.anim_no = no;
        self.anim_real = real;
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
        let (total, loop_start, len) = match self.anim() { Some(a) if !a.frames.is_empty() => (a.total_time, a.loop_start, a.frames.len()), _ => return };
        if total >= 0 && self.anim_elapsed >= total { self.anim_elapsed = self.elem_start(loop_start + 1); }
        let ft = self.anim().unwrap().frames[self.elem].time;
        self.anim_elapsed += 1;
        if ft < 0 { return; }
        self.elem_time += 1;
        if self.elem_time >= ft {
            self.elem_time = 0;
            self.elem += 1;
            if self.elem >= len { self.elem = loop_start; }
            let mut guard = 0;
            while self.anim().unwrap().frames[self.elem].time == 0 && guard < len {
                guard += 1;
                self.elem += 1;
                if self.elem >= len { self.elem = loop_start; }
            }
        }
    }

    pub fn front_width(&self) -> f32 {
        if let Some((f, _)) = self.width_override { return f; }
        (if self.state_type == 'A' { self.c("size.air.front", 12.0) } else { self.c("size.ground.front", 16.0) }) * self.lscl
    }
    pub fn back_width(&self) -> f32 {
        if let Some((_, b)) = self.width_override { return b; }
        (if self.state_type == 'A' { self.c("size.air.back", 12.0) } else { self.c("size.ground.back", 15.0) }) * self.lscl
    }
    pub fn add_power_local(&mut self, v: f32) { self.power = (self.power + v).clamp(0.0, self.power_max); }

    /// Collision boxes in world space (kind 1 = attack, 2 = hurt).
    pub fn boxes(&self, kind: u8) -> Vec<Box4> {
        let Some(fr) = self.frame() else { return vec![] };
        let list = if kind == 1 { &fr.clsn1 } else { &fr.clsn2 };
        let s = self.lscl * self.draw_scale[0];
        let sx = self.def.c("size.xscale", 1.0) as f32;
        let sy = self.def.c("size.yscale", 1.0) as f32;
        list.iter().map(|b| {
            let (mut x1, mut x2) = (b[0] * sx * s, b[2] * sx * s);
            if fr.flip_h { let t = -x2; x2 = -x1; x1 = t; }
            if self.facing < 0.0 { let t = -x2; x2 = -x1; x1 = t; }
            [self.pos.x + x1, self.pos.y + b[1] * sy * s, self.pos.x + x2, self.pos.y + b[3] * sy * s]
        }).collect()
    }
    pub fn has_assert(&self, f: u32) -> bool { self.asserts & f != 0 }
    /// True while nothing legitimately freezes this fighter (round running, no super pause, no hit pause/shake, not bound).
    pub fn round_free(&self, fighting: bool, paused: bool) -> bool { fighting && !paused && self.hit_pause == 0 && self.hit_shake == 0 && self.bind.is_none() }
}
