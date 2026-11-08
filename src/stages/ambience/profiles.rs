use super::{Beam, Glow, Mist, MoteKind, Motes, Profile};

mod classic;
mod wonders;

pub const ALL_STAGE_IDS: [&str; 21] = [
    "dojo", "neon", "bamboo", "plains", "spirit", "volcano", "brazil", "great_wall", "petra", "christ_redeemer", "texas", "chichen_itza",
    "colosseum", "taj_mahal", "pyramids_giza", "hanging_gardens", "temple_artemis", "statue_zeus", "mausoleum_halicarnassus",
    "colossus_rhodes", "lighthouse_alexandria",
];

const DEFAULT_MOTES: [Motes; 1] = [dust(18, 0xffffff, 0.25)];
const DEFAULT: Profile = Profile { mist: &[], motes: &DEFAULT_MOTES, glows: &[], beams: &[] };

/// Ambient layers of a built-in illustrated stage; unknown ids get a few drifting dust motes.
pub fn profile_for(id: &str) -> &'static Profile {
    classic::profile(id).or_else(|| wonders::profile(id)).unwrap_or(&DEFAULT)
}

pub(super) const fn mist(v: f32, height: f32, speed: f32, depth: f32, color: u32, alpha: f32, blobs: u8) -> Mist {
    Mist { v, height, speed, depth, color, alpha, blobs, front: false }
}

pub(super) const fn fog_front(mut m: Mist) -> Mist {
    m.front = true;
    m
}

#[allow(clippy::too_many_arguments)]
pub(super) const fn motes(kind: MoteKind, count: u16, rise: f32, drift: f32, size: f32, band: [f32; 2], color: u32, alpha: f32) -> Motes {
    Motes { kind, count, depth: 0.4, rise, drift, size, band, color, alpha, sway: 6.0, front: false }
}

pub(super) const fn dust(count: u16, color: u32, alpha: f32) -> Motes {
    motes(MoteKind::Dust, count, 2.0, 7.0, 1.6, [0.15, 0.95], color, alpha)
}

pub(super) const fn in_front(mut m: Motes) -> Motes {
    m.front = true;
    m.depth = 0.9;
    m
}

pub(super) const fn glow(u: f32, v: f32, radius: f32, color: u32, alpha: f32, flicker: f32, period: f32) -> Glow {
    Glow { u, v, radius, color, alpha, flicker, period }
}

pub(super) const fn beam(u: f32, width: f32, slant: f32, color: u32, alpha: f32, period: f32) -> Beam {
    Beam { u, width, slant, color, alpha, period }
}
