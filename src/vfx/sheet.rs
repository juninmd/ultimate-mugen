use crate::gfx::*;
use std::cell::OnceCell;
use ume_formats::png::decode_png;

const HIT_SPARK_PNG: &[u8] = include_bytes!("../../assets/fx/hit_spark.png");
const EXPLOSION_PNG: &[u8] = include_bytes!("../../assets/fx/explosion.png");
const PUNCH_HIT_PNG: &[u8] = include_bytes!("../../assets/fx/pvfx_punch_hit.png");
const CRIT_HIT_PNG: &[u8] = include_bytes!("../../assets/fx/pvfx_crit_hit.png");
const SHOCKWAVE_RING_PNG: &[u8] = include_bytes!("../../assets/fx/pvfx_shockwave_ring.png");
const GROUND_SLAM_PNG: &[u8] = include_bytes!("../../assets/fx/pvfx_ground_slam.png");
const PARRY_PNG: &[u8] = include_bytes!("../../assets/fx/pvfx_parry.png");
const DASH_DUST_PNG: &[u8] = include_bytes!("../../assets/fx/pvfx_dash_dust.png");
const MAX_SPRITES: usize = 16;
const KINDS: usize = 8;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum SheetKind {
    HitSpark,
    Explosion,
    PunchHit,
    CritHit,
    ShockwaveRing,
    GroundSlam,
    Parry,
    DashDust,
}

struct SheetSpec {
    png: &'static [u8],
    cell: (f32, f32),
    pivot: (f32, f32),
    cols: usize,
    frames: usize,
    seconds: f32,
    additive: bool,
    pixel_art: bool,
}

impl SheetKind {
    fn index(self) -> usize {
        self as usize
    }

    fn spec(self) -> SheetSpec {
        match self {
            SheetKind::HitSpark => SheetSpec {
                png: HIT_SPARK_PNG,
                cell: (32.0, 32.0),
                pivot: (16.0, 16.0),
                cols: 9,
                frames: 9,
                seconds: 0.2,
                additive: true,
                pixel_art: false,
            },
            SheetKind::Explosion => SheetSpec {
                png: EXPLOSION_PNG,
                cell: (100.0, 100.0),
                pivot: (50.0, 50.0),
                cols: 6,
                frames: 22,
                seconds: 0.55,
                additive: true,
                pixel_art: false,
            },
            SheetKind::PunchHit => SheetSpec {
                png: PUNCH_HIT_PNG,
                cell: (48.0, 40.0),
                pivot: (17.0, 21.0),
                cols: 8,
                frames: 11,
                seconds: 0.66,
                additive: false,
                pixel_art: true,
            },
            SheetKind::CritHit => SheetSpec {
                png: CRIT_HIT_PNG,
                cell: (96.0, 80.0),
                pivot: (48.0, 40.0),
                cols: 8,
                frames: 11,
                seconds: 0.77,
                additive: false,
                pixel_art: true,
            },
            SheetKind::ShockwaveRing => SheetSpec {
                png: SHOCKWAVE_RING_PNG,
                cell: (120.0, 52.0),
                pivot: (60.0, 30.0),
                cols: 8,
                frames: 16,
                seconds: 1.12,
                additive: false,
                pixel_art: true,
            },
            SheetKind::Parry => SheetSpec {
                png: PARRY_PNG,
                cell: (96.0, 80.0),
                pivot: (48.0, 38.0),
                cols: 8,
                frames: 13,
                seconds: 0.78,
                additive: false,
                pixel_art: true,
            },
            SheetKind::DashDust => SheetSpec {
                png: DASH_DUST_PNG,
                cell: (68.0, 32.0),
                pivot: (24.0, 28.0),
                cols: 8,
                frames: 13,
                seconds: 0.78,
                additive: false,
                pixel_art: true,
            },
            SheetKind::GroundSlam => SheetSpec {
                png: GROUND_SLAM_PNG,
                cell: (128.0, 64.0),
                pivot: (64.0, 44.0),
                cols: 8,
                frames: 19,
                seconds: 1.33,
                additive: false,
                pixel_art: true,
            },
        }
    }
}

struct SheetSprite {
    kind: SheetKind,
    x: f32,
    y: f32,
    size: f32,
    rot: f32,
    flip: bool,
    t: f32,
}

#[derive(Default)]
pub(super) struct SheetFx {
    sprites: Vec<SheetSprite>,
    textures: [OnceCell<Option<Texture2D>>; KINDS],
}

fn load(png: &[u8], pixel_art: bool) -> Option<Texture2D> {
    let decoded = decode_png(png).ok()?;
    let rgba = match &decoded.rgba {
        Some(rgba) => rgba.clone(),
        None => ume_formats::png::png_indexed_to_rgba(&decoded),
    };
    let t = Texture2D::from_rgba8(decoded.width as u16, decoded.height as u16, &rgba);
    t.set_filter(if pixel_art { FilterMode::Nearest } else { FilterMode::Linear });
    Some(t)
}

impl SheetFx {
    pub(super) fn spawn(&mut self, kind: SheetKind, x: f32, y: f32, size: f32, rot: f32) {
        if self.sprites.len() >= MAX_SPRITES {
            self.sprites.remove(0);
        }
        self.sprites.push(SheetSprite { kind, x, y, size, rot, flip: false, t: 0.0 });
    }

    pub(super) fn spawn_facing(&mut self, kind: SheetKind, x: f32, y: f32, size: f32, facing: f32) {
        self.spawn(kind, x, y, size, 0.0);
        if let Some(s) = self.sprites.last_mut() {
            s.flip = facing < 0.0;
        }
    }

    pub(super) fn guard(&mut self, x: f32, y: f32, size: f32) {
        self.spawn(SheetKind::Parry, x, y, size, 0.0);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn hit(&mut self, x: f32, y: f32, damage: f32, heavy: bool, flare: f32, ring: f32, spark_rot: f32, feet: Option<(f32, f32)>) {
        self.spawn(SheetKind::HitSpark, x, y, flare * 1.8, spark_rot);
        if heavy || damage >= 90.0 {
            self.spawn(SheetKind::CritHit, x, y, ring * 1.6, 0.0);
        } else {
            self.spawn(SheetKind::PunchHit, x, y, ring * 1.5, 0.0);
        }
        if damage < 150.0 {
            return;
        }
        self.spawn(SheetKind::Explosion, x, y, ring * 1.4, 0.0);
        if let Some((fx, fy)) = feet {
            self.spawn(SheetKind::ShockwaveRing, fx, fy, 150.0, 0.0);
            self.spawn(SheetKind::GroundSlam, fx, fy, 110.0, 0.0);
        }
    }

    pub(super) fn update(&mut self, dt: f32) {
        for s in &mut self.sprites {
            s.t += dt;
        }
        self.sprites.retain(|s| s.t < s.kind.spec().seconds);
    }

    pub(super) fn clear(&mut self) {
        self.sprites.clear();
    }

    fn texture(&self, kind: SheetKind) -> Option<&Texture2D> {
        self.textures[kind.index()]
            .get_or_init(|| {
                let spec = kind.spec();
                load(spec.png, spec.pixel_art)
            })
            .as_ref()
    }

    pub(super) fn draw(&self, tp: &dyn Fn(f32, f32) -> Vec2, scale: f32, gain: f32, additive: bool) {
        for s in &self.sprites {
            let spec = s.kind.spec();
            if spec.additive != additive {
                continue;
            }
            let Some(tex) = self.texture(s.kind) else {
                continue;
            };
            let progress = (s.t / spec.seconds).clamp(0.0, 0.999);
            let frame = (progress * spec.frames as f32) as usize;
            let (col, row) = (frame % spec.cols, frame / spec.cols);
            let k = s.size * scale / spec.cell.0;
            let at = tp(s.x, s.y);
            let pivot_x = if s.flip { spec.cell.0 - spec.pivot.0 } else { spec.pivot.0 };
            let alpha = if additive { (1.0 - progress).sqrt() } else { 1.0 } * gain.clamp(0.0, 1.0);
            draw_texture_ex(
                tex,
                at.x - pivot_x * k,
                at.y - spec.pivot.1 * k,
                Color::new(1.0, 1.0, 1.0, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(spec.cell.0 * k, spec.cell.1 * k)),
                    source: Some(Rect::new(col as f32 * spec.cell.0, row as f32 * spec.cell.1, spec.cell.0, spec.cell.1)),
                    rotation: s.rot,
                    flip_x: s.flip,
                    ..Default::default()
                },
            );
        }
    }
}
