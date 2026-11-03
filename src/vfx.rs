//! AAA-style impact feedback: layered particles (sparks, glows, dust, debris), shockwave rings, damage numbers, banners,
//! camera punch/KO zoom, slow-motion, screen-shake "trauma", and the parameters fed to the post-processing pass.
//! Everything here is presentation only — the deterministic simulation is untouched.
use crate::render::Gfx;
use crate::ui::Ui;
use macroquad::prelude::*;
use ume_engine::matchs::{Event, Match};

#[derive(Clone, Copy, PartialEq)]
enum Kind { Spark, Glow, Dust, Debris, Ember, Slash }
struct P { k: Kind, x: f32, y: f32, vx: f32, vy: f32, g: f32, life: f32, max: f32, size: f32, col: [f32; 3], rot: f32, vrot: f32 }
struct Ring { x: f32, y: f32, r0: f32, r1: f32, t: f32, dur: f32, w: f32, col: [f32; 3] }
/// SF6-style colour paint splash: irregular blob + droplets, spawned *behind the attacker* so the source of the hit reads clearly.
struct Splat { x: f32, y: f32, dir: f32, t: f32, dur: f32, col: [f32; 3], radii: [f32; 14], drops: [(f32, f32, f32); 7], size: f32 }
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
    splats: Vec<Splat>, lights: Vec<Light>, pub weather: Weather, clock: f32, invert_t: f32, dist_zoom: f32, dist_target: f32,
    parts: Vec<P>, rings: Vec<Ring>, floats: Vec<Float>, pub banners: Vec<Banner>,
    pub trauma: f32, punch: f32, ko_t: f32, ko_focus: (f32, f32), pub slow_t: f32,
    chroma: f32, flash: f32, radial: f32, vig: f32, lines: f32, lines_col: [f32; 3], sat_pulse: f32,
    super_t: f32, super_owner: usize,
    /// sound requests produced by the effects layer (drained by the session)
    pub sfx: Vec<&'static str>,
    combo: [i32; 2], meter: [i32; 2],
    snap: [Snap; 2], pre_mt: [char; 2], rng: u32, tick: u32,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

impl Vfx {
    pub fn new() -> Self {
        Vfx { splats: vec![], lights: vec![], weather: Weather::None, clock: 0.0, invert_t: 0.0, dist_zoom: 0.0, dist_target: 0.0, parts: vec![], rings: vec![], floats: vec![], banners: vec![], trauma: 0.0, punch: 0.0, ko_t: 0.0, ko_focus: (0.0, 0.0), slow_t: 0.0, chroma: 0.0, flash: 0.0, radial: 0.0, vig: 0.0, lines: 0.0,
            lines_col: [1.0, 1.0, 1.0], sat_pulse: 0.0, super_t: 0.0, super_owner: 0, sfx: vec![], combo: [0; 2], meter: [0; 2], snap: [Snap::default(); 2], pre_mt: [' '; 2], rng: 0x9e37_79b9, tick: 0 }
    }
    fn r(&mut self) -> f32 { let mut x = self.rng; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rng = x; x as f32 / 4294967296.0 }
    fn rr(&mut self, a: f32, b: f32) -> f32 { a + (b - a) * self.r() }

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
    fn splat(&mut self, x: f32, y: f32, dir: f32, size: f32, col: [f32; 3]) {
        let mut radii = [0.0; 14];
        for (i, r) in radii.iter_mut().enumerate() { *r = if i % 2 == 0 { self.rr(0.75, 1.15) } else { self.rr(0.35, 0.65) }; }
        let mut drops = [(0.0, 0.0, 0.0); 7];
        for d in drops.iter_mut() { *d = (self.rr(-1.2, 1.2), self.rr(1.1, 1.9), self.rr(0.04, 0.12)); }
        self.splats.push(Splat { x, y, dir, t: 0.0, dur: 0.6, col, radii, drops, size });
    }
    fn light(&mut self, x: f32, y: f32, r: f32, dur: f32, col: [f32; 3]) { self.lights.push(Light { x, y, r, t: 0.0, dur, col }); }
    /// Anime "impact frame" (colour inversion for a few ms) — used on super activation and KO.
    pub fn force_impact(&mut self) { self.invert_t = 0.075; }
    fn slashes(&mut self, x: f32, y: f32, dir: f32, n: usize, len: f32, col: [f32; 3]) {
        for i in 0..n {
            let a = (if dir >= 0.0 { 0.0 } else { std::f32::consts::PI }) + self.rr(-0.9, 0.9) + if i % 2 == 0 { 0.0 } else { 3.14159 };
            let l = len * self.rr(0.6, 1.2);
            self.parts.push(P { k: Kind::Slash, x, y, vx: 0.0, vy: 0.0, g: 0.0, life: 0.13, max: 0.13, size: l, col, rot: a, vrot: 0.0 });
        }
    }
    fn ring(&mut self, x: f32, y: f32, r1: f32, dur: f32, w: f32, col: [f32; 3]) { self.rings.push(Ring { x, y, r0: 2.0, r1, t: 0.0, dur, w, col }); }
    fn glow(&mut self, x: f32, y: f32, size: f32, life: f32, col: [f32; 3]) { self.parts.push(P { k: Kind::Glow, x, y, vx: 0.0, vy: 0.0, g: 0.0, life, max: life, size, col, rot: 0.0, vrot: 0.0 }); }
    fn float(&mut self, x: f32, y: f32, text: String, size: f32, col: [f32; 3]) { self.floats.push(Float { x, y, t: 0.0, text, size, col }); }
    pub fn banner(&mut self, text: &str, sub: &str, col: [f32; 3]) {
        // never stack the same banner twice (e.g. both fighters landing a counter in the same exchange)
        if self.banners.iter().any(|b| b.text == text && b.t < 0.8) { return; }
        self.banners.push(Banner { text: text.into(), sub: sub.into(), t: 0.0, col });
    }

    /// Call right before `Match::tick`: remember who was attacking (for COUNTER detection).
    pub fn pre_tick(&mut self, m: &Match) {
        for side in 0..2 { if let Some(f) = m.roots().find(|f| f.side == side) { self.pre_mt[side] = f.move_type; } }
    }

    fn chest(m: &Match, side: usize) -> Option<(f32, f32, f32)> {
        m.roots().find(|f| f.side == side).map(|f| Self::chest_of(f))
    }
    fn chest_of(f: &ume_engine::fighter::Fighter) -> (f32, f32, f32) { (f.pos.x, f.pos.y - f.def.c("size.height", 60.0) as f32 * 0.55, f.facing) }

    /// Feed one simulation event.
    pub fn event(&mut self, ev: &Event, m: &Match) {
        match ev {
            Event::Hit { attacker_side, defender_side, damage, guarded, heavy, attacker, defender } => {
                let Some(df) = m.fighters.get(*defender) else { return };
                let (dx, dy, dface) = Self::chest_of(df);
                let dir = if m.fighters.get(*attacker).is_some_and(|a| a.pos.x <= dx) { 1.0 } else { -1.0 };
                let (x, y) = (dx + dface * 7.0, dy);
                let mag = (damage / 90.0).clamp(0.2, 1.4);
                if *guarded {
                    self.spark_burst(x, y, dir, 7, 3.2, [0.45, 0.75, 1.0], 0.9);
                    self.ring(x, y, 16.0, 0.22, 1.4, [0.5, 0.8, 1.0]);
                    self.glow(x, y, 12.0, 0.12, [0.5, 0.8, 1.0]);
                    self.trauma = (self.trauma + 0.08).min(1.0);
                    self.float(x, y - 14.0, "GUARD".into(), 6.5, [0.55, 0.8, 1.0]);
                    return;
                }
                // 5-8 micro-responses at once: burst, glow, ring, dust, shake, punch, chroma, number
                let n = 9 + (mag * 14.0) as usize;
                self.spark_burst(x, y, dir, n, 3.0 + mag * 4.5, [1.0, 0.62, 0.15], 1.0);
                self.spark_burst(x, y, dir, 4, 5.5, [1.0, 1.0, 0.9], 0.4);
                self.glow(x, y, 16.0 + mag * 14.0, 0.14, [1.0, 0.75, 0.35]);
                self.slashes(x, y, dir, 3 + (mag * 3.0) as usize, 22.0 + mag * 16.0, [1.0, 0.95, 0.8]);
                self.light(x, y - 6.0, 70.0 + mag * 70.0, 0.2, [1.0, 0.7, 0.35]);
                self.ring(x, y, 20.0 + mag * 26.0, 0.28, 1.8, [1.0, 0.85, 0.5]);
                self.trauma = (self.trauma + 0.16 + mag * 0.22).min(1.0);
                self.punch = self.punch.max(0.012 + mag * 0.018);
                self.chroma = self.chroma.max(0.35 + mag * 0.4);
                if *heavy || mag > 0.75 {
                    self.ring(x, y, 52.0, 0.4, 2.6, [1.0, 1.0, 1.0]);
                    self.debris(x, y + 10.0, 8, [0.8, 0.72, 0.6]);
                    self.lines = 1.0; self.lines_col = [1.0, 0.95, 0.8];
                    // SF6-style colour paint, built behind the attacker so the source of the hit is readable
                    let (ax, scol) = (m.fighters.get(*attacker).map(|a| a.pos.x).unwrap_or(x - dir * 30.0), if *attacker_side == 0 { [0.13, 0.78, 1.0] } else { [1.0, 0.25, 0.55] });
                    self.splat(ax - dir * 6.0, y + 6.0, dir, 34.0 + mag * 22.0, scol);
                    self.flash = self.flash.max(0.18);
                    self.punch = self.punch.max(0.04);
                }
                let counter = self.pre_mt[*defender_side] == 'A';
                let col = if counter { [1.0, 0.35, 0.3] } else { [1.0, 0.92, 0.55] };
                self.float(x + dir * 12.0, y - 10.0, format!("{}", *damage as i32), 8.0 + mag * 6.0, col);
                if counter { self.sfx.push("counter"); self.banner("COUNTER!", "", [1.0, 0.3, 0.25]); self.glow(x, y, 26.0, 0.2, [1.0, 0.3, 0.2]); self.chroma = self.chroma.max(0.9); }
            }
            Event::Super { side } => {
                self.super_t = 1.0; self.super_owner = *side;
                if let Some((x, y, _)) = Self::chest(m, *side) {
                    self.ring(x, y, 90.0, 0.7, 3.5, [1.0, 0.9, 0.4]);
                    self.ring(x, y, 60.0, 0.5, 2.0, [1.0, 1.0, 1.0]);
                    self.glow(x, y, 50.0, 0.4, [1.0, 0.8, 0.3]);
                }
                self.flash = self.flash.max(0.55); self.chroma = self.chroma.max(1.0); self.radial = 1.0; self.trauma = (self.trauma + 0.4).min(1.0);
                self.lines = 1.4; self.lines_col = [1.0, 0.85, 0.4];
                self.invert_t = 0.075;
                if let Some((x, y, f)) = Self::chest(m, *side) { self.splat(x - f * 10.0, y + 8.0, f, 80.0, if *side == 0 { [0.13, 0.78, 1.0] } else { [1.0, 0.25, 0.55] }); self.light(x, y, 190.0, 0.45, [1.0, 0.85, 0.45]); }
                self.banner("SUPER ARTS", "", [1.0, 0.82, 0.3]);
            }
            Event::Ko { loser_side } => {
                self.slow_t = 1.7; self.ko_t = 1.0;
                if let Some((x, y, _)) = Self::chest(m, *loser_side) {
                    self.ko_focus = (x, y);
                    self.ring(x, y, 130.0, 0.9, 4.0, [1.0, 0.25, 0.3]);
                    self.ring(x, y, 80.0, 0.6, 2.5, [1.0, 1.0, 1.0]);
                    self.glow(x, y, 60.0, 0.5, [1.0, 0.3, 0.3]);
                    self.light(x, y, 220.0, 0.6, [1.0, 0.3, 0.3]);
                    self.spark_burst(x, y, 1.0, 26, 8.0, [1.0, 0.4, 0.3], 3.14);
                    self.debris(x, y + 12.0, 14, [0.7, 0.6, 0.55]);
                }
                self.invert_t = 0.075; self.flash = 0.9; self.chroma = 1.0; self.radial = 1.0; self.trauma = 1.0; self.vig = 1.0; self.lines = 1.6; self.lines_col = [1.0, 0.35, 0.3];
                let w = 1 - loser_side;
                if m.roots().find(|f| f.side == w).is_some_and(|f| f.life >= f.life_max - 0.5) { self.sfx.push("perfect"); self.banner("PERFECT!", "vitória sem sofrer dano", [1.0, 0.85, 0.2]); }
            }
            Event::Fight => { self.ring(m.camera.x, m.stage.zoffset - 60.0, 160.0, 0.6, 3.0, [1.0, 0.8, 0.3]); self.flash = self.flash.max(0.35); self.chroma = self.chroma.max(0.7); self.trauma = (self.trauma + 0.25).min(1.0); self.radial = 0.6; }
            Event::RoundStart(_) => { self.slow_t = 0.0; self.ko_t = 0.0; }
            _ => {}
        }
    }

    /// Per simulation tick: landing / run dust, super aura.
    pub fn tick(&mut self, m: &Match) {
        self.tick = self.tick.wrapping_add(1);
        { let mut it = m.roots().take(2); if let (Some(a), Some(b)) = (it.next(), it.next()) { let d = (a.pos.x - b.pos.x).abs(); self.dist_target = ((260.0 - d) / 260.0).clamp(0.0, 1.0) * 0.07; } }
        for (side, f) in m.roots().enumerate().take(2) {
            let now = Snap { grounded: f.state_type != 'A', vy: f.vel.y, vx: f.vel.x, move_type: f.move_type };
            let prev = self.snap[side];
            if !prev.grounded && now.grounded { let n = 6 + (prev.vy.abs() * 1.6) as usize; self.dust(f.pos.x, f.pos.y, n.min(16), 1.6, 0.5, [0.85, 0.82, 0.78]); if prev.vy > 3.0 { self.sfx.push("land"); }
 if prev.vy > 5.0 { self.ring(f.pos.x, f.pos.y - 1.0, 26.0, 0.25, 1.2, [0.9, 0.9, 0.9]); self.trauma = (self.trauma + 0.1).min(1.0); } }
            if prev.grounded && !now.grounded && now.vy < -2.0 { self.sfx.push("jump"); self.dust(f.pos.x, f.pos.y, 5, 1.2, 0.3, [0.85, 0.82, 0.78]); }
            if now.grounded && now.vx.abs() > 3.2 && prev.vx.abs() <= 3.2 && f.state_type == 'S' { self.sfx.push("dash"); }
            if now.grounded && now.vx.abs() > 3.2 && self.tick % 4 == 0 && f.state_type == 'S' { let b = -f.facing; self.dust(f.pos.x + b * 6.0, f.pos.y, 2, 0.6, 0.25, [0.85, 0.82, 0.78]); }
            if f.move_type == 'H' && f.state_type == 'L' && !prev.grounded == false && prev.move_type != 'H' { self.dust(f.pos.x, f.pos.y, 9, 2.0, 0.4, [0.85, 0.82, 0.78]); }
            self.snap[side] = now;
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
        // super aura: rising energy motes while the super pause runs
        if let Some(p) = &m.pause { if p.is_super {
            if let Some((x, y, _)) = Self::chest(m, self.super_owner) {
                for _ in 0..3 {
                    let ang = self.rr(0.0, 6.283); let rr = self.rr(14.0, 34.0);
                    let c = if self.r() > 0.5 { [1.0, 0.85, 0.35] } else { [0.5, 0.85, 1.0] };
                    let life = self.rr(0.4, 0.8);
                    let (px, py, vy) = (x + ang.cos() * rr, y + 26.0 + self.rr(-6.0, 14.0), -self.rr(0.8, 2.2));
                    let size = self.rr(0.8, 1.8);
                    self.parts.push(P { k: Kind::Ember, x: px, y: py, vx: 0.0, vy, g: -0.01, life, max: life, size, col: c, rot: 0.0, vrot: 0.0 });
                }
                if self.tick % 14 == 0 { self.ring(x, y, 46.0, 0.45, 1.6, [1.0, 0.9, 0.5]); }
            }
            self.lines = self.lines.max(0.7);
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
        self.invert_t = (self.invert_t - dt).max(0.0);
        self.dist_zoom += (self.dist_target - self.dist_zoom) * (3.0 * dt).min(1.0);
        for sp in &mut self.splats { sp.t += dt; }
        self.splats.retain(|s| s.t < s.dur);
        for l in &mut self.lights { l.t += dt; }
        self.lights.retain(|l| l.t < l.dur);
        for r in &mut self.rings { r.t += dt; }
        self.rings.retain(|r| r.t < r.dur);
        for f in &mut self.floats { f.t += dt; f.y -= 0.35 * k * (1.0 - (f.t / 0.9).min(1.0)); }
        self.floats.retain(|f| f.t < 0.9);
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
        self.parts.truncate(900);
    }

    /// Camera: zoom factor and focus in game-view coordinates.
    pub fn camera(&self, m: &Match, gain: f32) -> (f32, (f32, f32)) {
        let mid = {
            let mut it = m.roots().take(2);
            match (it.next(), it.next()) { (Some(a), Some(b)) => ((a.pos.x + b.pos.x) / 2.0, (a.pos.y + b.pos.y) / 2.0 - 35.0), (Some(a), None) => (a.pos.x, a.pos.y - 35.0), _ => (m.camera.x, -35.0) }
        };
        let ko = if self.ko_t > 0.0 { let e = (self.ko_t.min(1.0)).powf(0.6); e * 0.28 } else { 0.0 };
        let sup = self.super_t * 0.07;
        let z = 1.0 + (self.punch + ko + sup + self.dist_zoom) * gain;
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

    /// Behind the fighters: colour paint splashes.
    pub fn draw_back(&self, g: &Gfx, m: &Match, shx: f32, shy: f32) {
        let s = g.view_scale;
        let tp = |x: f32, y: f32| vec2(g.ox + (x - m.camera.x + m.view_w / 2.0 + shx) * s, g.oy + (y - m.camera.y + m.stage.zoffset + shy) * s);
        for sp in &self.splats {
            let t = sp.t / sp.dur;
            let e = 1.0 - (1.0 - (t * 3.0).min(1.0)).powi(3);
            let a = if t < 0.55 { 1.0 } else { 1.0 - (t - 0.55) / 0.45 };
            let c = tp(sp.x + sp.dir * sp.size * 0.35 * e, sp.y);
            let n = sp.radii.len();
            let pt = |i: usize, k: f32| { let ang = i as f32 / n as f32 * 6.2832 + if sp.dir < 0.0 { 3.14159 } else { 0.0 }; let stretch = 1.0 + 0.7 * ang.cos().max(0.0); vec2(c.x + ang.cos() * sp.radii[i % n] * sp.size * e * k * stretch * s, c.y + ang.sin() * sp.radii[i % n] * sp.size * e * k * 0.8 * s) };
            for (k, col) in [(1.12, Color::new(0.06, 0.02, 0.12, 0.85 * a)), (1.0, Color::new(sp.col[0], sp.col[1], sp.col[2], 0.8 * a)), (0.55, Color::new(lerp(sp.col[0], 1.0, 0.55), lerp(sp.col[1], 1.0, 0.55), lerp(sp.col[2], 1.0, 0.55), 0.6 * a))] {
                for i in 0..n { draw_triangle(c, pt(i, k), pt((i + 1) % n, k), col); }
            }
            for (ang, dist, rs) in sp.drops.iter() {
                let a2 = ang + if sp.dir < 0.0 { 3.14159 } else { 0.0 };
                let p = vec2(c.x + a2.cos() * dist * sp.size * e * s * 1.2, c.y + a2.sin() * dist * sp.size * e * s * 0.9);
                draw_circle(p.x, p.y, rs * sp.size * s * 0.5 * e.min(1.0), Color::new(sp.col[0], sp.col[1], sp.col[2], 0.85 * a));
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
                gl_use_default_material();
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
                gl_use_default_material();
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
                Kind::Dust => draw_circle(c.x, c.y, p.size * s, Color::new(p.col[0], p.col[1], p.col[2], 0.35 * a)),
                _ => { let r = p.size * s; let (sn, cs) = p.rot.sin_cos(); let pts = [vec2(-r, -r * 0.6), vec2(r, -r * 0.6), vec2(r, r * 0.6), vec2(-r, r * 0.6)].map(|q| c + vec2(q.x * cs - q.y * sn, q.x * sn + q.y * cs)); let col = Color::new(p.col[0], p.col[1], p.col[2], a); draw_triangle(pts[0], pts[1], pts[2], col); draw_triangle(pts[0], pts[2], pts[3], col); }
            }
        }
        // additive
        gl_use_material(&g.add);
        for p in self.parts.iter().filter(|p| matches!(p.k, Kind::Spark | Kind::Glow | Kind::Ember)) {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let c = tp(p.x, p.y);
            match p.k {
                Kind::Spark => { let t = tp(p.x - p.vx * 2.6, p.y - p.vy * 2.6); draw_line(c.x, c.y, t.x, t.y, (p.size * s).max(1.0), Color::new(p.col[0], p.col[1], p.col[2], a)); }
                Kind::Ember => draw_circle(c.x, c.y, p.size * s * (0.6 + 0.4 * a), Color::new(p.col[0], p.col[1], p.col[2], a * 0.9)),
                _ => { for i in 0..6 { let t = i as f32 / 6.0; draw_circle(c.x, c.y, p.size * s * (1.0 - t * 0.85), Color::new(p.col[0], p.col[1], p.col[2], a * 0.11)); } }
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
            for i in 0..9 { let k = 1.0 - i as f32 / 9.0; draw_circle(c.x, c.y, l.r * s * k, Color::new(l.col[0], l.col[1], l.col[2], 0.032 * a)); }
        }
        for r in &self.rings {
            let t = r.t / r.dur; let e = 1.0 - (1.0 - t).powi(3);
            let c = tp(r.x, r.y);
            draw_circle_lines(c.x, c.y, lerp(r.r0, r.r1, e) * s, (r.w * (1.0 - t) + 0.4) * s, Color::new(r.col[0], r.col[1], r.col[2], (1.0 - t) * 0.9));
        }
        gl_use_default_material();
        // damage numbers / labels
        for f in &self.floats {
            let t = f.t / 0.9;
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
    pub fn draw_banners(&self, ui: &Ui) {
        for (i, b) in self.banners.iter().enumerate() {
            let t = b.t;
            let sc = if t < 0.12 { lerp(1.9, 1.0, t / 0.12) } else if t < 0.2 { lerp(1.0, 1.08, (t - 0.12) / 0.08) } else { lerp(1.08, 1.0, ((t - 0.2) / 0.3).min(1.0)) };
            let a = if t > 1.2 { ((1.6 - t) / 0.4).max(0.0) } else { (t / 0.06).min(1.0) };
            let slide = if t < 0.12 { (1.0 - t / 0.12) * 120.0 } else { 0.0 };
            let y = 300.0 + i as f32 * 70.0;
            let (x0, y0, x1, y1) = (0.0, y - 56.0, 1280.0, y + 28.0);
            ui.quad([vec2(x0, y0 + 12.0), vec2(x1, y0), vec2(x1, y1 - 12.0), vec2(x0, y1)], Color::new(b.col[0] * 0.25, b.col[1] * 0.2, b.col[2] * 0.25, 0.55 * a));
            ui.rect(0.0, y - 56.0 + 8.0, 1280.0, 2.0, Color::new(b.col[0], b.col[1], b.col[2], 0.8 * a));
            ui.text_bold(&b.text, 640.0 - slide, y, 64.0 * sc, Color::new(b.col[0], b.col[1], b.col[2], a), Color::new(0.1, 0.02, 0.1, a), crate::ui::Align::Center);
            if !b.sub.is_empty() { ui.text(&b.sub, 640.0, y + 22.0, 18.0, Color::new(1.0, 1.0, 1.0, a * 0.8), crate::ui::Align::Center, false); }
        }
    }
}

/// Radial manga-style speed lines converging on `focus` (screen px). `seed` animates them in steps.
pub fn speed_lines(focus: (f32, f32), a: f32, col: [f32; 3], seed: u32, n: usize) {
    let (w, h) = (screen_width(), screen_height());
    let reach = (w * w + h * h).sqrt();
    let seed = seed as f32;
    gl_use_default_material();
    for i in 0..n {
        let h1 = ((i as f32 * 12.9898 + seed * 7.13).sin() * 43758.5453).fract().abs();
        let h2 = ((i as f32 * 78.233 + seed * 3.7).sin() * 12543.1).fract().abs();
        let ang = (i as f32 + h1 * 0.8) / n as f32 * 6.2832;
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
