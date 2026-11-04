//! AAA-style impact feedback: layered particles (sparks, glows, dust, debris), shockwave rings, damage numbers, banners,
//! camera punch/KO zoom, slow-motion, screen-shake "trauma", and the parameters fed to the post-processing pass.
//! Everything here is presentation only — the deterministic simulation is untouched.
use crate::poly;
use crate::render::Gfx;
use crate::ui::Ui;
use crate::gfx::*;
use ume_engine::matchs::{Event, Match};

#[derive(Clone, Copy, PartialEq)]
enum Kind { Spark, Glow, Dust, Debris, Ember, Slash }
struct P { k: Kind, x: f32, y: f32, vx: f32, vy: f32, g: f32, life: f32, max: f32, size: f32, col: [f32; 3], rot: f32, vrot: f32 }
struct Ring { x: f32, y: f32, r0: f32, r1: f32, t: f32, dur: f32, w: f32, col: [f32; 3] }
/// Dynamic stage lighting from impacts (Tekken 8: hit effects "light up the stage like fireworks").
struct Light { x: f32, y: f32, r: f32, t: f32, dur: f32, col: [f32; 3] }
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Weather { #[default] None, Rain, Fireflies, Wisps, Wind }
impl Weather {
    pub fn for_stage(name: &str) -> Weather {
        let n = name.to_lowercase();
        if n.contains("neo-metr") || n.contains("neon") { Weather::Rain } else if n.contains("bambu") { Weather::Fireflies } else if n.contains("espírit") || n.contains("espirit") { Weather::Wisps } else if n.contains("planalto") { Weather::Wind } else { Weather::None }
    }
}
struct Float { x: f32, y: f32, t: f32, text: String, size: f32, col: [f32; 3] }
pub struct Banner { pub text: String, pub t: f32, pub col: [f32; 3], pub sub: String }

#[derive(Default, Clone, Copy)]
struct Snap { grounded: bool, vy: f32, vx: f32, move_type: char }

/// What the post-processing pass should do this frame.
#[derive(Clone, Copy, Default)]
pub struct FxParams { pub invert: f32, pub chroma: f32, pub vignette: f32, pub bloom: f32, pub saturation: f32, pub flash: f32, pub radial: f32, pub center: [f32; 2], pub tint: [f32; 3] }

pub struct Vfx {
    lights: Vec<Light>, pub weather: Weather, clock: f32, invert_t: f32, dist_zoom: f32, dist_target: f32,
    parts: Vec<P>, rings: Vec<Ring>, floats: Vec<Float>, pub banners: Vec<Banner>,
    pub trauma: f32, punch: f32, ko_t: f32, ko_focus: (f32, f32), pub slow_t: f32,
    chroma: f32, flash: f32, radial: f32, vig: f32, lines: f32, lines_col: [f32; 3], sat_pulse: f32,
    super_t: f32, super_owner: u32,
    /// face-off camera push-in at the start of the first round (target is set by the session, eased in `update`)
    pub intro_target: f32, intro_zoom: f32,
    /// cinematic super camera 0..1 (eased in while the super pause runs, eased out afterwards: build-up + follow-through)
    sz: f32, super_active: bool,
    /// per-fighter squash (>0) / stretch (<0) amount, decays to 0 — landings squash, take-offs stretch
    squash: Vec<f32>,
    /// sound requests produced by the effects layer (drained by the session)
    pub sfx: Vec<&'static str>,
    combo: [i32; 2], meter: [i32; 2], first_hit: bool,
    /// seconds since the last large full-screen flash / colour inversion (WCAG: no more than 3 flashes per second)
    flash_clock: f32,
    snap: Vec<Snap>, pre_mt: Vec<char>, rng: u32, tick: u32,
}

const MAX_FLOATS: usize = 6;
const FLOAT_LIFE: f32 = 0.7;
const MAX_PARTS: usize = 400;
const MAX_RINGS: usize = 10;
const MAX_LIGHTS: usize = 5;

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

impl Vfx {
    pub fn new() -> Self {
        Vfx { lights: vec![], weather: Weather::None, clock: 0.0, invert_t: 0.0, dist_zoom: 0.0, dist_target: 0.0, parts: vec![], rings: vec![], floats: vec![], banners: vec![], trauma: 0.0, punch: 0.0, ko_t: 0.0, ko_focus: (0.0, 0.0), slow_t: 0.0, chroma: 0.0, flash: 0.0, radial: 0.0, vig: 0.0, lines: 0.0,
            lines_col: [1.0, 1.0, 1.0], sat_pulse: 0.0, super_t: 0.0, super_owner: 0, intro_target: 0.0, intro_zoom: 0.0, sz: 0.0, super_active: false, squash: vec![], sfx: vec![], combo: [0; 2], meter: [0; 2], first_hit: false, flash_clock: 1.0, snap: vec![], pre_mt: vec![], rng: 0x9e37_79b9, tick: 0 }
    }
    fn r(&mut self) -> f32 { let mut x = self.rng; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rng = x; x as f32 / 4294967296.0 }
    fn rr(&mut self, a: f32, b: f32) -> f32 { a + (b - a) * self.r() }

    /// Scale factors (x, y) for fighter `i` from its squash/stretch state; feet stay planted (sprites are anchored at the axis).
    pub fn squash_scale(&self, i: usize) -> [f32; 2] { let q = self.squash.get(i).copied().unwrap_or(0.0); [1.0 + 0.09 * q, 1.0 - 0.12 * q] }
    pub fn ko_active(&self) -> bool { self.ko_t > 0.0 }
    /// Real-time scale applied to the fixed-step accumulator (KO slow motion).
    pub fn time_scale(&self) -> f32 {
        if self.slow_t <= 0.0 { return 1.0 }
        // hold at 25% speed, then ease back to normal
        let t = self.slow_t;
        if t > 0.5 { 0.25 } else { lerp(1.0, 0.25, t / 0.5) }
    }

    fn spark_burst(&mut self, x: f32, y: f32, dir: f32, n: usize, speed: f32, col: [f32; 3], spread: f32) {
        for _ in 0..n {
            let a = (self.rr(-spread, spread)) + if dir >= 0.0 { 0.0 } else { std::f32::consts::PI };
            let s = self.rr(0.4, 1.0) * speed;
            let life = self.rr(0.18, 0.42);
            let c = [lerp(col[0], 1.0, self.r() * 0.5), lerp(col[1], 1.0, self.r() * 0.5), lerp(col[2], 1.0, self.r() * 0.4)];
            let size = self.rr(0.6, 1.3);
            self.parts.push(P { k: Kind::Spark, x, y, vx: a.cos() * s, vy: a.sin() * s - 0.4, g: 0.12, life, max: life, size, col: c, rot: 0.0, vrot: 0.0 });
        }
    }
    fn dust(&mut self, x: f32, y: f32, n: usize, spread: f32, up: f32, col: [f32; 3]) {
        for _ in 0..n {
            let life = self.rr(0.35, 0.7);
            let (vx, vy, sz) = (self.rr(-spread, spread), -self.rr(0.1, up), self.rr(2.0, 4.5));
            let jx = self.rr(-4.0, 4.0);
            self.parts.push(P { k: Kind::Dust, x: x + jx, y, vx, vy, g: -0.004, life, max: life, size: sz, col, rot: 0.0, vrot: 0.0 });
        }
    }
    fn debris(&mut self, x: f32, y: f32, n: usize, col: [f32; 3]) {
        for _ in 0..n {
            let life = self.rr(0.5, 0.95);
            let (vx, vy, sz, vr) = (self.rr(-2.2, 2.2), -self.rr(1.5, 4.2), self.rr(0.8, 1.8), self.rr(-0.4, 0.4));
            self.parts.push(P { k: Kind::Debris, x, y, vx, vy, g: 0.22, life, max: life, size: sz, col, rot: 0.0, vrot: vr });
        }
    }
    /// Colour of a fighter's awakened aura: its own colour pushed towards bright.
    pub fn form_color(f: &ume_engine::fighter::Fighter) -> [f32; 3] {
        // the character's colour at full saturation, a touch towards white
        let c = [f.def.color[0] as f32 / 255.0, f.def.color[1] as f32 / 255.0, f.def.color[2] as f32 / 255.0];
        let mx = c[0].max(c[1]).max(c[2]).max(0.2);
        [lerp(c[0] / mx, 1.0, 0.18), lerp(c[1] / mx, 1.0, 0.18), lerp(c[2] / mx, 1.0, 0.18)]
    }
    fn ember(&mut self, x: f32, y: f32, vx: f32, vy: f32, g: f32, life: f32, size: f32, col: [f32; 3]) { self.parts.push(P { k: Kind::Ember, x, y, vx, vy, g, life, max: life, size, col, rot: 0.0, vrot: 0.0 }); }
    fn light(&mut self, x: f32, y: f32, r: f32, dur: f32, col: [f32; 3]) { if self.lights.len() >= MAX_LIGHTS { self.lights.remove(0); } self.lights.push(Light { x, y, r, t: 0.0, dur, col }); }
    /// Anime "impact frame" (colour inversion for a few ms) — used on super activation and KO.
    pub fn force_impact(&mut self) { self.invert_t = 0.075; self.flash_clock = 0.0; }
    /// Large flashes are spaced >= 0.34 s apart (<= 3 per second); small ones pass through.
    fn flash_add(&mut self, v: f32) {
        if v >= 0.3 { if self.flash_clock < 0.34 { return; } self.flash_clock = 0.0; }
        self.flash = self.flash.max(v);
    }
    #[cfg(test)] pub fn flash_level(&self) -> f32 { self.flash }
    fn slashes(&mut self, x: f32, y: f32, dir: f32, n: usize, len: f32, col: [f32; 3]) {
        for i in 0..n {
            let a = (if dir >= 0.0 { 0.0 } else { std::f32::consts::PI }) + self.rr(-0.9, 0.9) + if i % 2 == 0 { 0.0 } else { std::f32::consts::PI };
            let l = len * self.rr(0.6, 1.2);
            self.parts.push(P { k: Kind::Slash, x, y, vx: 0.0, vy: 0.0, g: 0.0, life: 0.13, max: 0.13, size: l, col, rot: a, vrot: 0.0 });
        }
    }
    fn ring(&mut self, x: f32, y: f32, r1: f32, dur: f32, w: f32, col: [f32; 3]) { if self.rings.len() >= MAX_RINGS { self.rings.remove(0); } self.rings.push(Ring { x, y, r0: 2.0, r1, t: 0.0, dur, w, col }); }
    fn glow(&mut self, x: f32, y: f32, size: f32, life: f32, col: [f32; 3]) { self.parts.push(P { k: Kind::Glow, x, y, vx: 0.0, vy: 0.0, g: 0.0, life, max: life, size, col, rot: 0.0, vrot: 0.0 }); }
    fn float(&mut self, x: f32, y: f32, text: String, size: f32, col: [f32; 3]) {
        if self.floats.len() >= MAX_FLOATS { self.floats.remove(0); }
        self.floats.push(Float { x, y, t: 0.0, text, size, col });
    }
    fn tag(&mut self, x: f32, y: f32, text: &str, col: [f32; 3]) {
        if self.floats.iter().any(|f| f.text == text && f.t < 0.35) { return; }
        self.float(x, y, text.to_string(), 6.0, col);
    }
    pub fn banner(&mut self, text: &str, sub: &str, col: [f32; 3]) {
        // never stack the same banner twice (e.g. both fighters landing a counter in the same exchange)
        if self.banners.iter().any(|b| b.text == text && b.t < 0.8) { return; }
        self.banners.push(Banner { text: text.into(), sub: sub.into(), t: 0.0, col });
    }

    /// Call right before `Match::tick`: remember who was attacking (for COUNTER detection).
    pub fn pre_tick(&mut self, m: &Match) {
        self.pre_mt = m.fighters.iter().map(|f| f.move_type).collect();
    }

    fn chest(m: &Match, side: usize) -> Option<(f32, f32, f32)> {
        m.roots().find(|f| f.side == side).map(|f| Self::chest_of(f))
    }
    fn chest_of(f: &ume_engine::fighter::Fighter) -> (f32, f32, f32) { (f.pos.x, f.pos.y - f.def.c("size.height", 60.0) as f32 * 0.55, f.facing) }

    /// Feed one simulation event.
    pub fn event(&mut self, ev: &Event, m: &Match) {
        match ev {
            Event::Hit { damage, guarded, heavy, attacker, defender, .. } => {
                let Some(df) = m.fighters.get(*defender) else { return };
                let (dx, dy, dface) = Self::chest_of(df);
                let dir = if m.fighters.get(*attacker).is_some_and(|a| a.pos.x <= dx) { 1.0 } else { -1.0 };
                let (x, y) = (dx + dface * 7.0, dy);
                let mag = (damage / 90.0).clamp(0.2, 1.4);
                if *guarded {
                    self.spark_burst(x, y, dir, 5, 3.0, [0.45, 0.75, 1.0], 0.9);
                    self.ring(x, y, 14.0, 0.2, 1.2, [0.5, 0.8, 1.0]);
                    self.trauma = (self.trauma + 0.05).min(1.0);
                    self.tag(x, y - 14.0, "GUARD", [0.55, 0.8, 1.0]);
                    return;
                }
                let counter = self.pre_mt.get(*defender).copied() == Some('A');
                let n = 5 + (mag * 8.0) as usize;
                self.spark_burst(x, y, dir, n, 3.0 + mag * 3.5, [1.0, 0.62, 0.15], 1.0);
                self.spark_burst(x, y, dir, 2, 5.0, [1.0, 1.0, 0.9], 0.4);
                self.glow(x, y, 12.0 + mag * 10.0, 0.12, [1.0, 0.75, 0.35]);
                self.slashes(x, y, dir, 2 + (mag * 2.0) as usize, 16.0 + mag * 10.0, [1.0, 0.95, 0.8]);
                self.ring(x, y, 16.0 + mag * 18.0, 0.24, 1.4, [1.0, 0.85, 0.5]);
                self.trauma = (self.trauma + 0.07 + mag * 0.12).min(0.6);
                self.punch = self.punch.max(0.008 + mag * 0.012);
                if *heavy || mag > 0.75 {
                    self.debris(x, y + 10.0, 4, [0.8, 0.72, 0.6]);
                    self.flash_add(0.1);
                    self.punch = self.punch.max(0.022);
                }
                if !self.first_hit && m.round_state == 2 && !m.training { self.first_hit = true; self.tag(x, y - 26.0, "FIRST ATTACK", [0.4, 0.9, 1.0]); }
                let col = if counter { [1.0, 0.45, 0.35] } else { [1.0, 0.92, 0.55] };
                self.float(x + dir * 12.0, y - 6.0, format!("{}", *damage as i32), 6.0 + mag * 2.5, col);
                if counter { self.sfx.push("counter"); self.tag(x, y - 14.0, "COUNTER", [1.0, 0.45, 0.35]); self.ring(x, y, 18.0, 0.2, 1.2, [1.0, 0.45, 0.35]); }
            }
            Event::Super { fighter, .. } => {
                self.super_t = 1.0; self.super_owner = *fighter;
                if let Some((x, y, _)) = m.fighters.iter().find(|f| f.id == *fighter).map(Self::chest_of) {
                    self.ring(x, y, 70.0, 0.5, 2.4, [1.0, 0.9, 0.4]);
                    self.ring(x, y, 120.0, 0.6, 1.6, [1.0, 0.8, 0.35]);
                    self.glow(x, y, 36.0, 0.3, [1.0, 0.8, 0.3]);
                    self.spark_burst(x, y, 1.0, 22, 8.0, [1.0, 0.85, 0.4], std::f32::consts::PI);
                    self.debris(x, y + 18.0, 6, [0.75, 0.68, 0.6]);
                    self.light(x, y, 160.0, 0.5, [1.0, 0.8, 0.35]);
                    self.sfx.push("super");
                    let nm = m.fighters.iter().find(|f| f.id == *fighter).map(|f| f.def.display_name.to_uppercase()).unwrap_or_default();
                    self.tag(x, y - 46.0, &format!("SUPER · {nm}"), [1.0, 0.82, 0.3]);
                }
                self.flash_add(0.25); self.chroma = self.chroma.max(0.5); self.radial = 0.45; self.trauma = (self.trauma + 0.15).min(0.6);
                self.lines = 0.7; self.lines_col = [1.0, 0.85, 0.4];
            }
            Event::Ko { loser_side } => {
                self.slow_t = 1.7; self.ko_t = 1.0;
                if let Some((x, y, _)) = Self::chest(m, *loser_side) {
                    self.ko_focus = (x, y);
                    self.ring(x, y, 110.0, 0.8, 3.0, [1.0, 0.25, 0.3]);
                    self.glow(x, y, 44.0, 0.4, [1.0, 0.3, 0.3]);
                    self.light(x, y, 160.0, 0.5, [1.0, 0.3, 0.3]);
                    self.spark_burst(x, y, 1.0, 14, 7.0, [1.0, 0.4, 0.3], std::f32::consts::PI);
                    self.debris(x, y + 12.0, 6, [0.7, 0.6, 0.55]);
                }
                self.flash_add(0.5); self.chroma = 0.6; self.radial = 0.6; self.trauma = 0.6; self.vig = 0.8; self.lines = 0.8; self.lines_col = [1.0, 0.35, 0.3];
                let w = 1 - loser_side;
                if m.roots().find(|f| f.side == w).is_some_and(|f| f.life >= f.life_max - 0.5) { self.sfx.push("perfect"); self.banner("PERFECT!", "vitória sem sofrer dano", [1.0, 0.85, 0.2]); }
            }
            Event::Transform { fighter } => {
                self.super_t = 1.0; self.super_owner = *fighter;
                let mut col = [1.0, 0.85, 0.4];
                if let Some(f) = m.fighters.iter().find(|f| f.id == *fighter) {
                    col = Self::form_color(f);
                    let (x, y, _) = Self::chest_of(f);
                    for (i, r) in [70.0f32, 140.0].iter().enumerate() { self.ring(x, y, *r, 0.55 + i as f32 * 0.2, 2.8 - i as f32 * 0.8, col); }
                    self.glow(x, y, 70.0, 0.7, col); self.light(x, y, 200.0, 1.0, col);
                    for _ in 0..28 {
                        let (px, life) = (f.pos.x + self.rr(-20.0, 20.0), self.rr(0.7, 1.4));
                        let (vy, sz, c) = (-self.rr(1.2, 4.0), self.rr(1.0, 2.0), if self.r() > 0.4 { col } else { [1.0, 1.0, 1.0] });
                        let vx = self.rr(-0.3, 0.3); self.ember(px, f.pos.y, vx, vy, -0.01, life, sz, c);
                    }
                    self.spark_burst(x, y, 1.0, 20, 8.0, col, std::f32::consts::PI);
                    self.debris(f.pos.x, f.pos.y, 8, [0.7, 0.65, 0.6]);
                    self.dust(f.pos.x, f.pos.y, 12, 3.0, 1.0, [0.85, 0.8, 0.75]);
                    let nm = f.def.form_name.to_uppercase();
                    self.tag(x, y - 46.0, if nm.is_empty() { "DESPERTAR" } else { &nm }, col);
                }
                self.flash_add(0.4); self.chroma = 0.6; self.radial = 0.5; self.trauma = 0.5; self.vig = 0.3;
                self.lines = 0.7; self.lines_col = col; self.sfx.push("transform");
            }
            Event::Fight => { if m.roots().any(|f| f.def.boss) { self.banner("CHEFE FINAL", "MIKAZUKI OBORO", [1.0, 0.15, 0.3]); self.sfx.push("ko"); } self.ring(m.camera.x, m.stage.zoffset - 60.0, 120.0, 0.5, 2.0, [1.0, 0.8, 0.3]); self.flash_add(0.2); self.chroma = self.chroma.max(0.4); self.trauma = (self.trauma + 0.1).min(1.0); self.radial = 0.3; }
            Event::RoundStart(_) => { self.slow_t = 0.0; self.ko_t = 0.0; self.first_hit = false; }
            _ => {}
        }
    }

    /// Per simulation tick: landing / run dust, super aura.
    pub fn tick(&mut self, m: &Match) {
        self.tick = self.tick.wrapping_add(1);
        { let xs: Vec<f32> = m.roots().filter(|f| f.alive).map(|f| f.pos.x).collect(); if xs.len() >= 2 { let (lo, hi) = (xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max)); self.dist_target = ((260.0 - (hi - lo)) / 260.0).clamp(0.0, 1.0) * 0.07; } }
        self.snap.resize(m.fighters.len(), Snap::default());
        self.squash.resize(m.fighters.len(), 0.0);
        for (side, (fi, f)) in m.fighters.iter().enumerate().filter(|(_, f)| !f.is_helper).enumerate() {
            let _ = side;
            let now = Snap { grounded: f.state_type != 'A', vy: f.vel.y, vx: f.vel.x, move_type: f.move_type };
            let prev = self.snap[fi];
            if !prev.grounded && now.grounded { let n = 6 + (prev.vy.abs() * 1.6) as usize; self.dust(f.pos.x, f.pos.y, n.min(16), 1.6, 0.5, [0.85, 0.82, 0.78]); if prev.vy > 3.0 { self.sfx.push("land"); }
            if let Some(q) = self.squash.get_mut(fi) { *q = (prev.vy / 7.0).clamp(0.35, 1.0); }
 if prev.vy > 5.0 { self.ring(f.pos.x, f.pos.y - 1.0, 26.0, 0.25, 1.2, [0.9, 0.9, 0.9]); self.trauma = (self.trauma + 0.1).min(1.0); } }
            if prev.grounded && !now.grounded && now.vy < -2.0 { self.sfx.push("jump"); if let Some(q) = self.squash.get_mut(fi) { *q = -0.8; } self.dust(f.pos.x, f.pos.y, 5, 1.2, 0.3, [0.85, 0.82, 0.78]); }
            if now.grounded && now.vx.abs() > 3.2 && prev.vx.abs() <= 3.2 && f.state_type == 'S' { self.sfx.push("dash"); }
            if now.grounded && now.vx.abs() > 3.2 && self.tick % 4 == 0 && f.state_type == 'S' { let b = -f.facing; self.dust(f.pos.x + b * 6.0, f.pos.y, 2, 0.6, 0.25, [0.85, 0.82, 0.78]); }
            if f.move_type == 'H' && f.state_type == 'L' && !prev.grounded == false && prev.move_type != 'H' { self.dust(f.pos.x, f.pos.y, 9, 2.0, 0.4, [0.85, 0.82, 0.78]); }
            self.snap[fi] = now;
        }
        // combo pings (rising pitch) and super-meter level-up chime
        for side in 0..2 {
            let Some(att) = m.roots().find(|f| f.side == side) else { continue };
            let Some(def) = m.roots().find(|f| f.side != side) else { continue };
            let hits = if def.move_type == 'H' && def.ghv.hitcount >= 2 { def.ghv.hitcount } else { 0 };
            if hits > self.combo[side] && hits >= 2 { let n = hits.min(6); self.sfx.push(["combo1", "combo1", "combo2", "combo3", "combo4", "combo5", "combo6"][n as usize]); }
            self.combo[side] = hits;
            let lv = (att.power / 1000.0).floor() as i32;
            if lv > self.meter[side] && m.round_state == 2 { self.sfx.push("meter"); }
            self.meter[side] = lv;
        }
        self.super_active = m.pause.as_ref().is_some_and(|p| p.is_super);
        // awakened fighters burn with rising embers and crackling arcs; defeats get their own beat
        for f in m.fighters.iter().filter(|f| !f.is_helper) {
            if (f.form > 0 || f.def.boss) && f.alive && self.tick % if f.form > 0 { 2 } else { 4 } == 0 {
                let col = Self::form_color(f);
                let (px, life) = (f.pos.x + self.rr(-14.0, 14.0), self.rr(0.5, 1.0));
                let (vy, sz) = (-self.rr(0.8, 2.4), self.rr(0.7, 1.6));
                let (ey, vx) = (f.pos.y + self.rr(-90.0, -5.0), self.rr(-0.2, 0.2)); self.ember(px, ey, vx, vy, -0.01, life, sz, col);
                if self.tick % 18 == 0 { let (x, y, _) = Self::chest_of(f); self.slashes(x, y, 1.0, 3, 34.0, col); }
            }
            if (5190..=5193).contains(&f.state_no) && !f.alive {
                if f.state_time == 1 { self.sfx.push("defeat"); self.dust(f.pos.x, f.pos.y, 8, 1.4, 0.3, [0.7, 0.68, 0.7]); }
                if f.state_time == 52 { self.dust(f.pos.x, f.pos.y, 10, 1.8, 0.4, [0.8, 0.78, 0.76]); self.ring(f.pos.x, f.pos.y - 1.0, 34.0, 0.4, 1.4, [0.75, 0.75, 0.8]); }
                // the match-deciding defeat: slow push-in on the fallen fighter and pale motes drifting up
                if f.state_no == 5193 {
                    if f.state_time == 1 { let (x, y, _) = Self::chest_of(f); self.ko_focus = (x, y); self.ko_t = 1.0; self.slow_t = 1.4; }
                    if f.state_time > 20 && self.tick % 5 == 0 { let (px, life) = (f.pos.x + self.rr(-20.0, 20.0), self.rr(0.8, 1.4)); let (vy, sz) = (-self.rr(0.3, 0.9), self.rr(0.6, 1.2)); self.ember(px, f.pos.y - 6.0, 0.0, vy, -0.004, life, sz, [0.75, 0.8, 0.95]); }
                }
            }
        }
        // super aura: rising energy motes while the super pause runs
        if let Some(p) = &m.pause { if p.is_super {
            if let Some((x, y, _)) = m.fighters.iter().find(|f| f.id == self.super_owner).map(Self::chest_of) {
                self.trauma = self.trauma.max(0.12);
                if self.tick % 10 == 0 { self.slashes(x, y, 1.0, 1, 40.0, [1.0, 0.9, 0.5]); }
                for _ in 0..2 {
                    let ang = self.rr(0.0, std::f32::consts::TAU); let rr = self.rr(14.0, 34.0);
                    let c = if self.r() > 0.5 { [1.0, 0.85, 0.35] } else { [0.5, 0.85, 1.0] };
                    let life = self.rr(0.4, 0.8);
                    let (px, py, vy) = (x + ang.cos() * rr, y + 26.0 + self.rr(-6.0, 14.0), -self.rr(0.8, 2.2));
                    let size = self.rr(0.8, 1.8);
                    self.parts.push(P { k: Kind::Ember, x: px, y: py, vx: 0.0, vy, g: -0.01, life, max: life, size, col: c, rot: 0.0, vrot: 0.0 });
                }
                if self.tick % 28 == 0 { self.ring(x, y, 40.0, 0.4, 1.2, [1.0, 0.9, 0.5]); }
            }
            self.lines = self.lines.max(0.3);
        } }
    }

    /// Per rendered frame (real time).
    pub fn update(&mut self, dt: f32) {
        let k = dt * 60.0;
        for p in &mut self.parts {
            p.life -= dt; p.x += p.vx * k; p.y += p.vy * k; p.vy += p.g * k; p.rot += p.vrot * k;
            if p.k == Kind::Spark { p.vx *= 0.96f32.powf(k); }
            if p.k == Kind::Dust { p.vx *= 0.94f32.powf(k); p.size += 0.05 * k; }
        }
        self.parts.retain(|p| p.life > 0.0);
        self.clock += dt;
        for q in &mut self.squash { *q *= 0.80f32.powf(dt * 60.0); if q.abs() < 0.01 { *q = 0.0; } }
        // DBFZ-style: slow ease-in to the close-up, even slower pull-back
        let (target, rate) = if self.super_active { (1.0, 5.0) } else { (0.0, 2.2) };
        self.sz += (target - self.sz) * (rate * dt).min(1.0);
        self.invert_t = (self.invert_t - dt).max(0.0);
        self.flash_clock += dt;
        self.dist_zoom += (self.dist_target - self.dist_zoom) * (3.0 * dt).min(1.0);
        self.intro_zoom += (self.intro_target - self.intro_zoom) * (2.5 * dt).min(1.0);
        for l in &mut self.lights { l.t += dt; }
        self.lights.retain(|l| l.t < l.dur);
        for r in &mut self.rings { r.t += dt; }
        self.rings.retain(|r| r.t < r.dur);
        for f in &mut self.floats { f.t += dt; f.y -= 0.35 * k * (1.0 - (f.t / FLOAT_LIFE).min(1.0)); }
        self.floats.retain(|f| f.t < FLOAT_LIFE);
        for b in &mut self.banners { b.t += dt; }
        self.banners.retain(|b| b.t < 1.6);
        self.trauma = (self.trauma - dt * 1.6).max(0.0);
        self.punch -= self.punch * (9.0 * dt).min(1.0);
        self.chroma = (self.chroma - dt * 3.2).max(0.0);
        self.flash = (self.flash - dt * 2.6).max(0.0);
        self.radial = (self.radial - dt * 2.4).max(0.0);
        self.vig = (self.vig - dt * 0.5).max(0.0);
        self.lines = (self.lines - dt * 3.0).max(0.0);
        self.super_t = (self.super_t - dt * 1.2).max(0.0);
        if self.slow_t > 0.0 { self.slow_t = (self.slow_t - dt).max(0.0); }
        if self.slow_t <= 0.0 { self.ko_t = (self.ko_t - dt * 1.5).max(0.0); }
        self.sat_pulse = if self.ko_t > 0.0 { self.ko_t } else { 0.0 };
        self.parts.truncate(MAX_PARTS);
    }

    /// Camera: zoom factor and focus in game-view coordinates.
    pub fn camera(&self, m: &Match, gain: f32) -> (f32, (f32, f32)) {
        let mid = {
            let live: Vec<_> = m.roots().filter(|f| f.alive).collect();
            let all: Vec<_> = if live.is_empty() { m.roots().collect() } else { live };
            if all.is_empty() { (m.camera.x, -35.0) } else { let n = all.len() as f32; (all.iter().map(|f| f.pos.x).sum::<f32>() / n, all.iter().map(|f| f.pos.y).sum::<f32>() / n - 35.0) }
        };
        let ko = if self.ko_t > 0.0 { let e = (self.ko_t.min(1.0)).powf(0.6); e * 0.28 } else { 0.0 };
        let sup = self.sz * self.sz * (3.0 - 2.0 * self.sz) * 0.2;
        let z = 1.0 + (self.punch + ko + sup + self.dist_zoom + self.intro_zoom) * gain;
        let mid = if self.sz > 0.01 { match m.fighters.iter().find(|f| f.id == self.super_owner).map(Self::chest_of) { Some((x, y, _)) => (lerp(mid.0, x, self.sz), lerp(mid.1, y, self.sz)), None => mid } } else { mid };
        let focus = if self.ko_t > 0.0 { let e = self.ko_t.min(1.0); (lerp(mid.0, self.ko_focus.0, e), lerp(mid.1, self.ko_focus.1, e)) } else { mid };
        (z, focus)
    }

    pub fn shake(&mut self, gain: f32) -> (f32, f32) {
        let t = self.trauma * self.trauma * gain;
        if t <= 0.001 { return (0.0, 0.0); }
        (self.rr(-1.0, 1.0) * t * 5.0, self.rr(-1.0, 1.0) * t * 4.0)
    }

    pub fn params(&self, gain: f32, focus_screen: (f32, f32), bloom: f32) -> FxParams {
        FxParams {
            invert: if self.invert_t > 0.0 { gain.min(1.0) } else { 0.0 },
            chroma: (self.chroma * gain).min(1.5),
            vignette: 0.32 + self.vig * 0.45 * gain + self.super_t * 0.25 * gain,
            bloom: bloom * (0.55 + self.flash * 0.6 * gain),
            saturation: 1.06 - self.sat_pulse * 0.45 * gain,
            flash: self.flash * 0.35 * gain,
            radial: (self.radial * 0.9 + self.super_t * 0.45) * gain,
            center: [focus_screen.0, focus_screen.1],
            tint: [1.0 + self.super_t * 0.05, 1.0 + self.super_t * 0.02 - self.sat_pulse * 0.04, 1.0 - self.sat_pulse * 0.08],
        }
    }

    /// Behind the fighters: awakened-form flames and light pillar, colour paint splashes.
    pub fn draw_back(&self, g: &Gfx, m: &Match, shx: f32, shy: f32) {
        let s = g.view_scale;
        let tp = |x: f32, y: f32| vec2(g.ox + (x - m.camera.x + m.view_w / 2.0 + shx) * s, g.oy + (y - m.camera.y + m.stage.zoffset + shy) * s);
        for f in m.fighters.iter().filter(|f| !f.is_helper && (f.form > 0 || f.def.boss) && f.alive) {
            let col = Self::form_color(f);
            // the final boss always smoulders; awakening makes it blaze
            let amb = if f.form > 0 { 1.0 } else { 0.55 };
            let hgt = f.def.c("size.height", 60.0) as f32 * 1.35;
            let t = self.clock;
            // light pillar while the awakening plays
            if f.form > 0 && f.form_time < 85 {
                let k = (f.form_time as f32 / 85.0).min(1.0);
                let a = (1.0 - k).powf(0.6) * (f.form_time as f32 / 8.0).min(1.0);
                let (l, r, top, bot) = (tp(f.pos.x - 26.0 * (1.0 - k * 0.4), f.pos.y), tp(f.pos.x + 26.0 * (1.0 - k * 0.4), f.pos.y), tp(f.pos.x, f.pos.y - 600.0).y, tp(f.pos.x, f.pos.y).y);
                let cx = (l.x + r.x) / 2.0; let half = (r.x - l.x) / 2.0;
                poly::fill(&[vec2(l.x, bot), vec2(l.x, top), vec2(r.x, top), vec2(r.x, bot)], |q| { let e = 1.0 - ((q.x - cx).abs() / half.max(1.0)).min(1.0); Color::new(lerp(col[0], 1.0, e * e), lerp(col[1], 1.0, e * e), lerp(col[2], 1.0, e * e), a * 0.6 * e) });
            }
            // flame tongues licking up the body
            for layer in 0..2 {
                let n = 9;
                for i in 0..n {
                    let u = i as f32 / (n - 1) as f32;
                    let ph = t * (5.0 + i as f32 * 0.7) + i as f32 * 1.9;
                    let x0 = f.pos.x + (u - 0.5) * 34.0 * if layer == 0 { 1.0 } else { 0.55 };
                    let h = hgt * (0.55 + 0.45 * (ph).sin().abs()) * (1.0 - (u - 0.5).abs() * 0.9) * if layer == 0 { 1.0 } else { 0.62 };
                    let w = (9.5 - layer as f32 * 3.0) * s;
                    let (a, tip, b) = (tp(x0, f.pos.y), tp(x0 + (ph * 0.8).sin() * 6.0, f.pos.y - h), tp(x0, f.pos.y));
                    let (a, b) = (vec2(a.x - w, a.y), vec2(b.x + w, b.y));
                    let (topy, boty) = (tip.y, a.y);
                    let c = if layer == 0 { col } else { [1.0, 0.95, 0.85] };
                    poly::fill(&[a, tip, b], |q| { let k = ((boty - q.y) / (boty - topy).abs().max(1.0)).clamp(0.0, 1.0); Color::new(c[0], c[1], c[2], (0.95 - layer as f32 * 0.2) * amb * (1.0 - k).powf(1.1)) });
                }
            }
        }
    }

    /// Screen-space ambient weather per stage: rain, fireflies, spirit wisps, wind streaks.
    pub fn draw_weather(&self, gain: f32) {
        if self.weather == Weather::None || gain <= 0.0 { return; }
        let (w, h) = (screen_width(), screen_height());
        let u = h / 720.0;
        let t = self.clock;
        let hash = |i: f32, k: f32| ((i * 12.9898 + k * 78.233).sin() * 43758.5453).fract().abs();
        match self.weather {
            Weather::Rain => {
                set_blend(GfxBlend::Alpha);
                for i in 0..170 {
                    let f = i as f32;
                    let (hx, hy, hs) = (hash(f, 1.0), hash(f, 2.0), hash(f, 3.0));
                    let sp = (900.0 + hs * 500.0) * u;
                    let len = (14.0 + hs * 22.0) * u;
                    let y = (hy * h + t * sp) % (h + len) - len;
                    let x = (hx * (w + 200.0) - t * 140.0 * u).rem_euclid(w + 200.0) - 100.0 + (y / h) * 90.0 * u;
                    draw_line(x, y, x - len * 0.16, y + len, (1.0 * u).max(1.0), Color::new(0.7, 0.85, 1.0, 0.16 + hs * 0.14));
                }
                // ground splashes
                for i in 0..22 {
                    let f = i as f32 + 300.0;
                    let ph = (t * (1.2 + hash(f, 4.0)) + hash(f, 5.0)).fract();
                    let x = hash(f, 6.0) * w; let y = h * (0.80 + hash(f, 7.0) * 0.17);
                    draw_circle_lines(x, y, (2.0 + ph * 9.0) * u, (1.0 * u).max(1.0), Color::new(0.75, 0.9, 1.0, (1.0 - ph) * 0.35));
                }
            }
            Weather::Fireflies | Weather::Wisps => {
                let col = if self.weather == Weather::Fireflies { (0.85, 1.0, 0.4) } else { (0.45, 0.95, 0.9) };
                set_blend(GfxBlend::Alpha);
                for i in 0..26 {
                    let f = i as f32;
                    let (hx, hy, hs) = (hash(f, 8.0), hash(f, 9.0), hash(f, 10.0));
                    let x = hx * w + (t * (0.3 + hs) + f).sin() * 40.0 * u;
                    let y = if self.weather == Weather::Wisps { h - ((hy * h + t * (18.0 + hs * 26.0) * u) % h) } else { hy * h * 0.8 + h * 0.1 + (t * (0.4 + hs * 0.5) + f * 1.7).cos() * 30.0 * u };
                    let blink = 0.5 + 0.5 * (t * (1.5 + hs * 2.5) + f * 2.3).sin();
                    for k in 0..4 { draw_circle(x, y, (6.0 - k as f32 * 1.4) * u * (0.8 + hs * 0.6), Color::new(col.0, col.1, col.2, 0.05 * blink + 0.06 * k as f32 * blink)); }
                }
            }
            Weather::Wind => {
                for i in 0..24 {
                    let f = i as f32;
                    let (hy, hs) = (hash(f, 11.0), hash(f, 12.0));
                    let len = (60.0 + hs * 140.0) * u;
                    let x = ((hash(f, 13.0) * (w + 400.0)) + t * (260.0 + hs * 360.0) * u) % (w + 400.0) - 200.0;
                    let y = hy * h * 0.85 + (t * 0.8 + f).sin() * 6.0 * u;
                    draw_line(x, y, x - len, y + len * 0.05, (1.0 * u).max(1.0), Color::new(1.0, 0.95, 0.85, 0.08 + hs * 0.08));
                }
            }
            Weather::None => {}
        }
    }

    /// World-space particles and rings (call inside the zoomed transform).
    pub fn draw_world(&self, g: &Gfx, m: &Match, shx: f32, shy: f32, ui: &Ui) {
        let s = g.view_scale;
        let tp = |x: f32, y: f32| vec2(g.ox + (x - m.camera.x + m.view_w / 2.0 + shx) * s, g.oy + (y - m.camera.y + m.stage.zoffset + shy) * s);
        // normal blend
        for p in self.parts.iter().filter(|p| matches!(p.k, Kind::Dust | Kind::Debris)) {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let c = tp(p.x, p.y);
            match p.k {
                Kind::Dust => poly::circle(c, p.size * s, Color::new(p.col[0], p.col[1], p.col[2], 0.35 * a)),
                _ => {
                    // irregular 5-point shard, lit on one side
                    let r = p.size * s * 1.4; let (sn, cs) = p.rot.sin_cos();
                    let pts: Vec<Vec2> = [vec2(-r, -r * 0.4), vec2(-r * 0.1, -r * 0.9), vec2(r, -r * 0.2), vec2(r * 0.5, r * 0.8), vec2(-r * 0.6, r * 0.5)].iter().map(|q| c + vec2(q.x * cs - q.y * sn, q.x * sn + q.y * cs)).collect();
                    let base = Color::new(p.col[0], p.col[1], p.col[2], a);
                    poly::fill(&pts, |q| if q.x + q.y < c.x + c.y { poly::shade(base, 1.25) } else { poly::shade(base, 0.7) });
                    poly::outline(&pts, 0.5 * s, Color::new(0.1, 0.05, 0.08, a * 0.6));
                }
            }
        }
        // additive
        set_blend(GfxBlend::Add);
        for p in self.parts.iter().filter(|p| matches!(p.k, Kind::Spark | Kind::Glow | Kind::Ember)) {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let c = tp(p.x, p.y);
            match p.k {
                Kind::Spark => {
                    let t = tp(p.x - p.vx * 2.8, p.y - p.vy * 2.8);
                    let d = (t - c).normalize_or_zero(); let n = vec2(-d.y, d.x) * (p.size * s).max(1.0) * 0.9;
                    let head = Color::new(1.0, 1.0, 0.92, a); let tail = Color::new(p.col[0], p.col[1], p.col[2], 0.0);
                    // 4-point blade: wide bright head, long tapering tail
                    poly::fill(&[c - d * (p.size * s * 0.6), c + n, t, c - n], |q| if (q - t).length() < 0.001 { tail } else if (q - c).length() < 0.001 { head } else { Color::new(p.col[0], p.col[1], p.col[2], a) });
                }
                Kind::Ember => { let r = p.size * s * (0.6 + 0.4 * a); poly::solid(&poly::regular(c, r * 1.5, 4, p.rot + p.life * 6.0), Color::new(p.col[0], p.col[1], p.col[2], a * 0.9)); }
                _ => { for i in 0..6 { let t = i as f32 / 6.0; poly::circle(c, p.size * s * (1.0 - t * 0.85), Color::new(p.col[0], p.col[1], p.col[2], a * 0.11)); } }
            }
        }
        for p in self.parts.iter().filter(|p| p.k == Kind::Slash) {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let c = tp(p.x, p.y);
            let (sn, cs) = p.rot.sin_cos();
            let l = p.size * s * (1.3 - 0.3 * a);
            let w = (1.4 * a + 0.3) * s;
            let tip = vec2(c.x + cs * l, c.y + sn * l);
            let (nx, ny) = (-sn * w, cs * w);
            draw_triangle(vec2(c.x + nx, c.y + ny), vec2(c.x - nx, c.y - ny), tip, Color::new(p.col[0], p.col[1], p.col[2], a));
        }
        for l in &self.lights {
            let t = l.t / l.dur; let a = (1.0 - t).powi(2);
            let c = tp(l.x, l.y);
            for i in 0..9 { let k = 1.0 - i as f32 / 9.0; poly::circle(c, l.r * s * k, Color::new(l.col[0], l.col[1], l.col[2], 0.032 * a)); }
        }
        for r in &self.rings {
            let t = r.t / r.dur; let e = 1.0 - (1.0 - t).powi(3);
            let c = tp(r.x, r.y);
            poly::ring(c, lerp(r.r0, r.r1, e) * s, (r.w * (1.0 - t) + 0.4) * s, Color::new(r.col[0], r.col[1], r.col[2], (1.0 - t) * 0.9));
        }
        set_blend(GfxBlend::Alpha);
        // damage numbers / labels
        for f in &self.floats {
            let t = f.t / FLOAT_LIFE;
            let pop = if t < 0.15 { 0.6 + t / 0.15 * 0.7 } else { 1.3 - (t - 0.15).min(0.3) };
            let a = if t > 0.7 { (1.0 - t) / 0.3 } else { 1.0 };
            let c = tp(f.x, f.y);
            let v = ui.to_virtual(c);
            let sz = f.size * s / ui.s * pop;
            ui.text_bold(&f.text, v.x, v.y, sz, Color::new(f.col[0], f.col[1], f.col[2], a), Color::new(0.1, 0.02, 0.08, a), crate::ui::Align::Center);
        }
    }

    /// Screen-space speed lines (radial) around `focus` (screen px).
    pub fn draw_lines(&self, focus: (f32, f32), gain: f32) {
        if self.lines <= 0.02 { return; }
        speed_lines(focus, (self.lines.min(1.0)) * 0.55 * gain.min(1.2), self.lines_col, self.tick / 2, 46);
    }

    /// Big banner texts (COUNTER!, PERFECT!, SUPER ARTS…), slammed in with overshoot.
    pub fn draw_cinema_bars(&self, ui: &Ui) {
        if self.sz > 0.02 { let h = 26.0 * self.sz * self.sz * (3.0 - 2.0 * self.sz); ui.rect(0.0, 0.0, 1280.0, h, Color::new(0.0, 0.0, 0.0, 0.92)); ui.rect(0.0, 720.0 - h, 1280.0, h, Color::new(0.0, 0.0, 0.0, 0.92)); }
    }

    pub fn draw_banners(&self, ui: &Ui) {
        for (i, b) in self.banners.iter().enumerate() {
            let t = b.t;
            let sc = if t < 0.12 { lerp(1.25, 1.0, t / 0.12) } else { 1.0 };
            let a = if t > 1.2 { ((1.6 - t) / 0.4).max(0.0) } else { (t / 0.08).min(1.0) };
            let y = 124.0 + i as f32 * 44.0;
            let hw = 190.0 * (0.6 + 0.4 * (t / 0.15).min(1.0));
            ui.rect(640.0 - hw, y - 24.0, hw * 2.0, 34.0, Color::new(b.col[0] * 0.2, b.col[1] * 0.2, b.col[2] * 0.25, 0.35 * a));
            ui.rect(640.0 - hw, y + 10.0, hw * 2.0, 1.5, Color::new(b.col[0], b.col[1], b.col[2], 0.7 * a));
            ui.text_bold(&b.text, 640.0, y, 26.0 * sc, Color::new(b.col[0], b.col[1], b.col[2], a), Color::new(0.1, 0.02, 0.1, a), crate::ui::Align::Center);
            if !b.sub.is_empty() { ui.text(&b.sub, 640.0, y + 8.0, 11.0, Color::new(1.0, 1.0, 1.0, a * 0.8), crate::ui::Align::Center, false); }
        }
    }
}

/// Radial manga-style speed lines converging on `focus` (screen px). `seed` animates them in steps.
pub fn speed_lines(focus: (f32, f32), a: f32, col: [f32; 3], seed: u32, n: usize) {
    let (w, h) = (screen_width(), screen_height());
    let reach = (w * w + h * h).sqrt();
    let seed = seed as f32;
    set_blend(GfxBlend::Alpha);
    for i in 0..n {
        let h1 = ((i as f32 * 12.9898 + seed * 7.13).sin() * 43758.5453).fract().abs();
        let h2 = ((i as f32 * 78.233 + seed * 3.7).sin() * 12543.1).fract().abs();
        let ang = (i as f32 + h1 * 0.8) / n as f32 * std::f32::consts::TAU;
        let (r0, r1, wd) = (reach * (0.28 + h2 * 0.18), reach * (0.75 + h1 * 0.3), 0.012 + h2 * 0.018);
        let (c, s) = (ang.cos(), ang.sin());
        let (pc, ps) = ((ang + wd).cos(), (ang + wd).sin());
        draw_triangle(vec2(focus.0 + c * r0, focus.1 + s * r0), vec2(focus.0 + c * r1, focus.1 + s * r1), vec2(focus.0 + pc * r1, focus.1 + ps * r1), Color::new(col[0], col[1], col[2], a * (0.4 + h1 * 0.6)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn big_flashes_are_limited_to_three_per_second() {
        let mut v = Vfx::new();
        let mut fired = 0;
        for i in 0..60 { // one second at 60 fps, a big flash requested every frame
            let before = v.flash_level();
            v.flash = 0.0; v.flash_add(0.9); if v.flash_level() > before { fired += 1; }
            v.update(1.0 / 60.0);
        }
        assert!(fired <= 3, "{fired} large flashes in one second");
        assert!(fired >= 2, "but they are not suppressed entirely");
    }
    #[test]
    fn slow_motion_eases_back_to_normal() {
        let mut v = Vfx::new();
        assert_eq!(v.time_scale(), 1.0);
        v.slow_t = 1.5; assert!((v.time_scale() - 0.25).abs() < 1e-6, "held at 25%");
        v.slow_t = 0.25; let mid = v.time_scale(); assert!(mid > 0.25 && mid < 1.0, "easing out");
        v.slow_t = 0.0; assert_eq!(v.time_scale(), 1.0);
    }
    #[test]
    fn trauma_shake_is_quadratic_and_decays() {
        let mut v = Vfx::new();
        v.trauma = 0.2; let weak = { let (x, y) = v.shake(1.0); x.abs().max(y.abs()) };
        assert!(weak <= 0.2 * 0.2 * 5.0 + 1e-4, "light hits barely nudge");
        v.trauma = 1.0; v.update(0.1); assert!(v.trauma < 1.0);
        v.trauma = 0.0; assert_eq!(v.shake(1.0), (0.0, 0.0));
    }
}
