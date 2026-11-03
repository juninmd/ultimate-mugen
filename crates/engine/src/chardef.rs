//! Character definition shared by MUGEN-loaded and UMF-compiled characters.
use crate::commands::CommandDef;
use crate::expr::{parse, parse_list, Expr, Node};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
pub use ume_formats::air::{AnimFrame, Animation, Blend, Box4};

static IMG_ID: AtomicU64 = AtomicU64::new(1);

/// RGBA image owned by the engine (uploaded to a GPU texture by the renderer, cached by `id`).
#[derive(Debug)]
pub struct ImageData { pub w: u32, pub h: u32, pub rgba: Vec<u8>, pub id: u64 }
impl ImageData {
    pub fn new(w: u32, h: u32, rgba: Vec<u8>) -> Arc<Self> { Arc::new(ImageData { w, h, rgba, id: IMG_ID.fetch_add(1, Ordering::Relaxed) }) }
}

#[derive(Clone, Debug)]
pub struct Sprite { pub image: Option<Arc<ImageData>>, pub width: f32, pub height: f32, pub axis_x: f32, pub axis_y: f32, pub scale: f32 }

#[derive(Clone, Debug)]
pub struct Param { pub raw: String, pub exprs: Vec<Expr> }
impl Param {
    pub fn new(raw: &str) -> Self {
        let exprs = parse_list(raw).map(|v| v.into_iter().map(Expr::new).collect()).unwrap_or_default();
        Param { raw: raw.trim().to_string(), exprs }
    }
}

#[derive(Clone, Debug)]
pub struct Controller {
    pub kind: String,
    pub label: String,
    pub trigger_all: Vec<Expr>,
    pub triggers: Vec<Vec<Expr>>,
    pub params: Vec<(String, Param)>,
    pub persistent: i32,
    pub ignore_hitpause: bool,
    pub uid: u32,
}
impl Controller {
    pub fn param(&self, k: &str) -> Option<&Param> { self.params.iter().find(|(n, _)| n == k).map(|(_, p)| p) }
    pub fn has(&self, k: &str, i: usize) -> bool { self.param(k).is_some_and(|p| p.exprs.len() > i) }
    pub fn raw(&self, k: &str) -> Option<&str> { self.param(k).map(|p| p.raw.as_str()) }
}

#[derive(Clone, Debug, Default)]
pub struct StateDef { pub id: i32, pub params: Vec<(String, Param)>, pub controllers: Vec<Controller> }
impl StateDef {
    pub fn param(&self, k: &str) -> Option<&Param> { self.params.iter().find(|(n, _)| n == k).map(|(_, p)| p) }
}

static UID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
pub fn next_uid() -> u32 { UID.fetch_add(1, Ordering::Relaxed) }

pub fn safe_expr(src: &str) -> Expr { Expr::new(parse(src).unwrap_or(Node::Num(0.0, false))) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source { Mugen, Native }

#[derive(Clone, Debug)]
pub struct MoveInfo { pub name: String, pub input: String, pub command: String }

#[derive(Debug)]
pub struct CharDef {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub author: String,
    pub source: Source,
    pub local_w: f32,
    pub local_h: f32,
    pub constants: HashMap<String, f64>,
    pub sprites: HashMap<(i32, i32), Sprite>,
    pub anims: HashMap<i32, Animation>,
    pub commands: Vec<CommandDef>,
    pub states: HashMap<i32, StateDef>,
    pub sounds: HashMap<(i32, i32), Vec<u8>>,
    pub portrait: Option<Sprite>,
    pub movelist: Vec<MoveInfo>,
    pub ai_commands: Vec<String>,
    pub color: [u8; 3],
    pub series: String,
    pub description: String,
    /// story lines (round-start face-off, victory) and the transformed form's name; `boss` = arcade final boss
    pub intro: String, pub taunt: String, pub win_quote: String, pub form_name: String, pub boss: bool,
}
impl CharDef {
    pub fn empty(id: &str) -> Self {
        CharDef {
            id: id.into(), name: id.into(), display_name: id.into(), author: String::new(), source: Source::Native, local_w: 320.0, local_h: 240.0,
            constants: default_constants(), sprites: HashMap::new(), anims: HashMap::new(), commands: vec![], states: HashMap::new(), sounds: HashMap::new(),
            portrait: None, movelist: vec![], ai_commands: vec![], color: [90, 100, 140], series: String::new(), description: String::new(),
            intro: String::new(), taunt: String::new(), win_quote: String::new(), form_name: String::new(), boss: false,
        }
    }
    pub fn c(&self, k: &str, d: f64) -> f64 { *self.constants.get(k).unwrap_or(&d) }
}

pub fn default_constants() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    let mut s = |k: &str, v: f64| { m.insert(k.to_string(), v); };
    s("data.life", 1000.0); s("data.power", 3000.0); s("data.attack", 100.0); s("data.defence", 100.0);
    s("data.fall.defence_up", 50.0); s("data.liedown.time", 60.0); s("data.airjuggle", 15.0); s("data.sparkno", 2.0); s("data.guard.sparkno", 40.0);
    s("size.xscale", 1.0); s("size.yscale", 1.0); s("size.ground.back", 15.0); s("size.ground.front", 16.0); s("size.air.back", 12.0); s("size.air.front", 12.0);
    s("size.height", 60.0); s("size.attack.dist", 160.0); s("size.proj.attack.dist", 90.0); s("size.head.pos.x", -5.0); s("size.head.pos.y", -90.0);
    s("size.mid.pos.x", -5.0); s("size.mid.pos.y", -60.0); s("size.shadowoffset", 0.0); s("size.draw.offset.x", 0.0); s("size.draw.offset.y", 0.0);
    s("velocity.walk.fwd.x", 2.4); s("velocity.walk.back.x", -2.2); s("velocity.run.fwd.x", 4.6); s("velocity.run.fwd.y", 0.0);
    s("velocity.run.back.x", -4.5); s("velocity.run.back.y", -3.8); s("velocity.jump.y", -8.4); s("velocity.jump.neu.x", 0.0);
    s("velocity.jump.back.x", -2.55); s("velocity.jump.fwd.x", 2.5); s("velocity.runjump.back.x", -2.55); s("velocity.runjump.back.y", -8.1);
    s("velocity.runjump.fwd.x", 4.0); s("velocity.runjump.fwd.y", -8.1); s("velocity.airjump.y", -8.1); s("velocity.airjump.neu.x", 0.0);
    s("velocity.airjump.back.x", -2.55); s("velocity.airjump.fwd.x", 2.5);
    s("movement.airjump.num", 1.0); s("movement.airjump.height", 35.0); s("movement.yaccel", 0.44);
    s("movement.stand.friction", 0.85); s("movement.crouch.friction", 0.82);
    m
}
