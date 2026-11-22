use super::*;

/// One parallax layer. `front` layers are drawn after the fighters; `dynamic` layers have no bitmap (animated by `stagefx`).
pub struct Layer {
    pub image: Arc<ImageData>,
    pub w: f32,
    pub h: f32,
    pub y: f32,
    pub delta: f32,
    pub dy: f32,
    pub front: bool,
    pub dynamic: bool,
    /** empty rows (units) cropped off the top of the bitmap: draw at `y + trim` */
    pub trim: f32,
}
#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub s: f32,
    pub a: f32,
    pub c: [u8; 3],
}

pub struct MugenBg {
    pub stage: Arc<MugenStage>,
    pub images: std::collections::HashMap<(i32, i32), Arc<ImageData>>,
    pub scl: f32,
    pub scl_y: f32,
}

/// Which built-in look a stage has (drives the animated / reactive layer in `stagefx`); `Custom` = MUGEN stages (untouched).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StageKind {
    Dojo,
    Neon,
    Bamboo,
    Plains,
    Spirit,
    Volcano,
    Arena,
    #[default]
    Custom,
}

pub enum StageArt {
    Procedural {
        back: Vec<Layer>,
        particles: Vec<Particle>,
        tint: Option<[f32; 4]>,
    },
    Illustrated { image: Arc<ImageData>, frames: Vec<Arc<ImageData>>, ambience: &'static ambience::Profile, span: f32 },
    Mugen(Box<MugenBg>),
}
pub struct Stage {
    pub info: StageInfo,
    pub art: StageArt,
    pub music: u32,
    pub kind: StageKind,
    pub bgm: Option<Arc<(Vec<u8>, MusicLoop)>>,
}

pub struct Rng(pub(super) u32);
impl Rng {
    pub fn f(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0 as f32 / 4294967296.0
    }
}
