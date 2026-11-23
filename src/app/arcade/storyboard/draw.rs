use super::player::Playback;
use crate::app::App;
use crate::gfx::*;
use crate::ui::{hex, Align};
use ume_formats::screenpack::Element;

pub(super) fn draw(app: &mut App, playback: &Playback, scene_index: usize, time: f32) {
    let story = &playback.storyboard;
    let scene = &story.scenes[scene_index];
    let canvas = Canvas::new(story.localcoord);
    let clear = story.scenes[..=scene_index]
        .iter()
        .rev()
        .find_map(|scene| scene.clear_color)
        .unwrap_or([0, 0, 0]);
    app.ui.rect(
        0.0,
        0.0,
        1280.0,
        720.0,
        Color::from_rgba(clear[0], clear[1], clear[2], 255),
    );
    if let Some(name) = scene.bg.as_deref() {
        super::sprite_draw::draw_background(app, playback, name, canvas, time);
    }
    for layer in &scene.layers {
        let elapsed = time * 60.0 - layer.start_time as f32;
        if elapsed < 0.0 || layer.end_time.is_some_and(|end| time * 60.0 >= end as f32) {
            continue;
        }
        if layer.element.displaytime > 0 && elapsed >= layer.element.displaytime as f32 {
            continue;
        }
        draw_element(
            app,
            playback,
            &layer.element,
            scene.layer_pos,
            canvas,
            elapsed,
        );
    }
    super::sprite_draw::draw_fade(app, playback, scene_index, time);
    app.ui.rect(
        0.0,
        678.0,
        1280.0,
        42.0,
        Color::new(0.015, 0.01, 0.04, 0.78),
    );
    let route = app.arcade.as_ref().and_then(|run| run.storyboard_after);
    let label = match route {
        Some(super::player::After::FirstFight) => crate::i18n::tr3(
            "ARCADE · INTRODUÇÃO",
            "ARCADE · INTRO",
            "ARCADE · INTRODUCCIÓN",
        ),
        Some(super::player::After::Fight { .. }) => crate::i18n::tr3(
            "ARCADE · INTERLÚDIO",
            "ARCADE · INTERLUDE",
            "ARCADE · INTERLUDIO",
        ),
        _ => crate::i18n::tr3("ARCADE · EPÍLOGO", "ARCADE · EPILOGUE", "ARCADE · EPÍLOGO"),
    };
    app.ui
        .text(label, 26.0, 706.0, 13.0, hex("#ffcc33"), Align::Left, true);
    app.ui.text(
        &crate::i18n::tr3(
            "Toque/A/Enter avança · toque em Pular/Esc pula",
            "Tap/A/Enter advances · tap Skip/Esc skips",
            "Toque/A/Enter avanza · toca Saltar/Esc omite",
        ),
        1254.0,
        706.0,
        13.0,
        WHITE,
        Align::Right,
        false,
    );
}

#[derive(Clone, Copy)]
pub(super) struct Canvas {
    pub(super) scale: f32,
    offset: [f32; 2],
}

impl Canvas {
    pub(super) fn new(local: [f32; 2]) -> Self {
        let w = local[0].max(1.0);
        let h = local[1].max(1.0);
        let scale = (1280.0 / w).min(678.0 / h);
        Self {
            scale,
            offset: [(1280.0 - w * scale) * 0.5, (678.0 - h * scale) * 0.5],
        }
    }

    pub(super) fn point(self, x: f32, y: f32, ui: &crate::ui::Ui) -> [f32; 2] {
        [
            ui.x(self.offset[0] + x * self.scale),
            ui.y(self.offset[1] + y * self.scale),
        ]
    }
}

fn draw_element(
    app: &mut App,
    playback: &Playback,
    element: &Element,
    layer_pos: [f32; 2],
    canvas: Canvas,
    elapsed: f32,
) {
    let frame = element
        .anim
        .and_then(|id| playback.storyboard.anims.get(&id))
        .and_then(|animation| super::animation::frame_at(animation, elapsed as i32));
    if let Some(text) = element.text.as_deref() {
        let [px, py] = canvas.point(
            layer_pos[0] + element.offset[0],
            layer_pos[1] + element.offset[1],
            &app.ui,
        );
        app.ui.text(
            text,
            (px - app.ui.ox) / app.ui.s,
            (py - app.ui.oy) / app.ui.s,
            20.0 * canvas.scale * element.scale[1],
            WHITE,
            Align::Left,
            false,
        );
    }
    let (key, x, y, sx, sy, flip_x, flip_y) = if let Some(frame) = frame {
        (
            (frame.group, frame.index),
            layer_pos[0] + element.offset[0] + frame.x,
            layer_pos[1] + element.offset[1] + frame.y,
            frame.xscale * element.scale[0],
            frame.yscale * element.scale[1],
            frame.flip_h != (element.facing < 0),
            frame.flip_v != (element.vfacing < 0),
        )
    } else if let Some(key) = element.spr {
        (
            (key.0, key.1),
            layer_pos[0] + element.offset[0],
            layer_pos[1] + element.offset[1],
            element.scale[0],
            element.scale[1],
            element.facing < 0,
            element.vfacing < 0,
        )
    } else {
        return;
    };
    super::sprite_draw::draw_sprite(app, playback, key, canvas, x, y, sx, sy, flip_x, flip_y);
}
