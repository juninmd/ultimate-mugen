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
mod ctx;
mod look;
mod primitives;
mod crowd;
mod react;
mod frame;
mod debris;
mod glints;
mod impact;
mod draw;
mod floor;
mod dojo_layer;
mod dojo_front;
mod dojo_lantern;
mod spirit_charm;
mod spirit_front;
mod illustrated;
#[cfg(test)]
mod tests;
pub use primitives::*;
use look::look;
use crowd::{banner, spectator};
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
