//! Character definition shared by MUGEN-loaded and UMF-compiled characters.
use crate::commands::CommandDef;
use crate::expr::{parse, parse_list, Expr, Node};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
pub use ume_formats::air::{AnimFrame, Animation, Blend, Box4};

static IMG_ID: AtomicU64 = AtomicU64::new(1);

/// RGBA image owned by the engine (uploaded to a GPU texture by the renderer, cached by `id`).
#[derive(Debug)]
pub struct ImageData {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    pub id: u64,
}
impl ImageData {
    pub fn new(w: u32, h: u32, rgba: Vec<u8>) -> Arc<Self> {
        Arc::new(ImageData {
            w,
            h,
            rgba,
            id: IMG_ID.fetch_add(1, Ordering::Relaxed),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Sprite {
    pub image: Option<Arc<ImageData>>,
    pub width: f32,
    pub height: f32,
    pub axis_x: f32,
    pub axis_y: f32,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub raw: String,
    pub exprs: Vec<Expr>,
}
thread_local! {
    static EXPR_CACHE: RefCell<HashMap<String, Expr>> = RefCell::new(HashMap::with_capacity(1024));
    static EXPR_LIST_CACHE: RefCell<HashMap<String, Vec<Expr>>> = RefCell::new(HashMap::with_capacity(1024));
}

pub fn parse_expr_list(raw: &str) -> Vec<Expr> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    EXPR_LIST_CACHE.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(list) = map.get(trimmed) {
            return list.clone();
        }
        let list: Vec<Expr> = parse_list(trimmed)
            .map(|v| v.into_iter().map(Expr::new).collect())
            .unwrap_or_default();
        if map.len() < 8192 {
            map.insert(trimmed.to_string(), list.clone());
        }
        list
    })
}

impl Param {
    pub fn new(raw: &str) -> Self {
        let trimmed = raw.trim();
        let exprs = parse_expr_list(trimmed);
        Param {
            raw: trimmed.to_string(),
            exprs,
        }
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
    pub fn param(&self, k: &str) -> Option<&Param> {
        crate::diagnostics::note_param_read(k);
        self.params.iter().find(|(n, _)| n == k).map(|(_, p)| p)
    }
    pub fn has(&self, k: &str, i: usize) -> bool {
        self.param(k).is_some_and(|p| p.exprs.len() > i)
    }
    pub fn raw(&self, k: &str) -> Option<&str> {
        self.param(k).map(|p| p.raw.as_str())
    }
}

#[derive(Clone, Debug, Default)]
pub struct StateDef {
    pub id: i32,
    pub params: Vec<(String, Param)>,
    pub controllers: Vec<Controller>,
}
impl StateDef {
    pub fn param(&self, k: &str) -> Option<&Param> {
        self.params.iter().find(|(n, _)| n == k).map(|(_, p)| p)
    }
}

static UID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
pub fn next_uid() -> u32 {
    UID.fetch_add(1, Ordering::Relaxed)
}

pub fn safe_expr(src: &str) -> Expr {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Expr::num(0.0);
    }
    EXPR_CACHE.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(e) = map.get(trimmed) {
            return e.clone();
        }
        let expr = Expr::new(parse(trimmed).unwrap_or(Node::Num(0.0, false)));
        if map.len() < 8192 {
            map.insert(trimmed.to_string(), expr.clone());
        }
        expr
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Mugen,
    Native,
}

#[derive(Debug)]
pub struct MoveInfo {
    pub name: String,
    pub input: String,
    pub command: String,
}

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
    pub intro: String,
    pub taunt: String,
    pub win_quote: String,
    pub form_name: String,
    pub boss: bool,
    pub mugen_version: f32,
    pub unknown_asserts: Vec<String>,
    pub source_spans: crate::diagnostics::SourceSpans,
    pub diagnostics_cache: std::sync::OnceLock<crate::diagnostics::Diagnostics>,
    pub palettes: HashMap<PalKey, Arc<Vec<u8>>>,
    pub indexed: HashMap<(i32, i32), IndexedSprite>,
    pub remap_cache: Mutex<HashMap<((i32, i32), PalKey), Arc<ImageData>>>,
    pub image_cache: Mutex<HashMap<(i32, i32), Arc<ImageData>>>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub size_ground_back: f32,
    pub size_ground_front: f32,
    pub size_air_back: f32,
    pub size_air_front: f32,
    pub size_height: f32,
    pub size_attack_dist: f32,
}
impl CharDef {
    pub fn empty(id: &str) -> Self {
        let mut d = CharDef {
            id: id.into(),
            name: id.into(),
            display_name: id.into(),
            author: String::new(),
            source: Source::Native,
            local_w: 320.0,
            local_h: 240.0,
            constants: default_constants(),
            sprites: HashMap::new(),
            anims: HashMap::new(),
            commands: vec![],
            states: HashMap::new(),
            sounds: HashMap::new(),
            portrait: None,
            movelist: vec![],
            ai_commands: vec![],
            color: [90, 100, 140],
            series: String::new(),
            description: String::new(),
            intro: String::new(),
            taunt: String::new(),
            win_quote: String::new(),
            form_name: String::new(),
            boss: false,
            mugen_version: 1.0,
            unknown_asserts: vec![],
            source_spans: Default::default(),
            diagnostics_cache: Default::default(),
            palettes: HashMap::new(),
            indexed: HashMap::new(),
            remap_cache: Mutex::new(HashMap::new()),
            image_cache: Mutex::new(HashMap::new()),
            scale_x: 1.0,
            scale_y: 1.0,
            size_ground_back: 15.0,
            size_ground_front: 16.0,
            size_air_back: 12.0,
            size_air_front: 12.0,
            size_height: 60.0,
            size_attack_dist: 160.0,
        };
        d.refresh_sizes();
        d
    }
    pub fn diagnostics(&self) -> &crate::diagnostics::Diagnostics {
        self.diagnostics_cache
            .get_or_init(|| crate::diagnostics::audit_char(self, &self.source_spans))
    }
    pub fn refresh_sizes(&mut self) {
        self.scale_x = self.c("size.xscale", 1.0) as f32;
        self.scale_y = self.c("size.yscale", 1.0) as f32;
        self.size_ground_back = self.c("size.ground.back", 15.0) as f32;
        self.size_ground_front = self.c("size.ground.front", 16.0) as f32;
        self.size_air_back = self.c("size.air.back", 12.0) as f32;
        self.size_air_front = self.c("size.air.front", 12.0) as f32;
        self.size_height = self.c("size.height", 60.0) as f32;
        self.size_attack_dist = self.c("size.attack.dist", 160.0) as f32;
    }

    pub fn c(&self, k: &str, d: f64) -> f64 {
        *self.constants.get(k).unwrap_or(&d)
    }

    pub fn remap_palette(
        &self,
        sprite: (i32, i32),
        remap: &[(PalKey, PalKey)],
    ) -> Option<&Arc<Vec<u8>>> {
        let ix = self.indexed.get(&sprite)?;
        let dst = remap.iter().rev().find(|(src, _)| *src == ix.pal)?.1;
        if dst == ix.pal {
            return None;
        }
        self.palettes.get(&dst)
    }

    pub fn remapped(
        &self,
        sprite: (i32, i32),
        remap: &[(PalKey, PalKey)],
    ) -> Option<Arc<ImageData>> {
        let ix = self.indexed.get(&sprite)?;
        let dst = remap.iter().rev().find(|(src, _)| *src == ix.pal)?.1;
        if dst == ix.pal {
            return None;
        }
        let pal = self.palettes.get(&dst)?;
        let mut cache = self.remap_cache.lock().ok()?;
        if let Some(img) = cache.get(&(sprite, dst)) {
            return Some(img.clone());
        }
        let mut rgba = vec![0u8; ix.pixels.len() * 4];
        for (o, &c) in rgba.as_chunks_mut::<4>().0.iter_mut().zip(ix.pixels.iter()) {
            if c == 0 {
                continue;
            }
            let p = c as usize * 4;
            o[..3].copy_from_slice(&pal[p..p + 3]);
            o[3] = 255;
        }
        let img = ImageData::new(ix.w, ix.h, rgba);
        cache.insert((sprite, dst), img.clone());
        Some(img)
    }

    /// RGBA copy of a sprite, materialised on first use. Indexed sprites are drawn straight from the index
    /// atlas, so this is only needed by paths that must own the pixels (reflections, thumbnails).
    pub fn rgba_sprite(&self, sprite: (i32, i32)) -> Option<Arc<ImageData>> {
        let spr = self.sprites.get(&sprite)?;
        if let Some(img) = &spr.image { return Some(img.clone()); }
        let ix = self.indexed.get(&sprite)?;
        let mut cache = self.image_cache.lock().ok()?;
        if let Some(img) = cache.get(&sprite) { return Some(img.clone()); }
        let pal: &[u8] = match ix.palette.as_deref() { Some(p) => p, None => self.palettes.get(&ix.pal).map(|p| p.as_slice())? };
        if pal.len() < 1024 { return None; }
        let mut rgba = vec![0u8; ix.pixels.len() * 4];
        for (o, &c) in rgba.chunks_exact_mut(4).zip(ix.pixels.iter()) {
            if c == 0 { continue; }
            let p = c as usize * 4;
            o[..3].copy_from_slice(&pal[p..p + 3]);
            o[3] = 255;
        }
        let img = ImageData::new(ix.w, ix.h, rgba);
        cache.insert(sprite, img.clone());
        Some(img)
    }
}

pub type PalKey = (i32, i32);

#[derive(Clone, Debug)]
pub struct IndexedSprite {
    pub w: u32,
    pub h: u32,
    pub pixels: Arc<Vec<u8>>,
    pub pal: PalKey,
    pub palette: Option<Arc<Vec<u8>>>,
    pub id: u64,
}

impl IndexedSprite {
    pub fn new(
        w: u32,
        h: u32,
        pixels: Arc<Vec<u8>>,
        pal: PalKey,
        palette: Option<Arc<Vec<u8>>>,
    ) -> Self {
        IndexedSprite {
            w,
            h,
            pixels,
            pal,
            palette,
            id: IMG_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

pub fn parse_mugen_version(raw: Option<&str>) -> f32 {
    let Some(v) = raw.map(str::trim).filter(|v| !v.is_empty()) else {
        return 1.0;
    };
    let mut parts = v.split('.');
    match (
        parts.next().and_then(|x| x.trim().parse::<u32>().ok()),
        parts
            .next()
            .and_then(|x| x.trim().chars().next())
            .and_then(|c| c.to_digit(10)),
    ) {
        (Some(major), Some(minor)) if major >= 1 && !v.contains(',') => {
            major as f32 + minor as f32 / 10.0
        }
        (Some(major), None) if major >= 1 && !v.contains(',') => major as f32,
        _ => 0.0,
    }
}

pub fn default_constants() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    let mut s = |k: &str, v: f64| {
        m.insert(k.to_string(), v);
    };
    s("data.life", 1000.0);
    s("data.power", 3000.0);
    s("data.attack", 100.0);
    s("data.defence", 100.0);
    s("data.fall.defence_up", 50.0);
    s("data.liedown.time", 60.0);
    s("data.airjuggle", 15.0);
    s("data.sparkno", 2.0);
    s("data.guard.sparkno", 40.0);
    s("size.xscale", 1.0);
    s("size.yscale", 1.0);
    s("size.ground.back", 15.0);
    s("size.ground.front", 16.0);
    s("size.air.back", 12.0);
    s("size.air.front", 12.0);
    s("size.height", 60.0);
    s("size.attack.dist", 160.0);
    s("size.proj.attack.dist", 90.0);
    s("size.head.pos.x", -5.0);
    s("size.head.pos.y", -90.0);
    s("size.mid.pos.x", -5.0);
    s("size.mid.pos.y", -60.0);
    s("size.shadowoffset", 0.0);
    s("size.draw.offset.x", 0.0);
    s("size.draw.offset.y", 0.0);
    s("velocity.walk.fwd.x", 2.4);
    s("velocity.walk.back.x", -2.2);
    s("velocity.run.fwd.x", 4.6);
    s("velocity.run.fwd.y", 0.0);
    s("velocity.run.back.x", -4.5);
    s("velocity.run.back.y", -3.8);
    s("velocity.jump.y", -8.4);
    s("velocity.jump.neu.x", 0.0);
    s("velocity.jump.back.x", -2.55);
    s("velocity.jump.fwd.x", 2.5);
    s("velocity.runjump.back.x", -2.55);
    s("velocity.runjump.back.y", -8.1);
    s("velocity.runjump.fwd.x", 4.0);
    s("velocity.runjump.fwd.y", -8.1);
    s("velocity.airjump.y", -8.1);
    s("velocity.airjump.neu.x", 0.0);
    s("velocity.airjump.back.x", -2.55);
    s("velocity.airjump.fwd.x", 2.5);
    s("movement.airjump.num", 1.0);
    s("movement.airjump.height", 35.0);
    s("movement.yaccel", 0.44);
    s("movement.stand.friction", 0.85);
    s("movement.crouch.friction", 0.82);
    s("default.lifetoredlifemul", 0.75);
    s("super.lifetoredlifemul", 0.75);
    s("default.lifetodizzypointsmul", 1.8);
    s("super.lifetodizzypointsmul", 0.0);
    s("default.lifetoguardpointsmul", 1.5);
    s("super.lifetoguardpointsmul", -0.33);
    // M.U.G.E.N. 1.1 common1 constants: without them a bouncing / recovering fighter gets 0 gravity and flies off (frozen in 5101 etc.)
    s("movement.down.bounce.yaccel", 0.4);
    s("movement.down.bounce.groundlevel", 12.0);
    s("movement.down.bounce.offset.x", 0.0);
    s("movement.down.bounce.offset.y", 20.0);
    s("movement.down.friction.threshold", 0.05);
    s("movement.air.gethit.groundlevel", 25.0);
    s("movement.air.gethit.groundrecover.ground.threshold", -20.0);
    s("movement.air.gethit.groundrecover.groundlevel", 10.0);
    s("movement.air.gethit.airrecover.threshold", -1.0);
    s("movement.air.gethit.airrecover.yaccel", 0.35);
    s("movement.air.gethit.trip.groundlevel", 15.0);
    m
}
