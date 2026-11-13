use crate::chardef::{Animation, CharDef};
use crate::fighter::{HitDefData, V2};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct StageInfo {
    pub name: String,
    pub bound_left: f32, pub bound_right: f32, pub bound_high: f32, pub bound_low: f32,
    pub p1x: f32, pub p2x: f32, pub zoffset: f32, pub vertical_follow: f32, pub tension: f32,
    pub start_x: f32, pub start_y: f32, pub zoom_out: f32,
    pub zoom_in: f32,
    pub camera: CameraParams,
    pub auto_turn: bool,
}

#[derive(Clone, Debug)]
pub struct CameraParams {
    pub vertical_follow_delta: f32,
    pub y_scroll_speed: f32,
    pub zoom_in_speed: f32,
    pub zoom_out_speed: f32,
    pub lowest_cap: bool,
    pub dynamic_bound_high: bool,
    pub bound_high_delta: f32,
}
impl Default for CameraParams {
    fn default() -> Self { CameraParams { vertical_follow_delta: 0.0, y_scroll_speed: 0.3, zoom_in_speed: 0.04, zoom_out_speed: 0.04, lowest_cap: false, dynamic_bound_high: false, bound_high_delta: 0.0 } }
}
impl Default for StageInfo {
    fn default() -> Self { StageInfo { name: "Flat".into(), bound_left: -200.0, bound_right: 200.0, bound_high: -60.0, bound_low: 0.0, p1x: -70.0, p2x: 70.0, zoffset: 210.0, vertical_follow: 0.2, tension: 60.0, start_x: 0.0, start_y: 0.0, zoom_out: 1.0, zoom_in: 1.0, camera: CameraParams::default(), auto_turn: true } }
}

#[derive(Clone)]
pub struct AnimCursor { pub def: Arc<CharDef>, pub anim: i32, pub elem: usize, pub elem_time: i32, pub elapsed: i32 }
impl AnimCursor {
    pub fn new(def: Arc<CharDef>, anim: i32) -> Self { AnimCursor { def, anim, elem: 0, elem_time: 0, elapsed: 0 } }
    pub fn a(&self) -> Option<&Animation> { self.def.anims.get(&self.anim) }
    pub fn frame(&self) -> Option<&crate::chardef::AnimFrame> { self.a().and_then(|a| a.frames.get(self.elem)) }
    pub fn done(&self) -> bool { self.a().is_none_or(|a| a.total_time >= 0 && self.elapsed >= a.total_time) }
    pub fn advance(&mut self) {
        let Some(a) = self.def.anims.get(&self.anim) else { return };
        if a.frames.is_empty() { return; }
        if a.total_time >= 0 && self.elapsed >= a.total_time { self.elapsed = a.frames.iter().take(a.loop_start).map(|f| f.time.max(0)).sum(); }
        let ft = a.frames[self.elem].time;
        self.elapsed += 1;
        if ft < 0 { return; }
        self.elem_time += 1;
        if self.elem_time >= ft { self.elem_time = 0; self.elem += 1; if self.elem >= a.frames.len() { self.elem = a.loop_start; } }
    }
}

#[derive(Clone)]
pub struct Explod {
    pub owner: u32, pub id: i32, pub cur: AnimCursor, pub pos: V2, pub vel: V2, pub accel: V2, pub facing: f32, pub vfacing: f32,
    pub bind_time: i32, pub bind_to: Option<u32>, pub bind_off: V2, pub remove_time: i32, pub time: i32, pub spr_priority: i32,
    pub on_top: bool, pub under: bool, pub scale: [f32; 2], pub add: bool, pub remove_on_get_hit: bool, pub super_move: i32, pub pause_move: i32, pub lscl: f32,
    pub ignore_hit_pause: bool, pub screen: bool,
}

impl Explod {
    pub fn world_pos(&self, camera: V2) -> V2 { if self.screen { V2 { x: camera.x + self.pos.x, y: camera.y + self.pos.y } } else { self.pos } }
}

#[derive(Clone)]
pub struct Projectile {
    pub owner: u32, pub side: usize, pub teamside: usize, pub root: u32, pub tally: u32, pub id: i32, pub cur: AnimCursor, pub hit_anim: i32, pub rem_anim: i32, pub cancel_anim: i32,
    pub pos: V2, pub vel: V2, pub accel: V2, pub vel_mul: V2, pub rem_vel: V2, pub facing: f32, pub hits: i32, pub miss_time: i32, pub miss_cd: i32,
    pub priority: i32, pub spr_priority: i32, pub edge_bound: f32, pub height_bound: [f32; 2], pub remove_time: i32, pub time: i32,
    pub hitdef: HitDefData, pub dying: bool, pub guard_dist: f32, pub scale: [f32; 2], pub remove: bool, pub lscl: f32,
    pub remove_on_hit: bool, pub stage_bound: f32, pub super_move: i32, pub pause_move: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BgmAction {
    Play { file: String, looping: bool, volume: f32, loop_start: usize, loop_end: usize, freq_mul: f32 },
    Stop,
    Volume(f32),
    Loop(bool),
    LoopPoints { start: usize, end: usize },
    Rate(f32),
}

/// Events for the presentation layer (audio, rumble, UI).
#[derive(Clone)]
pub enum Event {
    Sound { def: Option<Arc<CharDef>>, group: i32, index: i32, own: bool, volume: f32, channel: i32, owner: u32, builtin: Option<&'static str>, pan: f32, looping: bool, freq_mul: f32, low_priority: bool, stop_on_get_hit: bool, stop_on_change_state: bool, loop_start: u32, loop_end: u32, loop_count: i32 },
    /// retunes the loop points of a live sound channel (`ModifySnd`)
    SoundLoop { owner: u32, channel: i32, start: u32, end: u32 },
    /// the fighter `fighter` entered a new state (used to stop flagged sound channels)
    StateChange { fighter: u32 },
    Hit { attacker_side: usize, defender_side: usize, damage: f32, guarded: bool, heavy: bool, attacker: usize, defender: usize },
    /// `fighter` = id of the root fighter that fired it (teams can have many per side)
    Super { side: usize, fighter: u32 },
    Ko { loser_side: usize, silent: bool, slow: bool },
    /// a fighter awakened into its transformed form (id of the root fighter)
    Transform { fighter: u32 },
    /// `fighter` spent meter to cancel a connected move (rush cancel)
    Rush { fighter: u32, side: usize, guarded: bool },
    StopSound { owner: u32, channel: i32 },
    SoundPan { owner: u32, channel: i32, pan: f32 },
    SoundVolume { owner: u32, channel: i32, volume: f32 },
    Rumble { fighter: u32, strong: f32, weak: f32, ms: u32 },
    Bgm(BgmAction),
    /// a tag-team swap: `fighter` (root id) just stepped in for `side`
    Tag { side: usize, fighter: u32 },
    RoundStart(i32), Fight, RoundEnd(i32), MatchOver(usize),
}

/// Modern-fighter damage proration: every hit of a combo deals less than the one before (never below 20 %).
/// `hits_before` = hits already landed in the combo. Applies to native (UMF) attackers; M.U.G.E.N. characters keep their own rules.
/// How long (ticks) the combo accumulator survives without a new hit to the same defender.
pub const COMBO_WINDOW: i32 = 70;
pub fn combo_scale(hits_before: i32) -> f32 {
    const T: [f32; 10] = [1.0, 1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.25];
    T[(hits_before.max(0) as usize).min(T.len() - 1)].max(0.2)
}
/// Hyper (super) hits never scale below half damage so a multi-hit finisher still feels like one, whatever combo led into it.
pub const HYPER_SCALE_FLOOR: f32 = 0.5;
pub fn combo_scale_for(hits_before: i32, hyper: bool) -> f32 {
    let scale = combo_scale(hits_before);
    if hyper { scale.max(HYPER_SCALE_FLOOR) } else { scale }
}

#[cfg(test)]
mod scale_tests {
    use super::*;

    #[test]
    fn normal_hits_keep_scaling_down_but_hyper_hits_stop_at_half() {
        assert_eq!(combo_scale_for(0, false), 1.0);
        assert_eq!(combo_scale_for(9, false), 0.25);
        assert_eq!(combo_scale_for(9, true), HYPER_SCALE_FLOOR);
        assert_eq!(combo_scale_for(2, true), 0.9, "early hits are not raised");
    }
}
/// How far (game px) the attacker is shoved back when the defender is cornered, given the hit's knock-back speed.
pub fn corner_push(xvel: f32) -> f32 { (xvel.abs() * 0.5).clamp(1.2, 4.5) }

pub const BASE_W: f32 = 426.67;
pub const BASE_H: f32 = 240.0;

pub const ENGINE_STATE_FACE_OFF: i32 = 19900;
pub const ENGINE_STATE_FACE_OFF_TAUNT: i32 = 19901;
pub const ENGINE_STATE_AWAKENING: i32 = 19902;

#[derive(Default, Clone)]
pub struct PauseState { pub by: u32, pub time: i32, pub move_time: i32, pub is_super: bool, pub darken: bool, pub unhittable: bool, pub p2_def_mul: f32, pub end_cmd_buf_time: i32, pub pause_bg: bool }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnnounceKind { #[default] Other, Round, Fight, Ko, Win }
#[derive(Clone)]
pub struct Announce { pub text: String, pub time: i32, pub total: i32, pub kind: AnnounceKind }

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TeamMode { #[default] Single, Simul, Turns, Tag }
impl TeamMode {
    pub fn number(self) -> i32 { self as i32 }
    pub fn name(self) -> &'static str { match self { TeamMode::Single => "single", TeamMode::Simul => "simul", TeamMode::Turns => "turns", TeamMode::Tag => "tag" } }
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() { "single" | "0" => Some(TeamMode::Single), "simul" | "1" => Some(TeamMode::Simul), "turns" | "2" => Some(TeamMode::Turns), "tag" | "3" => Some(TeamMode::Tag), _ => None }
    }
}

pub const MAX_EXPLODS: usize = 256;
pub const MAX_PROJECTILES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignKind { Var, FVar, SysVar, SysFVar }

impl AssignKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "var" => Some(AssignKind::Var),
            "fvar" => Some(AssignKind::FVar),
            "sysvar" => Some(AssignKind::SysVar),
            "sysfvar" => Some(AssignKind::SysFVar),
            _ => None,
        }
    }
}