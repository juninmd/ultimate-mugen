pub(super) const KINDS: usize = 17;

#[derive(Clone, Copy, PartialEq)]
pub(in crate::vfx) enum SheetKind {
    HitSpark,
    Explosion,
    PunchHit,
    CritHit,
    ShockwaveRing,
    GroundSlam,
    Parry,
    DashDust,
    LightningStrike,
    StaticBurst,
    FirePillar,
    SmokeBomb,
    FrostNova,
    AwakenBurst,
    SuperFlash,
    CounterFlash,
    GuardBreak,
}

pub(super) struct SheetSpec {
    pub png: &'static [u8],
    pub cell: (f32, f32),
    pub pivot: (f32, f32),
    pub cols: usize,
    pub frames: usize,
    pub seconds: f32,
    pub additive: bool,
    pub pixel_art: bool,
}

const HITSPARK_PNG: &[u8] = include_bytes!("../../../assets/fx/hit_spark.png");
const EXPLOSION_PNG: &[u8] = include_bytes!("../../../assets/fx/explosion.png");
const PUNCHHIT_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_punch_hit.png");
const CRITHIT_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_crit_hit.png");
const SHOCKWAVERING_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_shockwave_ring.png");
const GROUNDSLAM_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_ground_slam.png");
const PARRY_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_parry.png");
const DASHDUST_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_dash_dust.png");
const LIGHTNINGSTRIKE_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_lightning_strike.png");
const STATICBURST_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_static_burst.png");
const FIREPILLAR_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_fire_pillar.png");
const SMOKEBOMB_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_smoke_bomb.png");
const FROSTNOVA_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_frost_nova.png");
const AWAKENBURST_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_awaken_burst.png");
const SUPERFLASH_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_super_flash.png");
const COUNTERFLASH_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_counter_flash.png");
const GUARDBREAK_PNG: &[u8] = include_bytes!("../../../assets/fx/pvfx_guard_break.png");

const fn spec(png: &'static [u8], cell: (f32, f32), pivot: (f32, f32), cols: usize, frames: usize, millis: u32, additive: bool) -> SheetSpec {
    SheetSpec { png, cell, pivot, cols, frames, seconds: millis as f32 / 1000.0, additive, pixel_art: !additive }
}

const fn pixel_spec(png: &'static [u8], cell: (f32, f32), pivot: (f32, f32), cols: usize, frames: usize, millis: u32, additive: bool) -> SheetSpec {
    let mut sheet = spec(png, cell, pivot, cols, frames, millis, additive);
    sheet.pixel_art = true;
    sheet
}

impl SheetKind {
    pub(super) fn index(self) -> usize {
        self as usize
    }

    pub(super) fn spec(self) -> SheetSpec {
        match self {
            SheetKind::HitSpark => pixel_spec(HITSPARK_PNG, (32.0, 32.0), (16.0, 16.0), 9, 9, 200, true),
            SheetKind::Explosion => spec(EXPLOSION_PNG, (100.0, 100.0), (50.0, 50.0), 6, 22, 550, true),
            SheetKind::PunchHit => spec(PUNCHHIT_PNG, (48.0, 40.0), (17.0, 21.0), 8, 11, 660, false),
            SheetKind::CritHit => spec(CRITHIT_PNG, (96.0, 80.0), (48.0, 40.0), 8, 11, 770, false),
            SheetKind::ShockwaveRing => spec(SHOCKWAVERING_PNG, (120.0, 52.0), (60.0, 30.0), 8, 16, 1120, false),
            SheetKind::GroundSlam => spec(GROUNDSLAM_PNG, (128.0, 64.0), (64.0, 44.0), 8, 19, 1330, false),
            SheetKind::Parry => spec(PARRY_PNG, (96.0, 80.0), (48.0, 38.0), 8, 13, 780, false),
            SheetKind::DashDust => spec(DASHDUST_PNG, (68.0, 32.0), (24.0, 28.0), 8, 13, 780, false),
            SheetKind::LightningStrike => spec(LIGHTNINGSTRIKE_PNG, (96.0, 128.0), (38.0, 118.0), 8, 18, 1440, false),
            SheetKind::StaticBurst => spec(STATICBURST_PNG, (112.0, 112.0), (56.0, 58.0), 8, 13, 780, false),
            SheetKind::FirePillar => spec(FIREPILLAR_PNG, (64.0, 112.0), (32.0, 102.0), 8, 24, 1680, false),
            SheetKind::SmokeBomb => spec(SMOKEBOMB_PNG, (64.0, 56.0), (32.0, 50.0), 8, 21, 1470, false),
            SheetKind::FrostNova => spec(FROSTNOVA_PNG, (120.0, 76.0), (60.0, 50.0), 8, 18, 1440, false),
            SheetKind::AwakenBurst => spec(AWAKENBURST_PNG, (96.0, 112.0), (48.0, 106.0), 4, 8, 720, false),
            SheetKind::SuperFlash => spec(SUPERFLASH_PNG, (96.0, 96.0), (48.0, 48.0), 4, 16, 640, false),
            SheetKind::CounterFlash => spec(COUNTERFLASH_PNG, (96.0, 96.0), (48.0, 48.0), 4, 8, 480, false),
            SheetKind::GuardBreak => spec(GUARDBREAK_PNG, (96.0, 96.0), (48.0, 48.0), 4, 8, 560, false),
        }
    }
}
