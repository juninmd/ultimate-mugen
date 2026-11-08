//! Living backgrounds for illustrated stages: data-driven ambient layers (mist banks, drifting motes, light pools, god rays)
//! drawn over the flat illustration with per-vertex-colour polygons. Presentation only; time comes from `Match::game_time`.
mod draw;
mod profiles;

pub use draw::{draw_profile, ImageRect};
pub use profiles::profile_for;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoteKind {
    Ember,
    Petal,
    Dust,
    Firefly,
    Snow,
    Leaf,
    Spark,
    Bird,
}

/// Slow translucent fog bank. `v` is the vertical centre as a fraction of the view height, `speed` is in view units per second
/// (negative drifts left), `depth` 0..1 is the camera parallax (0 = glued to the screen, 1 = glued to the world).
#[derive(Clone, Copy, Debug)]
pub struct Mist {
    pub v: f32,
    pub height: f32,
    pub speed: f32,
    pub depth: f32,
    pub color: u32,
    pub alpha: f32,
    pub blobs: u8,
    pub front: bool,
}

/// A cloud of small moving particles inside the vertical band `band` (fractions of the view height).
/// `rise` > 0 floats up, < 0 falls (units/s); `drift` is the horizontal wind (units/s); `sway` the sine wobble amplitude (units).
#[derive(Clone, Copy, Debug)]
pub struct Motes {
    pub kind: MoteKind,
    pub count: u16,
    pub depth: f32,
    pub rise: f32,
    pub drift: f32,
    pub size: f32,
    pub band: [f32; 2],
    pub color: u32,
    pub alpha: f32,
    pub sway: f32,
    pub front: bool,
}

/// Additive light pool anchored to the illustration (`u`, `v` = fractions of the image, `radius` = fraction of the image width).
/// `flicker` 0..1 is how much the intensity wavers, `period` is the pulse length in seconds.
#[derive(Clone, Copy, Debug)]
pub struct Glow {
    pub u: f32,
    pub v: f32,
    pub radius: f32,
    pub color: u32,
    pub alpha: f32,
    pub flicker: f32,
    pub period: f32,
}

/// Slanted light shaft falling from the top edge at image fraction `u`; `slant` shifts the foot (fraction of image width).
#[derive(Clone, Copy, Debug)]
pub struct Beam {
    pub u: f32,
    pub width: f32,
    pub slant: f32,
    pub color: u32,
    pub alpha: f32,
    pub period: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub mist: &'static [Mist],
    pub motes: &'static [Motes],
    pub glows: &'static [Glow],
    pub beams: &'static [Beam],
}

impl Profile {
    pub const EMPTY: Profile = Profile { mist: &[], motes: &[], glows: &[], beams: &[] };

    /// Upper bound of polygons drawn per frame at full density; the budget test keeps every stage under `PRIMITIVE_BUDGET`.
    pub fn primitives(&self) -> usize {
        self.mist.iter().map(|m| m.blobs as usize).sum::<usize>()
            + self.motes.iter().map(|m| m.count as usize * if m.kind == MoteKind::Bird { 2 } else { 1 }).sum::<usize>()
            + self.glows.len()
            + self.beams.len()
    }
}

pub const PRIMITIVE_BUDGET: usize = 420;

/// `UME_AMBIENCE=0` turns every living layer off (before/after comparisons, perf probes).
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("UME_AMBIENCE").map_or(true, |v| v != "0"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stage_profile_fits_the_primitive_budget() {
        for id in profiles::ALL_STAGE_IDS {
            let profile = profile_for(id);
            assert!(profile.primitives() <= PRIMITIVE_BUDGET, "{id}: {} primitives", profile.primitives());
        }
    }

    #[test]
    fn every_builtin_illustration_has_a_living_profile() {
        for id in profiles::ALL_STAGE_IDS {
            assert!(profile_for(id).primitives() > 0, "{id} has no ambient layers");
        }
    }

    #[test]
    fn profile_values_stay_inside_their_valid_ranges() {
        for id in profiles::ALL_STAGE_IDS {
            let p = profile_for(id);
            for m in p.mist {
                assert!((0.0..=1.0).contains(&m.v) && (0.0..=1.0).contains(&m.alpha) && m.blobs >= 1 && (0.0..=1.0).contains(&m.depth), "{id} mist {m:?}");
            }
            for m in p.motes {
                assert!(m.band[0] < m.band[1] && m.band[0] >= 0.0 && m.band[1] <= 1.0 && (0.0..=1.0).contains(&m.alpha) && m.size > 0.0, "{id} motes {m:?}");
            }
            for g in p.glows {
                assert!((0.0..=1.0).contains(&g.u) && (0.0..=1.0).contains(&g.v) && g.radius > 0.0 && g.period > 0.0 && (0.0..=1.0).contains(&g.alpha), "{id} glow {g:?}");
            }
            for b in p.beams {
                assert!((0.0..=1.0).contains(&b.u) && b.width > 0.0 && b.period > 0.0 && (0.0..=1.0).contains(&b.alpha), "{id} beam {b:?}");
            }
        }
    }
}
