use crate::gfx::*;
use std::cell::OnceCell;
use ume_formats::png::decode_png;

const HIT_SPARK_PNG: &[u8] = include_bytes!("../../assets/fx/hit_spark.png");
const EXPLOSION_PNG: &[u8] = include_bytes!("../../assets/fx/explosion.png");
const MAX_SPRITES: usize = 12;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum SheetKind {
    HitSpark,
    Explosion,
}

struct SheetSpec {
    png: &'static [u8],
    cell: f32,
    cols: usize,
    frames: usize,
    seconds: f32,
}

impl SheetKind {
    fn spec(self) -> SheetSpec {
        match self {
            SheetKind::HitSpark => SheetSpec {
                png: HIT_SPARK_PNG,
                cell: 32.0,
                cols: 9,
                frames: 9,
                seconds: 0.2,
            },
            SheetKind::Explosion => SheetSpec {
                png: EXPLOSION_PNG,
                cell: 100.0,
                cols: 6,
                frames: 22,
                seconds: 0.55,
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
    t: f32,
}

#[derive(Default)]
pub(super) struct SheetFx {
    sprites: Vec<SheetSprite>,
    hit_spark: OnceCell<Option<Texture2D>>,
    explosion: OnceCell<Option<Texture2D>>,
}

fn load(png: &[u8]) -> Option<Texture2D> {
    let decoded = decode_png(png).ok()?;
    let rgba = match &decoded.rgba {
        Some(rgba) => rgba.clone(),
        None => ume_formats::png::png_indexed_to_rgba(&decoded),
    };
    let t = Texture2D::from_rgba8(decoded.width as u16, decoded.height as u16, &rgba);
    t.set_filter(FilterMode::Linear);
    Some(t)
}

impl SheetFx {
    pub(super) fn spawn(&mut self, kind: SheetKind, x: f32, y: f32, size: f32, rot: f32) {
        if self.sprites.len() >= MAX_SPRITES {
            self.sprites.remove(0);
        }
        self.sprites.push(SheetSprite { kind, x, y, size, rot, t: 0.0 });
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
        let slot = match kind {
            SheetKind::HitSpark => &self.hit_spark,
            SheetKind::Explosion => &self.explosion,
        };
        slot.get_or_init(|| load(kind.spec().png)).as_ref()
    }

    pub(super) fn draw(&self, tp: &dyn Fn(f32, f32) -> Vec2, scale: f32, gain: f32) {
        for s in &self.sprites {
            let Some(tex) = self.texture(s.kind) else {
                continue;
            };
            let spec = s.kind.spec();
            let progress = (s.t / spec.seconds).clamp(0.0, 0.999);
            let frame = (progress * spec.frames as f32) as usize;
            let (col, row) = (frame % spec.cols, frame / spec.cols);
            let side = s.size * scale;
            let at = tp(s.x, s.y);
            let fade = (1.0 - progress).sqrt() * gain.clamp(0.0, 1.0);
            draw_texture_ex(
                tex,
                at.x - side / 2.0,
                at.y - side / 2.0,
                Color::new(1.0, 1.0, 1.0, fade),
                DrawTextureParams {
                    dest_size: Some(vec2(side, side)),
                    source: Some(Rect::new(col as f32 * spec.cell, row as f32 * spec.cell, spec.cell, spec.cell)),
                    rotation: s.rot,
                    ..Default::default()
                },
            );
        }
    }
}
