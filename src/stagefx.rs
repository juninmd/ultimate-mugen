//! Living stages: animated details + environment reaction, drawn per frame with per-vertex-colour polygons (no textures are
//! created per frame; WebGL1-safe). Presentation only — the deterministic simulation never reads any of this.
//!
//! # Driving the reaction (API for the match code)
//! `MatchRenderer` owns a [`StageReact`] (`renderer.fx`). Out of the box it **detects** hits, landings, supers, KOs and wall
//! slams by watching the `Match` every drawn frame, so no wiring is needed. To feed exact events instead (e.g. from
//! `Session::step` next to `Vfx::event`), call
//! `self.r.fx.react(&[Impulse { x: fighter_x, strength: 0.0..=1.0, kind: ImpulseKind::Hit }])`
//! — the first explicit call switches auto-detection off. `kind`: `Hit` (strength ≈ damage/100, heavy ≥ 0.6), `Land`
//! (≈ fall speed/8), `Super`, `Ko`, `Wall` (knock-back into the screen edge). Footsteps (water ripples) always come from the match.
//!
//! What reacts: props shake and lights flicker on heavy hits/supers, banners/foliage/cloth flap harder (gusts), lamps swing,
//! crowds cheer on supers and KOs, shock waves bend foliage and blow petals/sand/leaves, dust/debris/droplets/sparks are kicked
//! up, ground cracks and scorch marks are left at heavy impact points (cleared each round), water/floors ripple.
mod scenes;
use crate::gfx::*;
use crate::render::Gfx;
use crate::stages::{
    dojo_lanterns, lava_cracks, neon_signs, skyline, toro_xs, Layer, StageKind, NEON,
};
use crate::texgen::{hash, nz};
use ume_engine::matchs::Match;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ImpulseKind {
    Hit,
    Land,
    Super,
    Ko,
    Wall,
}
/// One thing that happened in the fight, in world coordinates (`x` = fighter x). `strength` 0..1 (can exceed 1 for huge ones).
#[derive(Clone, Copy, Debug)]
pub struct Impulse {
    pub x: f32,
    pub strength: f32,
    pub kind: ImpulseKind,
}

// ───────────────────────────────────────── draw primitives ─────────────────────────────────────────

pub fn hx(h: u32, a: f32) -> Color {
    Color::new(
        ((h >> 16) & 255) as f32 / 255.0,
        ((h >> 8) & 255) as f32 / 255.0,
        (h & 255) as f32 / 255.0,
        a,
    )
}
fn ca(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}
fn lerpc(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
fn qd(p: [Vec2; 4], c: [Color; 4]) {
    let vertices = (0..4)
        .map(|i| Vertex::new(p[i].x, p[i].y, 0.0, 0.0, 0.0, c[i]))
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}
fn tr(p: [Vec2; 3], c: [Color; 3]) {
    let vertices = (0..3)
        .map(|i| Vertex::new(p[i].x, p[i].y, 0.0, 0.0, 0.0, c[i]))
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2],
        texture: None,
    });
}
/// Soft radial blob (ellipse with `cin` in the centre fading to `cout` on the rim) — glows, fog, light pools.
fn fan(c: Vec2, rx: f32, ry: f32, n: usize, cin: Color, cout: Color) {
    let mut vertices = Vec::with_capacity(n + 1);
    vertices.push(Vertex::new(c.x, c.y, 0.0, 0.0, 0.0, cin));
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        vertices.push(Vertex::new(
            c.x + a.cos() * rx,
            c.y + a.sin() * ry,
            0.0,
            0.0,
            0.0,
            cout,
        ));
    }
    let mut indices = Vec::with_capacity(n * 3);
    for i in 0..n {
        indices.extend([0u16, 1 + i as u16, 1 + ((i + 1) % n) as u16]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
/// Triangle strip between two polylines `a` and `b` with a colour per vertex.
fn strip(a: &[Vec2], b: &[Vec2], cola: &[Color], colb: &[Color]) {
    let n = a.len().min(b.len());
    if n < 2 {
        return;
    }
    let mut vertices = Vec::with_capacity(n * 2);
    for i in 0..n {
        vertices.push(Vertex::new(a[i].x, a[i].y, 0.0, 0.0, 0.0, cola[i]));
        vertices.push(Vertex::new(b[i].x, b[i].y, 0.0, 0.0, 0.0, colb[i]));
    }
    let mut indices = Vec::with_capacity((n - 1) * 6);
    for i in 0..n - 1 {
        let k = (i * 2) as u16;
        indices.extend([k, k + 1, k + 2, k + 1, k + 3, k + 2]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
/// Thick segment with a colour at each end.
fn seg(a: Vec2, b: Vec2, w: f32, ca_: Color, cb: Color) {
    let d = (b - a).normalize_or_zero();
    let n = vec2(-d.y, d.x) * w * 0.5;
    qd([a + n, b + n, b - n, a - n], [ca_, cb, cb, ca_]);
}
fn ring_e(c: Vec2, rx: f32, ry: f32, th: f32, col: Color) {
    let n = ((rx.sqrt() * 3.0) as usize).clamp(14, 48);
    let mut vertices = Vec::with_capacity(n * 2);
    for k in 0..n {
        let a = k as f32 / n as f32 * std::f32::consts::TAU;
        let (cs, sn) = (a.cos(), a.sin());
        vertices.push(Vertex::new(
            c.x + cs * (rx + th),
            c.y + sn * (ry + th * 0.3),
            0.0,
            0.0,
            0.0,
            col,
        ));
        vertices.push(Vertex::new(
            c.x + cs * (rx - th).max(0.0),
            c.y + sn * (ry - th * 0.3).max(0.0),
            0.0,
            0.0,
            0.0,
            col,
        ));
    }
    let mut indices = Vec::with_capacity(n * 6);
    for k in 0..n {
        let (a, b, c2, d) = (
            (2 * k) as u16,
            (2 * k + 1) as u16,
            (2 * ((k + 1) % n)) as u16,
            (2 * ((k + 1) % n) + 1) as u16,
        );
        indices.extend([a, b, c2, b, d, c2]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
fn rot(p: Vec2, a: f32) -> Vec2 {
    let (s, c) = a.sin_cos();
    vec2(p.x * c - p.y * s, p.x * s + p.y * c)
}
fn additive(_: &Gfx, f: impl FnOnce()) {
    set_blend(GfxBlend::Add);
    f();
    set_blend(GfxBlend::Alpha);
}

/// Parallax-aware mapping from stage coordinates to screen pixels (same transform as the baked layers).
pub struct Ctx<'a> {
    pub s: f32,
    ox: f32,
    oy: f32,
    cam_x: f32,
    cam_y: f32,
    vw: f32,
    vh: f32,
    zoff: f32,
    shx: f32,
    shy: f32,
    layers: &'a [Layer],
}
impl<'a> Ctx<'a> {
    pub fn new(g: &Gfx, m: &Match, layers: &'a [Layer], shx: f32, shy: f32) -> Self {
        Ctx {
            s: g.view_scale,
            ox: g.ox,
            oy: g.oy,
            cam_x: m.camera.x,
            cam_y: m.camera.y,
            vw: m.view_w,
            vh: m.view_h,
            zoff: m.stage.zoffset,
            shx,
            shy,
            layers,
        }
    }
    /// World point (floor line = y 0, up is negative) → screen.
    pub fn w(&self, x: f32, y: f32) -> Vec2 {
        vec2(
            self.ox + (x - self.cam_x + self.vw / 2.0 + self.shx) * self.s,
            self.oy + (y - self.cam_y + self.zoff + self.shy) * self.s,
        )
    }
    /// Point in the local coordinates of back layer `i`.
    pub fn l(&self, i: usize, x: f32, y: f32) -> Vec2 {
        let l = &self.layers[i];
        let (lx, ly) = (
            self.vw / 2.0 - l.w / 2.0 - self.cam_x * l.delta + self.shx,
            l.y - self.cam_y * l.dy + (self.zoff - 205.0) + self.shy,
        );
        vec2(self.ox + (lx + x) * self.s, self.oy + (ly + y) * self.s)
    }
    /// World x that a layer-local x appears above (to look up shock waves / impacts).
    pub fn lw(&self, i: usize, x: f32) -> f32 {
        let l = &self.layers[i];
        self.vw / 2.0 - l.w / 2.0 - self.cam_x * l.delta + x - self.vw / 2.0 + self.cam_x
    }
    /// Screen-fixed view coordinates with a parallax factor (foreground sprinkles).
    pub fn v(&self, x: f32, y: f32) -> Vec2 {
        vec2(
            self.ox + (x + self.shx) * self.s,
            self.oy + (y + self.shy) * self.s,
        )
    }
    pub fn view(&self) -> (f32, f32) {
        (self.vw, self.vh)
    }
    pub fn cam(&self) -> f32 {
        self.cam_x
    }
}

// ───────────────────────────────────────── state ─────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum BitKind {
    Dust,
    Shard,
    Leaf,
    Spark,
    Drop,
    Ember,
}
struct Bit {
    k: BitKind,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    g: f32,
    life: f32,
    max: f32,
    size: f32,
    rot: f32,
    vrot: f32,
    col: Color,
    ground: f32,
}
struct Shock {
    x: f32,
    age: f32,
    power: f32,
}
#[derive(Clone, Copy, PartialEq)]
enum DecalKind {
    Crack,
    Scorch,
}
struct Decal {
    x: f32,
    y: f32,
    age: f32,
    size: f32,
    seed: u32,
    kind: DecalKind,
}
struct Ripple {
    x: f32,
    y: f32,
    age: f32,
    power: f32,
}
#[derive(Clone, Copy, Default)]
struct FSnap {
    id: u32,
    hit_shake: i32,
    y: f32,
    vy: f32,
    vx: f32,
    x: f32,
    alive: bool,
    cd: f32,
    step_acc: f32,
}

/// Per-stage tuning of the shared systems.
struct Look {
    gloss: f32,
    wobble: f32,
    tint: [f32; 3],
    dust: Color,
    shard: Color,
    leaf: Color,
    leaf2: Color,
    pool: Color,
    crack: Color,
    glow: Option<Color>,
    ripple: Color,
    wet: bool,
}
fn look(k: StageKind) -> Look {
    match k {
        StageKind::Dojo => Look {
            gloss: 0.16,
            wobble: 0.0,
            tint: [1.0, 0.9, 0.82],
            dust: hx(0xd8b898, 1.0),
            shard: hx(0x9a6a48, 1.0),
            leaf: hx(0xffb7c8, 1.0),
            leaf2: hx(0xff8fb0, 1.0),
            pool: hx(0xffa050, 1.0),
            crack: hx(0x1a0a06, 1.0),
            glow: None,
            ripple: hx(0xffd8a8, 1.0),
            wet: false,
        },
        StageKind::Neon => Look {
            gloss: 0.42,
            wobble: 0.4,
            tint: [0.9, 0.86, 1.0],
            dust: hx(0x9a88c8, 1.0),
            shard: hx(0x4a3a78, 1.0),
            leaf: hx(0xff4adc, 1.0),
            leaf2: hx(0x30e8ff, 1.0),
            pool: hx(0xd04aff, 1.0),
            crack: hx(0x06020e, 1.0),
            glow: None,
            ripple: hx(0xb8a0ff, 1.0),
            wet: true,
        },
        StageKind::Bamboo => Look {
            gloss: 0.5,
            wobble: 1.0,
            tint: [0.86, 0.96, 0.98],
            dust: hx(0xa8d8d0, 1.0),
            shard: hx(0x2a6a56, 1.0),
            leaf: hx(0x6ad07a, 1.0),
            leaf2: hx(0xa8e86a, 1.0),
            pool: hx(0x7ad8e0, 1.0),
            crack: hx(0x0a2a28, 1.0),
            glow: None,
            ripple: hx(0xd8fff8, 1.0),
            wet: true,
        },
        StageKind::Plains => Look {
            gloss: 0.06,
            wobble: 0.0,
            tint: [1.0, 0.96, 0.88],
            dust: hx(0xe0c090, 1.0),
            shard: hx(0x9a6a44, 1.0),
            leaf: hx(0xc8a860, 1.0),
            leaf2: hx(0xa08848, 1.0),
            pool: hx(0xffe8b0, 1.0),
            crack: hx(0x2a1608, 1.0),
            glow: None,
            ripple: hx(0xf0d8a8, 1.0),
            wet: false,
        },
        StageKind::Spirit => Look {
            gloss: 0.5,
            wobble: 0.35,
            tint: [0.84, 0.97, 0.96],
            dust: hx(0xa8e0d8, 1.0),
            shard: hx(0x4a7a78, 1.0),
            leaf: hx(0x9afff0, 1.0),
            leaf2: hx(0x6ad8ff, 1.0),
            pool: hx(0x4affe0, 1.0),
            crack: hx(0x061418, 1.0),
            glow: Some(hx(0x4affe8, 1.0)),
            ripple: hx(0x8affee, 1.0),
            wet: true,
        },
        StageKind::Volcano => Look {
            gloss: 0.3,
            wobble: 0.25,
            tint: [1.0, 0.8, 0.66],
            dust: hx(0x6a4a40, 1.0),
            shard: hx(0x3a2018, 1.0),
            leaf: hx(0xff8a30, 1.0),
            leaf2: hx(0xffcc50, 1.0),
            pool: hx(0xff6a20, 1.0),
            crack: hx(0x120403, 1.0),
            glow: Some(hx(0xff7a20, 1.0)),
            ripple: hx(0xff9a40, 1.0),
            wet: false,
        },
        StageKind::Custom => Look {
            gloss: 0.0,
            wobble: 0.0,
            tint: [1.0; 3],
            dust: hx(0xcccccc, 1.0),
            shard: hx(0x888888, 1.0),
            leaf: hx(0xffffff, 1.0),
            leaf2: hx(0xffffff, 1.0),
            pool: hx(0xffffff, 1.0),
            crack: hx(0, 1.0),
            glow: None,
            ripple: hx(0xffffff, 1.0),
            wet: false,
        },
    }
}

/// Reactive state of one procedural stage (see the module docs).
pub struct StageReact {
    kind: StageKind,
    look: Look,
    t: f32,
    rng: u32,
    explicit: bool,
    queue: Vec<Impulse>,
    shake: f32,
    flicker: f32,
    gust: f32,
    cheer: f32,
    swing: f32,
    swing_v: f32,
    rotor: f32,
    shocks: Vec<Shock>,
    decals: Vec<Decal>,
    ripples: Vec<Ripple>,
    bits: Vec<Bit>,
    prev: Vec<FSnap>,
    round_no: i32,
    was_super: bool,
    /// number of impulses handled so far (diagnostics / tests)
    pub applied: u32,
    /// `UME_NOFX=1` (native): bypass the animated layer entirely (A/B performance checks)
    off: bool,
}

impl StageReact {
    pub fn new(kind: StageKind) -> Self {
        StageReact {
            kind,
            look: look(kind),
            t: 0.0,
            rng: 0x1234_5679,
            explicit: false,
            queue: vec![],
            shake: 0.0,
            flicker: 0.0,
            gust: 0.0,
            cheer: 0.0,
            swing: 0.0,
            swing_v: 0.0,
            rotor: 0.0,
            shocks: vec![],
            decals: vec![],
            ripples: vec![],
            bits: vec![],
            prev: vec![],
            round_no: -1,
            was_super: false,
            applied: 0,
            #[cfg(not(target_arch = "wasm32"))]
            off: std::env::var("UME_NOFX").is_ok(),
            #[cfg(target_arch = "wasm32")]
            off: false,
        }
    }
    pub fn time(&self) -> f32 {
        self.t
    }
    pub fn active(&self) -> bool {
        self.kind != StageKind::Custom && !self.off
    }
    /// Queue explicit impulses (switches off the automatic detection from the `Match`).
    pub fn react(&mut self, imps: &[Impulse]) {
        self.explicit = true;
        self.queue.extend_from_slice(imps);
    }
    /// Reflection strength (0 = matte) and horizontal ripple wobble of the floor.
    pub fn gloss(&self) -> (f32, f32) {
        (self.look.gloss, self.look.wobble)
    }
    /// Multiplicative lighting colour for a fighter standing at world x (warm sunset, neon side lights, lava flicker...).
    pub fn ambient(&self, x: f32, cam_x: f32) -> Color {
        let t = self.look.tint;
        let side = ((x - cam_x) / 200.0).clamp(-1.0, 1.0);
        let mut c = [t[0], t[1], t[2]];
        match self.kind {
            StageKind::Neon => {
                c = [
                    t[0] + side * 0.1,
                    t[1] - side.abs() * 0.04,
                    t[2] - side * 0.1,
                ];
            }
            StageKind::Volcano => {
                let f = 0.5 + 0.5 * (self.t * 7.0).sin() * (0.5 + self.flicker);
                c[1] += f * 0.07;
                c[2] += f * 0.05;
            }
            _ => {}
        }
        Color::new(c[0].min(1.0), c[1].min(1.0), c[2].min(1.0), 1.0)
    }
    fn r(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x as f32 / 4294967296.0
    }
    fn rr(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.r()
    }

    // ───────── per-frame update ─────────
    /// Call once per drawn frame, before drawing the stage.
    pub fn begin_frame(&mut self, m: &Match, gain: f32) {
        if !self.active() {
            return;
        }
        let dt = get_frame_time().clamp(0.0, 0.05);
        self.t += dt;
        if m.round_no != self.round_no {
            self.round_no = m.round_no;
            self.decals.clear();
            self.ripples.clear();
            self.prev.clear();
        }
        let detected = self.observe(m);
        let mut imps: Vec<Impulse> = if self.explicit {
            detected
                .into_iter()
                .filter(|i| i.kind == ImpulseKind::Wall)
                .collect()
        } else {
            detected
        };
        imps.append(&mut self.queue);
        for i in imps {
            self.apply(i);
        }
        self.footsteps(m);
        let k = dt * 60.0;
        self.shake = (self.shake - dt * 1.5).max(0.0);
        self.flicker = (self.flicker - dt * 1.4).max(0.0);
        self.gust = (self.gust - dt * 0.8).max(0.0);
        self.cheer = (self.cheer - dt * 0.35).max(0.0);
        // pendulum: lamps swing back to rest (spring + damping)
        self.swing_v += (-14.0 * self.swing - 1.6 * self.swing_v) * dt;
        self.swing += self.swing_v * dt;
        self.rotor += (0.5 + self.gust * 2.0 + self.shake * 3.0) * dt;
        for s in &mut self.shocks {
            s.age += dt;
        }
        self.shocks.retain(|s| s.age < 1.6);
        for d in &mut self.decals {
            d.age += dt;
        }
        self.decals.retain(|d| d.age < 40.0);
        for r in &mut self.ripples {
            r.age += dt;
        }
        self.ripples.retain(|r| r.age < 1.5);
        let gust = self.gust;
        for b in &mut self.bits {
            b.life -= dt;
            b.x += b.vx * k;
            b.y += b.vy * k;
            b.vy += b.g * k;
            b.rot += b.vrot * k;
            match b.k {
                BitKind::Dust => {
                    b.vx *= 0.95f32.powf(k);
                    b.vy *= 0.97f32.powf(k);
                    b.size += 0.04 * k;
                }
                BitKind::Leaf => {
                    b.vx += (0.02 + gust * 0.03) * k * (self.t * 3.0 + b.rot).sin();
                    b.vy = b.vy.min(0.7);
                    b.vrot += 0.01 * (b.rot * 3.0).sin();
                }
                _ => {}
            }
            if matches!(b.k, BitKind::Shard | BitKind::Drop | BitKind::Leaf) && b.y > b.ground {
                b.y = b.ground;
                b.vy = 0.0;
                b.vx *= 0.5;
                b.vrot = 0.0;
                b.life = b.life.min(0.6);
            }
        }
        // ambient ripples: rain drops on the neon street, fish / drips in the pond, spirit pulses
        let rate = match self.kind {
            StageKind::Neon => 3.5,
            StageKind::Bamboo => 0.8,
            StageKind::Spirit => 0.5,
            _ => 0.0,
        };
        if rate > 0.0 && self.r() < dt * rate {
            let (x, y) = (m.camera.x + self.rr(-200.0, 200.0), self.rr(2.0, 26.0));
            self.ripples.push(Ripple {
                x,
                y,
                age: 0.0,
                power: 0.2,
            });
        }
        // photosensitivity / reduced-effects setting: no flicker or shake, calmer gusts
        let gn = gain.clamp(0.0, 1.3);
        self.flicker = self.flicker.min(gn);
        self.shake = self.shake.min(gn);
        if gn < 0.5 {
            self.gust = self.gust.min(1.0);
        }
        self.bits.retain(|b| b.life > 0.0);
        if self.bits.len() > 260 {
            let n = self.bits.len() - 260;
            self.bits.drain(0..n);
        }
    }

    fn observe(&mut self, m: &Match) -> Vec<Impulse> {
        let mut out = vec![];
        let roots: Vec<&ume_engine::fighter::Fighter> = m
            .fighters
            .iter()
            .filter(|f| !f.is_helper && !f.destroyed)
            .collect();
        self.prev.resize(roots.len(), FSnap::default());
        let (l, r) = (
            m.camera.x - m.view_w / 2.0 + 14.0,
            m.camera.x + m.view_w / 2.0 - 14.0,
        );
        for (i, f) in roots.iter().enumerate() {
            let p = self.prev[i];
            if p.id == f.id {
                if f.hit_shake > p.hit_shake && f.hit_shake > 0 {
                    let s = (f.ghv.damage / 100.0).clamp(0.2, 1.0)
                        * if f.ghv.guarded { 0.35 } else { 1.0 }
                        * if f.ghv.fall { 1.25 } else { 1.0 };
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: s,
                        kind: ImpulseKind::Hit,
                    });
                }
                if p.y < -1.5 && f.pos.y >= -0.05 && p.vy > 1.5 {
                    let s = (p.vy / 8.0).clamp(0.15, 1.2)
                        * if f.ghv.fall || f.state_type == 'L' {
                            1.5
                        } else {
                            1.0
                        };
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: s,
                        kind: ImpulseKind::Land,
                    });
                }
                if p.alive && !f.alive {
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: 1.0,
                        kind: ImpulseKind::Ko,
                    });
                }
                let at_edge = f.pos.x <= l
                    || f.pos.x >= r
                    || f.pos.x <= m.stage.bound_left + 4.0
                    || f.pos.x >= m.stage.bound_right - 4.0;
                if f.move_type == 'H'
                    && at_edge
                    && p.vx.abs() > 2.2
                    && f.vel.x.abs() < 0.8
                    && p.cd <= 0.0
                {
                    out.push(Impulse {
                        x: f.pos.x,
                        strength: (p.vx.abs() / 7.0).clamp(0.2, 1.0),
                        kind: ImpulseKind::Wall,
                    });
                    self.prev[i].cd = 0.5;
                }
            }
            let cd = (self.prev[i].cd - 0.016).max(0.0);
            self.prev[i] = FSnap {
                id: f.id,
                hit_shake: f.hit_shake,
                y: f.pos.y,
                vy: f.vel.y,
                vx: f.vel.x,
                x: f.pos.x,
                alive: f.alive,
                cd,
                step_acc: p.step_acc,
            };
        }
        let sup = m.pause.as_ref().filter(|p| p.is_super);
        if let Some(p) = sup {
            if !self.was_super {
                let x = m
                    .fighters
                    .iter()
                    .find(|f| f.id == p.by)
                    .map(|f| f.pos.x)
                    .unwrap_or(m.camera.x);
                out.push(Impulse {
                    x,
                    strength: 1.0,
                    kind: ImpulseKind::Super,
                });
            }
        }
        self.was_super = sup.is_some();
        out
    }

    fn footsteps(&mut self, m: &Match) {
        let roots: Vec<&ume_engine::fighter::Fighter> = m
            .fighters
            .iter()
            .filter(|f| !f.is_helper && !f.destroyed)
            .collect();
        for (i, f) in roots.iter().enumerate() {
            let Some(p) = self.prev.get(i).copied() else {
                continue;
            };
            if p.id != f.id {
                continue;
            }
            if f.pos.y >= -0.05 && f.state_type != 'L' && (f.pos.x - p.x).abs() < 20.0 {
                let acc = p.step_acc + (f.pos.x - p.x).abs();
                if acc > 13.0 && self.look.wet {
                    self.ripples.push(Ripple {
                        x: f.pos.x,
                        y: 2.0,
                        age: 0.0,
                        power: 0.35,
                    });
                    if self.r() < 0.5 {
                        self.drops(f.pos.x, 2, 0.7);
                    }
                    self.prev[i].step_acc = 0.0;
                } else {
                    self.prev[i].step_acc = if acc > 13.0 { 0.0 } else { acc };
                }
            }
        }
    }

    fn push_bit(&mut self, b: Bit) {
        self.bits.push(b);
    }
    fn dust_at(&mut self, x: f32, n: usize, spread: f32, up: f32, col: Color) {
        for _ in 0..n {
            let (vx, vy, life, size, jx) = (
                self.rr(-spread, spread),
                -self.rr(0.1, up),
                self.rr(0.45, 0.9),
                self.rr(2.0, 4.5),
                self.rr(-6.0, 6.0),
            );
            {
                let b = Bit {
                    k: BitKind::Dust,
                    x: x + jx,
                    y: self.rr(-1.0, 4.0),
                    vx,
                    vy,
                    g: -0.004,
                    life,
                    max: life,
                    size,
                    rot: 0.0,
                    vrot: 0.0,
                    col,
                    ground: 99.0,
                };
                self.push_bit(b);
            }
        }
    }
    fn shards(&mut self, x: f32, n: usize, power: f32) {
        for _ in 0..n {
            let (vx, vy, life, size, vr, gr) = (
                self.rr(-1.0, 1.0) * (1.2 + power * 2.2),
                -self.rr(1.4, 3.6) * (0.6 + power * 0.6),
                self.rr(0.8, 1.5),
                self.rr(0.8, 1.7),
                self.rr(-0.4, 0.4),
                self.rr(0.0, 14.0),
            );
            let col = lerpc(self.look.shard, WHITE, self.rr(0.0, 0.25));
            {
                let b = Bit {
                    k: BitKind::Shard,
                    x,
                    y: -4.0,
                    vx,
                    vy,
                    g: 0.2,
                    life,
                    max: life,
                    size,
                    rot: 0.0,
                    vrot: vr,
                    col,
                    ground: gr,
                };
                self.push_bit(b);
            }
        }
    }
    fn drops(&mut self, x: f32, n: usize, power: f32) {
        for _ in 0..n {
            let (vx, vy, life, gr) = (
                self.rr(-1.0, 1.0) * (0.8 + power),
                -self.rr(1.6, 3.4) * (0.6 + power * 0.5),
                self.rr(0.6, 1.1),
                self.rr(0.0, 10.0),
            );
            let c = self.look.ripple;
            {
                let b = Bit {
                    k: BitKind::Drop,
                    x,
                    y: -1.0,
                    vx,
                    vy,
                    g: 0.17,
                    life,
                    max: life,
                    size: self.rr(0.6, 1.1),
                    rot: 0.0,
                    vrot: 0.0,
                    col: ca(c, 0.9),
                    ground: gr,
                };
                self.push_bit(b);
            }
        }
    }
    fn leaves(&mut self, x: f32, n: usize, power: f32, from_top: bool) {
        for _ in 0..n {
            let dirx = if self.r() < 0.5 { -1.0 } else { 1.0 };
            let (vx, vy, life, size, vr) = (
                dirx * self.rr(0.8, 2.6) * (0.5 + power),
                if from_top {
                    self.rr(0.1, 0.5)
                } else {
                    -self.rr(0.8, 2.6) * (0.5 + power * 0.5)
                },
                self.rr(2.2, 4.2),
                self.rr(1.4, 2.4),
                self.rr(-0.25, 0.25),
            );
            let col = if self.r() < 0.5 {
                self.look.leaf
            } else {
                self.look.leaf2
            };
            let (y, gr) = (
                if from_top {
                    self.rr(-170.0, -60.0)
                } else {
                    self.rr(-6.0, -1.0)
                },
                self.rr(0.0, 22.0),
            );
            {
                let b = Bit {
                    k: BitKind::Leaf,
                    x: x + self.rr(-14.0, 14.0),
                    y,
                    vx,
                    vy,
                    g: 0.022,
                    life,
                    max: life,
                    size,
                    rot: self.rr(0.0, std::f32::consts::TAU),
                    vrot: vr,
                    col,
                    ground: gr,
                };
                self.push_bit(b);
            }
        }
    }
    fn sparks(&mut self, x: f32, y: f32, n: usize, col: Color, power: f32) {
        for _ in 0..n {
            let a = self.rr(-3.1, 0.0);
            let s = self.rr(0.8, 3.6) * (0.6 + power);
            let life = self.rr(0.3, 0.7);
            {
                let b = Bit {
                    k: BitKind::Spark,
                    x,
                    y,
                    vx: a.cos() * s,
                    vy: a.sin() * s * 0.8,
                    g: 0.14,
                    life,
                    max: life,
                    size: self.rr(0.6, 1.2),
                    rot: 0.0,
                    vrot: 0.0,
                    col,
                    ground: 99.0,
                };
                self.push_bit(b);
            }
        }
    }
    fn embers(&mut self, x: f32, n: usize, col: Color, up: f32) {
        for _ in 0..n {
            let life = self.rr(0.8, 1.8);
            {
                let b = Bit {
                    k: BitKind::Ember,
                    x: x + self.rr(-10.0, 10.0),
                    y: self.rr(-3.0, 3.0),
                    vx: self.rr(-0.5, 0.5),
                    vy: -self.rr(0.6, 1.8) * up,
                    g: -0.004,
                    life,
                    max: life,
                    size: self.rr(0.5, 1.3),
                    rot: 0.0,
                    vrot: 0.0,
                    col,
                    ground: 99.0,
                };
                self.push_bit(b);
            }
        }
    }
    fn decal(&mut self, x: f32, size: f32, kind: DecalKind) {
        if self.decals.len() >= 9 {
            self.decals.remove(0);
        }
        let seed = (self.r() * 1e6) as u32;
        let y = self.rr(3.0, 14.0);
        self.decals.push(Decal {
            x,
            y,
            age: 0.0,
            size,
            seed,
            kind,
        });
    }

    /// Turn an impulse into stage motion, debris and marks.
    fn apply(&mut self, i: Impulse) {
        self.applied += 1;
        let s = i.strength.clamp(0.0, 1.5);
        let x = i.x;
        let kind = self.kind;
        let (dust, ripple) = (self.look.dust, self.look.ripple);
        match i.kind {
            ImpulseKind::Hit => {
                self.shake = self.shake.max(s * 0.7);
                if s > 0.3 {
                    self.flicker = self.flicker.max(s);
                }
                self.gust = (self.gust + s * 0.7).min(3.0);
                let sg = if self.r() < 0.5 { -1.0 } else { 1.0 };
                self.swing_v += sg * s * 1.6;
                self.cheer = self.cheer.max(s * 0.3);
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: s * 0.6,
                });
                self.dust_at(x, 2 + (s * 8.0) as usize, 1.2, 0.35, dust);
                if s > 0.45 {
                    self.shards(x, 3 + (s * 8.0) as usize, s);
                    self.ripples.push(Ripple {
                        x,
                        y: 3.0,
                        age: 0.0,
                        power: s * 0.8,
                    });
                    self.decal(x, 16.0 + s * 26.0, DecalKind::Crack);
                }
                if s > 0.8 {
                    self.decal(x, 20.0 + s * 20.0, DecalKind::Scorch);
                }
                self.special(kind, x, s * 0.7);
            }
            ImpulseKind::Land => {
                self.shake = self.shake.max(s * 0.28);
                self.dust_at(x, 3 + (s * 10.0) as usize, 1.6, 0.4, dust);
                self.ripples.push(Ripple {
                    x,
                    y: 2.0,
                    age: 0.0,
                    power: s * 0.7,
                });
                if self.look.wet {
                    self.drops(x, 3 + (s * 6.0) as usize, s);
                }
                if s > 0.7 {
                    self.shards(x, 2 + (s * 5.0) as usize, s * 0.7);
                    self.shocks.push(Shock {
                        x,
                        age: 0.0,
                        power: s * 0.5,
                    });
                    self.gust = (self.gust + s * 0.4).min(3.0);
                }
                if s > 0.95 {
                    self.decal(x, 14.0 + s * 18.0, DecalKind::Crack);
                }
                if s > 0.45 {
                    self.special(kind, x, s * 0.45);
                }
            }
            ImpulseKind::Super => {
                self.shake = self.shake.max(0.9);
                self.flicker = 1.0;
                self.gust = (self.gust + 1.8).min(3.0);
                self.cheer = self.cheer.max(0.85);
                self.swing_v += 2.2;
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: 1.4,
                });
                self.ripples.push(Ripple {
                    x,
                    y: 3.0,
                    age: 0.0,
                    power: 1.4,
                });
                self.dust_at(x, 12, 2.4, 0.5, dust);
                self.decal(x, 38.0, DecalKind::Scorch);
                self.special(kind, x, 1.4);
            }
            ImpulseKind::Ko => {
                self.shake = 1.0;
                self.flicker = 1.0;
                self.gust = 3.0;
                self.cheer = 1.0;
                self.swing_v += 3.0;
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: 1.8,
                });
                self.ripples.push(Ripple {
                    x,
                    y: 3.0,
                    age: 0.0,
                    power: 1.6,
                });
                self.dust_at(x, 16, 2.8, 0.6, dust);
                self.shards(x, 16, 1.2);
                self.decal(x, 44.0, DecalKind::Crack);
                self.decal(x, 36.0, DecalKind::Scorch);
                self.special(kind, x, 1.8);
            }
            ImpulseKind::Wall => {
                self.shake = self.shake.max(0.6 * s + 0.2);
                self.flicker = self.flicker.max(0.5 * s);
                self.gust = (self.gust + s).min(3.0);
                self.dust_at(x, 8, 1.8, 0.4, dust);
                self.shards(x, 6 + (s * 8.0) as usize, s + 0.3);
                self.decal(x, 18.0 + s * 20.0, DecalKind::Crack);
                for q in 0..5 {
                    let life = self.rr(0.8, 1.5);
                    let (vx, sz) = (self.rr(-0.5, 0.5), self.rr(0.8, 1.8));
                    let yy = -self.rr(40.0, 150.0);
                    {
                        let b = Bit {
                            k: BitKind::Shard,
                            x: x + self.rr(-8.0, 8.0),
                            y: yy,
                            vx,
                            vy: 0.0,
                            g: 0.2,
                            life,
                            max: life,
                            size: sz,
                            rot: q as f32,
                            vrot: 0.2,
                            col: self.look.shard,
                            ground: self.rr(0.0, 14.0),
                        };
                        self.push_bit(b);
                    }
                }
                self.shocks.push(Shock {
                    x,
                    age: 0.0,
                    power: s * 0.8,
                });
                self.ripples.push(Ripple {
                    x,
                    y: 3.0,
                    age: 0.0,
                    power: s,
                });
                self.special(kind, x, s);
            }
        }
        let _ = ripple;
    }

    /// Stage-specific flavour of an impact: petals, sparks, leaves, sand, wisps, lava.
    fn special(&mut self, kind: StageKind, x: f32, s: f32) {
        let n = (4.0 + s * 12.0) as usize;
        match kind {
            StageKind::Dojo => self.leaves(x, n, s, false),
            StageKind::Neon => {
                let jx = self.rr(-60.0, 60.0);
                self.sparks(x + jx, -140.0, 3 + (s * 6.0) as usize, hx(0xffe8a0, 1.0), s);
                self.drops(x, n / 2, s);
            }
            StageKind::Bamboo => {
                self.leaves(x, n, s, true);
                self.drops(x, n, s);
            }
            StageKind::Plains => {
                self.dust_at(x, n, 3.2 + s, 0.5, self.look.dust);
                self.leaves(x, n / 2, s * 1.4, false);
            }
            StageKind::Spirit => {
                self.embers(x, n, hx(0x7affee, 1.0), 1.0);
                self.leaves(x, n / 2, s * 0.6, false);
            }
            StageKind::Volcano => {
                self.embers(x, n * 2, hx(0xff9a30, 1.0), 1.4);
                self.sparks(x, -2.0, n, hx(0xffb040, 1.0), s * 1.2);
                for _ in 0..(s * 3.0) as usize {
                    let jx = self.rr(-60.0, 60.0);
                    self.shards(x + jx, 1, s);
                }
            }
            StageKind::Custom => {}
        }
    }

    // ───────── wind / helpers ─────────
    /// Lateral push (−1..1+) at world x from passing shock waves.
    fn push_at(&self, wx: f32) -> f32 {
        let mut p = 0.0;
        for s in &self.shocks {
            let d = wx - s.x;
            let front = s.age * 260.0;
            let e = (d.abs() - front) / 38.0;
            p += d.signum() * s.power * (-e * e).exp() * (1.0 - s.age / 1.6);
        }
        p
    }
    /// Smooth wind value around 0.35 (steady breeze) with gusts; `ph` desynchronises elements.
    fn wind(&self, x: f32, ph: f32) -> f32 {
        let base = 0.35
            + 0.28 * (self.t * 0.9 + x * 0.011 + ph).sin()
            + 0.12 * (self.t * 2.1 + ph * 2.0 + x * 0.02).sin();
        base * (1.0 + self.gust * 1.3) + self.shake * 0.12 * (self.t * 31.0 + ph * 5.0).sin()
    }
    /// Light multiplier with random dropouts while `flicker` is high.
    fn flick(&self, i: u32) -> f32 {
        let n = hash(i, (self.t * 16.0) as u32);
        let calm = 0.93 + 0.07 * (self.t * 3.0 + i as f32 * 1.7).sin();
        if n < self.flicker * 0.7 {
            0.2 + n
        } else {
            calm * (1.0 + self.flicker * 0.3)
        }
    }

    // ───────── drawing entry points ─────────
    /// Animated content attached to back layer `i` (called right after the baked layer is drawn).
    pub fn draw_layer(&self, i: usize, cx: &Ctx, g: &Gfx, m: &Match) {
        match self.kind {
            StageKind::Dojo => self.dojo_layer(i, cx, g),
            StageKind::Neon => self.neon_layer(i, cx, g),
            StageKind::Bamboo => self.bamboo_layer(i, cx, g),
            StageKind::Plains => self.plains_layer(i, cx, g),
            StageKind::Spirit => self.spirit_layer(i, cx, g),
            StageKind::Volcano => self.volcano_layer(i, cx, g),
            StageKind::Custom => {}
        }
        if i == 4 {
            self.floor_common(cx, g, m);
        }
    }
    /// Foreground pass (after the fighters): front layers' animated parts, debris, fog, vignette.
    pub fn draw_front(&self, cx: &Ctx, g: &Gfx) {
        match self.kind {
            StageKind::Dojo => self.dojo_front(cx, g),
            StageKind::Neon => self.neon_front(cx, g),
            StageKind::Bamboo => self.bamboo_front(cx, g),
            StageKind::Plains => self.plains_front(cx, g),
            StageKind::Spirit => self.spirit_front(cx, g),
            StageKind::Volcano => self.volcano_front(cx, g),
            StageKind::Custom => {}
        }
        self.draw_bits(cx, g);
    }

    /// Per-stage vignette and colour grade, applied through the post-processing parameters (no extra geometry): darker, tinted
    /// edges per stage, pulsing with the flicker of lights.
    pub fn grade(&self, p: &mut crate::vfx::FxParams, gain: f32) {
        if !self.active() {
            return;
        }
        let (v, tint) = match self.kind {
            StageKind::Dojo => (0.14, [1.03, 0.99, 0.95]),
            StageKind::Neon => (0.16, [0.99, 0.97, 1.04]),
            StageKind::Bamboo => (0.16, [0.97, 1.0, 1.02]),
            StageKind::Plains => (0.05, [1.03, 1.01, 0.97]),
            StageKind::Spirit => (0.15, [0.97, 1.02, 1.02]),
            StageKind::Volcano => (0.2, [1.05, 0.98, 0.94]),
            StageKind::Custom => (0.0, [1.0; 3]),
        };
        p.vignette += v + self.flicker * 0.07 * gain.min(1.0);
        for k in 0..3 {
            p.tint[k] *= tint[k];
        }
    }

    /// Everything on the floor plane: decals, ripples, light pools, ambient gloss streak.
    fn floor_common(&self, cx: &Ctx, g: &Gfx, m: &Match) {
        let s = cx.s;
        // glossy sheen band along the floor top (wet / polished surfaces)
        if self.look.gloss > 0.1 {
            let a = self.look.gloss * 0.22;
            qd(
                [
                    cx.w(cx.cam() - 260.0, 0.0),
                    cx.w(cx.cam() + 260.0, 0.0),
                    cx.w(cx.cam() + 260.0, 22.0),
                    cx.w(cx.cam() - 260.0, 22.0),
                ],
                [
                    ca(self.look.ripple, a),
                    ca(self.look.ripple, a),
                    ca(self.look.ripple, 0.0),
                    ca(self.look.ripple, 0.0),
                ],
            );
        }
        // decals
        for d in &self.decals {
            let fade = if d.age > 30.0 {
                1.0 - (d.age - 30.0) / 10.0
            } else {
                1.0
            };
            let pop = (d.age * 8.0).min(1.0);
            let c0 = cx.w(d.x, d.y);
            match d.kind {
                DecalKind::Scorch => {
                    let r = d.size * pop;
                    fan(
                        c0,
                        r * s,
                        r * 0.28 * s,
                        18,
                        ca(self.look.crack, 0.55 * fade),
                        ca(self.look.crack, 0.0),
                    );
                }
                DecalKind::Crack => {
                    for arm in 0..7u32 {
                        let a =
                            (arm as f32 + hash(d.seed, arm) * 0.8) / 7.0 * std::f32::consts::TAU;
                        let mut p = (0.0f32, 0.0f32);
                        let len = d.size * (0.45 + hash(d.seed, arm + 20) * 0.55) * pop;
                        for k in 0..3 {
                            let a2 = a + (hash(d.seed, arm * 7 + k + 40) - 0.5) * 0.8;
                            let q = (p.0 + a2.cos() * len / 3.0, p.1 + a2.sin() * len / 3.0 * 0.3);
                            let w = (1.3 - k as f32 * 0.35) * s;
                            seg(
                                cx.w(d.x + p.0, d.y + p.1),
                                cx.w(d.x + q.0, d.y + q.1),
                                w,
                                ca(self.look.crack, 0.85 * fade),
                                ca(self.look.crack, 0.85 * fade),
                            );
                            if k == 1 {
                                let b2 = a2 + 0.9 * if arm % 2 == 0 { 1.0 } else { -1.0 };
                                seg(
                                    cx.w(d.x + q.0, d.y + q.1),
                                    cx.w(
                                        d.x + q.0 + b2.cos() * len * 0.2,
                                        d.y + q.1 + b2.sin() * len * 0.06,
                                    ),
                                    0.8 * s,
                                    ca(self.look.crack, 0.7 * fade),
                                    ca(self.look.crack, 0.0),
                                );
                            }
                            p = q;
                        }
                    }
                    if let Some(gc) = self.look.glow {
                        let hot = (1.0 - d.age / 12.0).max(0.25) * fade;
                        additive(g, || {
                            for arm in 0..7u32 {
                                let a = (arm as f32 + hash(d.seed, arm) * 0.8) / 7.0
                                    * std::f32::consts::TAU;
                                let len = d.size * (0.45 + hash(d.seed, arm + 20) * 0.55) * pop;
                                seg(
                                    c0,
                                    cx.w(d.x + a.cos() * len * 0.9, d.y + a.sin() * len * 0.27),
                                    0.9 * s,
                                    ca(gc, 0.6 * hot),
                                    ca(gc, 0.0),
                                );
                            }
                        });
                    }
                }
            }
        }
        // ripples
        for r in &self.ripples {
            let t = r.age / 1.5;
            let e = 1.0 - (1.0 - t).powi(2);
            let rx = (4.0 + e * (26.0 + 36.0 * r.power)) * s;
            let a = (1.0 - t) * (0.25 + 0.4 * r.power.min(1.2));
            let c = cx.w(r.x, r.y + 3.0 + e * 4.0);
            additive(g, || {
                ring_e(
                    c,
                    rx,
                    rx * 0.22,
                    (0.9 + r.power) * s * (1.0 - t * 0.6),
                    ca(self.look.ripple, a),
                );
                if r.power > 0.5 && t < 0.7 {
                    ring_e(
                        c,
                        rx * 0.62,
                        rx * 0.14,
                        0.7 * s,
                        ca(self.look.ripple, a * 0.6),
                    );
                }
            });
        }
        // light pools under the fighters
        let pool = self.look.pool;
        additive(g, || {
            for (i, f) in m
                .fighters
                .iter()
                .filter(|f| !f.is_helper && !f.destroyed && f.alive)
                .enumerate()
            {
                let fl = self.flick(300 + i as u32);
                let c = cx.w(f.pos.x, 6.0);
                let pc = if self.kind == StageKind::Neon {
                    if i % 2 == 0 {
                        hx(0xff40d8, 1.0)
                    } else {
                        hx(0x30e8ff, 1.0)
                    }
                } else {
                    pool
                };
                fan(c, 48.0 * s, 9.0 * s, 20, ca(pc, 0.17 * fl), ca(pc, 0.0));
            }
        });
    }

    fn draw_bits(&self, cx: &Ctx, g: &Gfx) {
        let s = cx.s;
        for b in self.bits.iter().filter(|b| {
            matches!(
                b.k,
                BitKind::Dust | BitKind::Shard | BitKind::Leaf | BitKind::Drop
            )
        }) {
            let a = (b.life / b.max).clamp(0.0, 1.0);
            let c = cx.w(b.x, b.y);
            match b.k {
                BitKind::Dust => fan(
                    c,
                    b.size * s,
                    b.size * s * 0.85,
                    12,
                    ca(b.col, 0.32 * a),
                    ca(b.col, 0.0),
                ),
                BitKind::Shard => {
                    let r = b.size * s * 1.5;
                    let (sn, cs) = b.rot.sin_cos();
                    let pts: Vec<Vec2> = [
                        vec2(-r, -r * 0.4),
                        vec2(-r * 0.1, -r * 0.9),
                        vec2(r, -r * 0.2),
                        vec2(r * 0.5, r * 0.8),
                        vec2(-r * 0.6, r * 0.5),
                    ]
                    .iter()
                    .map(|q| c + vec2(q.x * cs - q.y * sn, q.x * sn + q.y * cs))
                    .collect();
                    let base = ca(b.col, a.min(1.0));
                    crate::poly::fill(&pts, |q| {
                        if q.x + q.y < c.x + c.y {
                            crate::poly::shade(base, 1.3)
                        } else {
                            crate::poly::shade(base, 0.7)
                        }
                    });
                }
                BitKind::Leaf => {
                    let l = b.size * s * 1.7;
                    let flap = (self.t * 9.0 + b.rot * 4.0).sin().abs() * 0.7 + 0.3;
                    let (d1, d2) = (
                        rot(vec2(l, 0.0), b.rot),
                        rot(vec2(0.0, l * 0.55 * flap), b.rot),
                    );
                    let c1 = ca(b.col, a.min(1.0) * 0.95);
                    let c2 = ca(crate::poly::shade(b.col, 0.78), a.min(1.0) * 0.95);
                    tr([c - d1, c + d2, c + d1], [c1, c1, c2]);
                    tr([c - d1, c - d2, c + d1], [c2, c2, c1]);
                }
                BitKind::Drop => {
                    crate::poly::circle(c, b.size * s, ca(b.col, 0.85 * a));
                }
                _ => {}
            }
        }
        additive(g, || {
            for b in self
                .bits
                .iter()
                .filter(|b| matches!(b.k, BitKind::Spark | BitKind::Ember))
            {
                let a = (b.life / b.max).clamp(0.0, 1.0);
                let c = cx.w(b.x, b.y);
                match b.k {
                    BitKind::Spark => {
                        let t = cx.w(b.x - b.vx * 2.6, b.y - b.vy * 2.6);
                        seg(c, t, b.size * s, ca(hx(0xffffff, 1.0), a), ca(b.col, 0.0));
                    }
                    _ => {
                        let r = b.size * s * (0.7 + 0.5 * a);
                        fan(c, r * 2.6, r * 2.6, 8, ca(b.col, 0.55 * a), ca(b.col, 0.0));
                        crate::poly::circle(c, r * 0.8, ca(hx(0xfff0c0, 1.0), a));
                    }
                }
            }
        });
    }
}

// ───────────────────────────────────────── shared figures ─────────────────────────────────────────

/// A cheering spectator standing at `feet`, `u` = pixels per unit. `jump` lifts the body, `arms` (0..1) raises the arms.
#[allow(clippy::too_many_arguments)]
fn spectator(
    feet: Vec2,
    u: f32,
    h: f32,
    jump: f32,
    arms: f32,
    body: Color,
    head: Color,
    accent: Color,
    stick: Option<Color>,
    t: f32,
    ph: f32,
) {
    let b = feet - vec2(0.0, jump * u);
    let w = h * 0.34 * u;
    let hh = h * u;
    let sway = (t * 2.0 + ph).sin() * 0.6 * u;
    qd(
        [
            b + vec2(-w * 0.5 + sway, -hh * 0.78),
            b + vec2(w * 0.5 + sway, -hh * 0.78),
            b + vec2(w * 0.62, 0.0),
            b + vec2(-w * 0.62, 0.0),
        ],
        [accent, accent, body, body],
    );
    let hc = b + vec2(sway, -hh * 0.9);
    crate::poly::circle(hc, h * 0.16 * u, head);
    crate::poly::fill(
        &crate::poly::regular(hc + vec2(0.0, -h * 0.03 * u), h * 0.17 * u, 8, 0.0)
            .into_iter()
            .filter(|p| p.y <= hc.y - h * 0.03 * u + 0.1)
            .collect::<Vec<_>>(),
        |_| crate::poly::shade(body, 0.45),
    );
    // arms: down at rest, thrown up when cheering
    let al = hc + vec2(-w * 0.55, hh * 0.26);
    let ar = hc + vec2(w * 0.55, hh * 0.26);
    let wave = (t * 11.0 + ph * 3.0).sin() * 0.25 * arms;
    let tl = al
        + vec2(
            -w * (0.3 + 0.2 * arms),
            -hh * (0.1 + 0.62 * arms) - wave * hh * 0.2,
        );
    let tr_ = ar
        + vec2(
            w * (0.3 + 0.2 * arms),
            -hh * (0.1 + 0.62 * arms) + wave * hh * 0.2,
        );
    seg(al, tl, 0.18 * h * u * 0.6, body, body);
    seg(ar, tr_, 0.18 * h * u * 0.6, body, body);
    if let Some(sc) = stick {
        if arms > 0.15 {
            seg(
                tr_,
                tr_ + vec2(wave * 4.0 * u, -h * 0.34 * u),
                0.2 * u * 1.4,
                ca(sc, 0.9 * arms),
                ca(sc, 0.0),
            );
            seg(
                tl,
                tl + vec2(-wave * 4.0 * u, -h * 0.34 * u),
                0.2 * u * 1.4,
                ca(sc, 0.9 * arms),
                ca(sc, 0.0),
            );
        }
    }
}

/// Hanging cloth strip (flag / banner / paper charm). `top` anchor, `len` hangs downward along +y, `width` across.
#[allow(clippy::too_many_arguments)]
fn banner(
    top: Vec2,
    u: f32,
    len: f32,
    width: f32,
    segs: usize,
    amp: f32,
    t: f32,
    ph: f32,
    c_top: Color,
    c_bot: Color,
    mark: Option<Color>,
) {
    let (mut a, mut b, mut ca_, mut cb) = (vec![], vec![], vec![], vec![]);
    for k in 0..=segs {
        let f = k as f32 / segs as f32;
        let off = (t * 5.0 - f * 3.2 + ph).sin() * amp * f * u + amp * f * f * u * 0.5;
        let slope = ((t * 5.0 - f * 3.2 + ph).cos() * 0.5 + 0.5) * 0.35;
        let y = f * len * u * (1.0 - 0.06 * (off / u).abs() / amp.max(0.1));
        a.push(top + vec2(off - width * 0.5 * u, y));
        b.push(top + vec2(off + width * 0.5 * u, y));
        let c = lerpc(c_top, c_bot, f);
        let sh = 1.0 - slope * 0.6 + slope * 0.2;
        ca_.push(crate::poly::shade(c, sh * 1.1));
        cb.push(crate::poly::shade(c, sh * 0.8));
    }
    strip(&a, &b, &ca_, &cb);
    if let Some(mc) = mark {
        let (mut a2, mut b2, mut c2) = (vec![], vec![], vec![]);
        for k in 2..=segs.saturating_sub(1).max(3) {
            let i = k.min(segs);
            a2.push((a[i] * 0.72) + (b[i] * 0.28));
            b2.push((a[i] * 0.28) + (b[i] * 0.72));
            c2.push(mc);
        }
        strip(&a2, &b2, &c2, &c2);
    }
}

// ───────────────────────────────────────── DOJO ─────────────────────────────────────────

impl StageReact {
    fn dojo_layer(&self, i: usize, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        match i {
            0 => additive(g, || {
                // god rays fanning out of the setting sun
                let c = cx.l(0, 260.0, 150.0);
                for k in 0..7 {
                    let a = -std::f32::consts::PI
                        + (k as f32 + 0.5) / 7.0 * std::f32::consts::PI
                        + (t * 0.05 + k as f32).sin() * 0.04;
                    let wd = 0.07 + 0.03 * (t * 0.4 + k as f32 * 2.0).sin();
                    let (p1, p2) = (
                        c + vec2((a - wd).cos(), (a - wd).sin()) * 260.0 * s,
                        c + vec2((a + wd).cos(), (a + wd).sin()) * 260.0 * s,
                    );
                    tr(
                        [c, p1, p2],
                        [
                            hx(0xffe8b0, 0.085 + 0.025 * (t + k as f32).sin()),
                            hx(0xffb070, 0.0),
                            hx(0xffb070, 0.0),
                        ],
                    );
                }
            }),
            1 => {
                // flock of birds crossing the sky, wings flapping
                for k in 0..7u32 {
                    let x = (t * 16.0 + k as f32 * 21.0) % 820.0 - 60.0;
                    let y = 70.0 + (k as f32 * 1.9).sin() * 16.0 + (t * 0.8 + k as f32).sin() * 3.0;
                    let flap = (t * 8.0 + k as f32 * 1.3).sin() * (0.45 + self.gust * 0.2);
                    let c = cx.l(1, x, y);
                    let col = hx(0x2a1030, 0.9);
                    let (l, r) = (
                        c + vec2(-5.5, -2.5 * flap - 0.5) * s,
                        c + vec2(5.5, -2.5 * flap - 0.5) * s,
                    );
                    seg(l, c, 1.1 * s, col, col);
                    seg(c, r, 1.1 * s, col, col);
                }
            }
            2 => {
                // chimney smoke from the village + lit windows twinkling
                for ch in 0..2 {
                    let bx = 180.0 + ch as f32 * 470.0;
                    for k in 0..6 {
                        let p = (t * 0.22 + k as f32 / 6.0 + ch as f32 * 0.3) % 1.0;
                        let c = cx.l(
                            2,
                            bx + p * 30.0 * (0.4 + self.wind(bx, 1.0))
                                + (p * 9.0 + k as f32).sin() * 3.0,
                            178.0 - p * 60.0,
                        );
                        fan(
                            c,
                            (3.0 + p * 11.0) * s,
                            (3.0 + p * 11.0) * s,
                            10,
                            hx(0x6a3a5a, 0.30 * (1.0 - p)),
                            hx(0x6a3a5a, 0.0),
                        );
                    }
                }
                additive(g, || {
                    for k in 0..22u32 {
                        let (x, y) = (30.0 + hash(k, 5) * 840.0, 170.0 + hash(k, 6) * 36.0);
                        let b = 0.3 + 0.7 * ((t * (0.5 + hash(k, 7)) + k as f32).sin() * 0.5 + 0.5);
                        fan(
                            cx.l(2, x, y),
                            4.0 * s,
                            4.0 * s,
                            8,
                            hx(0xffc060, 0.5 * b),
                            hx(0xffc060, 0.0),
                        );
                    }
                });
            }
            3 => {
                // students watching behind the railing; they jump and cheer on supers / KOs
                for k in 0..20u32 {
                    let x = 40.0 + k as f32 * 48.0 + hash(k, 1) * 14.0;
                    let ph = hash(k, 2) * std::f32::consts::TAU;
                    let jump = self.cheer * (0.5 + 0.5 * (t * 9.0 + ph).sin().abs()) * 8.0
                        + (t * 1.7 + ph).sin().max(0.0) * 0.4;
                    let base = [
                        hx(0xe8e0d8, 1.0),
                        hx(0xd0c8c0, 1.0),
                        hx(0xf0e8e0, 1.0),
                        hx(0x40306a, 1.0),
                    ][(hash(k, 3) * 4.0) as usize % 4];
                    let dark = crate::poly::shade(base, 0.55);
                    let head = hx(0xe8b898, 1.0);
                    let tint = hx(0xff8a60, 1.0);
                    let (base, dark) =
                        (lerpc(base, tint, 0.35), lerpc(dark, hx(0x6a2a4a, 1.0), 0.4));
                    let c = cx.l(3, x, 207.0);
                    spectator(
                        c,
                        s,
                        31.0,
                        jump,
                        (self.cheer * 1.4).min(1.0),
                        dark,
                        head,
                        base,
                        None,
                        t,
                        ph,
                    );
                    if hash(k, 4) < 0.6 {
                        let hb = c - vec2(0.0, (31.0 * 0.9 + jump) * s);
                        seg(
                            hb + vec2(-4.5 * s, 0.0),
                            hb + vec2(4.5 * s, 0.0),
                            1.2 * s,
                            hx(0xd02a2a, 1.0),
                            hx(0xd02a2a, 1.0),
                        );
                    }
                }
                // falling blossoms on the sakura trees
                for k in 0..24u32 {
                    let sp = 5.0 + hash(k, 8) * 7.0;
                    let y = (t * sp + hash(k, 9) * 240.0) % 250.0 - 10.0;
                    let x = (hash(k, 10) * 1000.0
                        + (t * 0.8 + k as f32).sin() * 14.0
                        + t * (2.0 + self.wind(0.0, k as f32) * 8.0))
                        % 1000.0;
                    let c = cx.l(3, x, y);
                    let a = t * 2.0 + k as f32;
                    let d = rot(vec2(2.0 * s, 0.0), a);
                    tr(
                        [c - d, c + vec2(-d.y, d.x) * 0.5, c + d],
                        [hx(0xffc0d0, 0.9), hx(0xff90b0, 0.9), hx(0xffc0d0, 0.9)],
                    );
                }
            }
            4 => {
                // lantern light pools on the deck + halos
                additive(g, || {
                    for (n, x) in dojo_lanterns().into_iter().enumerate() {
                        if !(cx.l(4, x, 12.0).x > -80.0
                            && cx.l(4, x, 12.0).x < screen_width() + 80.0)
                        {
                            continue;
                        }
                        let fl = self.flick(n as u32);
                        fan(
                            cx.l(4, x, 12.0),
                            22.0 * s,
                            24.0 * s,
                            14,
                            hx(0xffa050, 0.30 * fl),
                            hx(0xff7030, 0.0),
                        );
                        fan(
                            cx.l(4, x, 46.0),
                            46.0 * s,
                            10.0 * s,
                            16,
                            hx(0xffa050, 0.13 * fl),
                            hx(0xff7030, 0.0),
                        );
                    }
                    // sunset shafts across the planks
                    for k in 0..4 {
                        let x0 = cx.cam() * 0.0
                            + 120.0
                            + k as f32 * 290.0
                            + (t * 0.3 + k as f32).sin() * 12.0;
                        qd(
                            [
                                cx.l(4, x0, 30.0),
                                cx.l(4, x0 + 38.0, 30.0),
                                cx.l(4, x0 + 150.0, 118.0),
                                cx.l(4, x0 + 90.0, 118.0),
                            ],
                            [
                                hx(0xffc880, 0.10),
                                hx(0xffc880, 0.10),
                                hx(0xffb070, 0.0),
                                hx(0xffb070, 0.0),
                            ],
                        );
                    }
                });
            }
            _ => {}
        }
    }

    fn dojo_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        // big paper lanterns swinging under the eave (layer 5)
        for k in 0..3u32 {
            let x = 70.0 + k as f32 * 210.0;
            let rope = 22.0 + hash(k, 1) * 8.0;
            let ang = self.swing * 0.55
                + (t * 1.3 + k as f32 * 1.9).sin() * (0.03 + self.gust * 0.015)
                + self.shake * (t * 33.0 + k as f32).sin() * 0.05;
            let top = cx.l(5, x, 30.0);
            let c = top + rot(vec2(0.0, rope * s), -ang);
            let fl = self.flick(100 + k);
            seg(top, c, 0.6 * s, hx(0x1a0c0a, 1.0), hx(0x1a0c0a, 1.0));
            let (rx, ry) = (11.0 * s, 15.0 * s);
            fan(
                c + vec2(0.0, ry),
                rx,
                ry,
                16,
                lerpc(
                    hx(0xffd890, 1.0),
                    hx(0xff6a30, 1.0),
                    0.45 * (1.0 - fl * 0.5),
                ),
                hx(0xb81c18, 1.0),
            );
            for r in 1..5 {
                let f = r as f32 / 5.0 - 0.1;
                let yy = (f - 0.5) * 2.0;
                let w = (1.0 - yy * yy).sqrt() * rx;
                seg(
                    c + vec2(-w, ry + yy * ry * 0.98),
                    c + vec2(w, ry + yy * ry * 0.98),
                    0.5 * s,
                    hx(0x701410, 0.6),
                    hx(0x701410, 0.6),
                );
            }
            qd(
                [
                    c + vec2(-rx * 0.5, 0.0),
                    c + vec2(rx * 0.5, 0.0),
                    c + vec2(rx * 0.5, 1.6 * s),
                    c + vec2(-rx * 0.5, 1.6 * s),
                ],
                [hx(0x140808, 1.0); 4],
            );
            seg(
                c + vec2(0.0, 2.0 * ry),
                c + vec2(0.0, 2.0 * ry + 6.0 * s),
                0.7 * s,
                hx(0xe8c070, 1.0),
                hx(0xa07030, 1.0),
            );
            additive(g, || {
                fan(
                    c + vec2(0.0, ry),
                    30.0 * s,
                    30.0 * s,
                    14,
                    hx(0xff9040, 0.22 * fl),
                    hx(0xff7030, 0.0),
                )
            });
        }
        // nobori banners on tall poles at the screen edges (layer 6): flap harder in gusts / hits
        for (k, x) in [38.0f32, 520.0].iter().enumerate() {
            let top = cx.l(6, *x, 60.0);
            let pole_c = hx(0x2a1410, 1.0);
            seg(
                cx.l(6, *x, 250.0),
                cx.l(6, *x, 56.0),
                2.4 * s,
                pole_c,
                pole_c,
            );
            seg(
                top + vec2(0.0, 2.0 * s),
                top + vec2(18.0 * s, 2.0 * s),
                1.4 * s,
                pole_c,
                pole_c,
            );
            let amp = 2.0 + self.wind(*x, k as f32) * 5.0 + self.shake * 4.0;
            banner(
                top + vec2(0.0, 3.0 * s),
                s,
                88.0,
                17.0,
                10,
                amp,
                t,
                k as f32 * 1.7,
                hx(0xc42a2a, 1.0),
                hx(0x8a1818, 1.0),
                Some(hx(0xf4ece0, 0.95)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heavy_events_shake_props_and_leave_marks() {
        let mut fx = StageReact::new(StageKind::Plains);
        fx.apply(Impulse {
            x: 10.0,
            strength: 0.9,
            kind: ImpulseKind::Hit,
        });
        assert!(
            fx.shake > 0.5 && fx.flicker > 0.5,
            "heavy hit shakes props and flickers lights"
        );
        assert!(
            !fx.decals.is_empty() && !fx.bits.is_empty() && !fx.shocks.is_empty(),
            "cracks, debris and a shock wave"
        );
        let mut ko = StageReact::new(StageKind::Dojo);
        ko.apply(Impulse {
            x: 0.0,
            strength: 1.0,
            kind: ImpulseKind::Ko,
        });
        assert_eq!(ko.cheer, 1.0, "the crowd goes wild on a KO");
        assert!(ko.push_at(500.0).abs() < 1e-3, "shock waves are local");
    }
    #[test]
    fn light_taps_do_not_leave_cracks() {
        let mut fx = StageReact::new(StageKind::Volcano);
        fx.apply(Impulse {
            x: 0.0,
            strength: 0.2,
            kind: ImpulseKind::Hit,
        });
        assert!(fx.decals.is_empty());
        assert!(fx.shake < 0.2);
    }
    #[test]
    fn mugen_stages_are_untouched() {
        assert!(!StageReact::new(StageKind::Custom).active());
    }
    #[test]
    fn explicit_impulses_are_queued() {
        let mut fx = StageReact::new(StageKind::Neon);
        fx.react(&[Impulse {
            x: 0.0,
            strength: 1.0,
            kind: ImpulseKind::Super,
        }]);
        assert!(fx.explicit && fx.queue.len() == 1);
    }
}
