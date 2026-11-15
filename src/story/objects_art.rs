use super::objects::catalog;
use super::objects_textures::with_texture;
use crate::gfx::*;
use crate::render::Gfx;
use ume_engine::matchs::Match;
use ume_engine::stage_objects::{Breakable, Pickup, PICKUP_LIFETIME, SHAKE_TICKS};

pub const PICKUP_FRAMES: usize = 6;
pub const PICKUP_SIZE: f32 = 24.0;
const REMAINS_FADE_START: i32 = 200;
const REMAINS_FADE_TICKS: i32 = 60;
const EXPIRY_BLINK_TICKS: i32 = 150;
const NEIGHBOUR_RANGE: f32 = 150.0;

fn in_front_of_fighters(m: &Match, x: f32, lane: f32) -> bool {
    m.fighters
        .iter()
        .filter(|f| !f.is_helper && !f.destroyed && (f.pos.x - x).abs() < NEIGHBOUR_RANGE)
        .min_by(|a, b| (a.pos.x - x).abs().total_cmp(&(b.pos.x - x).abs()))
        .is_some_and(|f| lane > f.lane)
}

struct View<'a> {
    g: &'a Gfx,
    m: &'a Match,
    shake: (f32, f32),
}

impl View<'_> {
    fn point(&self, x: f32, y: f32) -> Vec2 {
        let s = self.g.view_scale;
        vec2(
            self.g.ox + (x - self.m.camera.x + self.m.view_w / 2.0 + self.shake.0) * s,
            self.g.oy + (y - self.m.camera.y + self.m.stage.zoffset + self.shake.1) * s,
        )
    }

    fn on_screen(&self, x: f32, half_width: f32) -> bool {
        (x - self.m.camera.x).abs() < self.m.view_w / 2.0 + half_width + 8.0
    }

    fn shadow(&self, x: f32, lane: f32, half_width: f32, alpha: f32) {
        let p = self.point(x, lane);
        let s = self.g.view_scale;
        draw_ellipse(
            p.x,
            p.y,
            half_width * s,
            half_width * 0.2 * s,
            0.0,
            Color::new(0.0, 0.0, 0.05, 0.38 * alpha),
        );
    }

    fn sprite(
        &self,
        texture: &Texture2D,
        frames: usize,
        frame: usize,
        feet: Vec2,
        size: [f32; 2],
        color: Color,
    ) {
        let s = self.g.view_scale;
        let cell = texture.width() / frames.max(1) as f32;
        let (w, h) = (size[0] * s, size[1] * s);
        draw_texture_ex(
            texture,
            feet.x - w * 0.5,
            feet.y - h,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                source: Some(Rect::new(
                    cell * frame.min(frames.saturating_sub(1)) as f32,
                    0.0,
                    cell,
                    texture.height(),
                )),
                ..Default::default()
            },
        );
    }

    fn breakable(&self, b: &Breakable) {
        let Some(kind) = catalog().kind(b.kind) else {
            return;
        };
        if !self.on_screen(b.x, kind.size[0] * 0.5) {
            return;
        }
        let frames = kind.frames.max(1);
        let fade = if b.broken && !b.solid {
            1.0 - ((b.broken_ticks - REMAINS_FADE_START) as f32 / REMAINS_FADE_TICKS as f32)
                .clamp(0.0, 1.0)
        } else {
            1.0
        };
        if fade <= 0.0 {
            return;
        }
        let frame = if b.broken {
            frames - 1
        } else {
            ((b.damage_ratio() * (frames - 1) as f32) as usize).min(frames.saturating_sub(2))
        };
        let wobble = (b.shake as f32 * 2.3).sin() * b.shake as f32 * 0.32;
        let feet = self.point(b.x + wobble, b.lane);
        self.shadow(b.x, b.lane, kind.size[0] * 0.46, fade);
        with_texture(&kind.sprite, |texture| {
            self.sprite(
                texture,
                frames,
                frame,
                feet,
                kind.size,
                Color::new(1.0, 1.0, 1.0, fade),
            );
            if b.shake > SHAKE_TICKS - 4 && self.g.fx_gain > 0.2 {
                set_blend(GfxBlend::Add);
                self.sprite(
                    texture,
                    frames,
                    frame,
                    feet,
                    kind.size,
                    Color::new(1.0, 1.0, 1.0, 0.5),
                );
                set_blend(GfxBlend::Alpha);
            }
        });
    }

    fn pickup(&self, p: &Pickup) {
        let Some(food) = catalog().food(p.spec.kind) else {
            return;
        };
        let left = PICKUP_LIFETIME - p.age;
        if !self.on_screen(p.pos.x, PICKUP_SIZE)
            || (left < EXPIRY_BLINK_TICKS && (p.age / 4) % 2 == 0)
        {
            return;
        }
        let resting = p.pos.y >= 0.0 && p.vel.y == 0.0;
        let bob = if resting {
            (p.age as f32 * 0.11).sin() * 1.6 - 2.0
        } else {
            0.0
        };
        let feet = self.point(p.pos.x, p.pos.y + p.lane + bob);
        self.shadow(p.pos.x, p.lane, PICKUP_SIZE * 0.4, 0.8);
        if self.g.fx_gain > 0.2 {
            let s = self.g.view_scale;
            let pulse = 0.5 + 0.5 * (p.age as f32 * 0.16).sin();
            set_blend(GfxBlend::Add);
            let radius = PICKUP_SIZE * (0.55 + 0.12 * pulse) * s;
            draw_ellipse(
                feet.x,
                feet.y - PICKUP_SIZE * 0.5 * s,
                radius,
                radius,
                0.0,
                Color::new(1.0, 0.92, 0.55, 0.16 + 0.08 * pulse),
            );
            set_blend(GfxBlend::Alpha);
        }
        let frame = (p.age as usize / 5) % PICKUP_FRAMES;
        with_texture(&food.sprite, |texture| {
            self.sprite(
                texture,
                PICKUP_FRAMES,
                frame,
                feet,
                [PICKUP_SIZE, PICKUP_SIZE],
                WHITE,
            );
        });
    }
}

pub fn draw(g: &Gfx, m: &Match, front: bool, shx: f32, shy: f32) {
    if m.objects.breakables.is_empty() && m.objects.pickups.is_empty() {
        return;
    }
    let view = View {
        g,
        m,
        shake: (shx, shy),
    };
    for b in m
        .objects
        .breakables
        .iter()
        .filter(|b| in_front_of_fighters(m, b.x, b.lane) == front)
    {
        view.breakable(b);
    }
    for p in m
        .objects
        .pickups
        .iter()
        .filter(|p| in_front_of_fighters(m, p.pos.x, p.lane) == front)
    {
        view.pickup(p);
    }
}
