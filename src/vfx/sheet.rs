use crate::gfx::*;
use std::cell::OnceCell;
use ume_formats::png::decode_png;

mod kind;

use kind::KINDS;
pub(super) use kind::SheetKind;

const MAX_SPRITES: usize = 20;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum SheetLayer {
    BehindFighters,
    Foreground,
    Additive,
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

    pub(super) fn guard(&mut self, x: f32, y: f32, size: f32, frost_at: Option<(f32, f32)>) {
        self.spawn(SheetKind::Parry, x, y, size, 0.0);
        if let Some((fx, fy)) = frost_at {
            self.spawn(SheetKind::FrostNova, fx, fy, 120.0, 0.0);
        }
    }

    pub(super) fn guard_break(&mut self, x: f32, y: f32) {
        self.spawn(SheetKind::GuardBreak, x, y, 96.0, 0.0);
    }

    pub(super) fn counter(&mut self, x: f32, y: f32) {
        self.spawn(SheetKind::CounterFlash, x, y, 86.0, 0.0);
    }

    pub(super) fn super_cast(&mut self, x: f32, chest_y: f32, feet: f32) {
        self.spawn(SheetKind::LightningStrike, x, feet, 110.0, 0.0);
        self.spawn(SheetKind::StaticBurst, x, chest_y, 100.0, 0.0);
        self.spawn(SheetKind::SuperFlash, x, chest_y, 124.0, 0.0);
    }

    pub(super) fn transform(&mut self, x: f32, feet: f32) {
        self.spawn(SheetKind::FirePillar, x, feet, 96.0, 0.0);
        self.spawn(SheetKind::AwakenBurst, x, feet, 120.0, 0.0);
    }

    pub(super) fn knockout(&mut self, x: f32, feet: f32) {
        self.spawn(SheetKind::SmokeBomb, x, feet, 120.0, 0.0);
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

    pub(super) fn available(&self, kind: SheetKind) -> bool {
        self.texture(kind).is_some()
    }

    pub(super) fn draw_spark(&self, center: Vec2, size: f32, rotation: f32, progress: f32, tint: Color) -> bool {
        let kind = SheetKind::HitSpark;
        let spec = kind.spec();
        let Some(tex) = self.texture(kind) else { return false };
        let frame = (progress.clamp(0.0, 0.999) * spec.frames as f32) as usize;
        let (col, row) = (frame % spec.cols, frame / spec.cols);
        let k = size / spec.cell.0;
        draw_texture_ex(
            tex,
            center.x - spec.pivot.0 * k,
            center.y - spec.pivot.1 * k,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(spec.cell.0 * k, spec.cell.1 * k)),
                source: Some(Rect::new(col as f32 * spec.cell.0, row as f32 * spec.cell.1, spec.cell.0, spec.cell.1)),
                rotation,
                ..Default::default()
            },
        );
        true
    }

    pub(super) fn draw(
        &self,
        tp: &dyn Fn(f32, f32) -> Vec2,
        scale: f32,
        gain: f32,
        layer: SheetLayer,
    ) {
        for s in &self.sprites {
            let spec = s.kind.spec();
            let sprite_layer = if matches!(s.kind, SheetKind::FirePillar | SheetKind::AwakenBurst) {
                SheetLayer::BehindFighters
            } else if spec.additive {
                SheetLayer::Additive
            } else {
                SheetLayer::Foreground
            };
            if layer != sprite_layer {
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
            let alpha = if spec.additive { (1.0 - progress).sqrt() } else { 1.0 } * gain.clamp(0.0, 1.0);
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
