use super::draw::Canvas;
use super::player::Playback;
use crate::app::App;
use crate::gfx::*;

pub(super) fn draw_background(
    app: &mut App,
    playback: &Playback,
    name: &str,
    canvas: Canvas,
    time: f32,
) {
    for layer in playback.storyboard.background(name).layers {
        if let Some(animation) = layer.anim.as_ref() {
            if let Some(frame) = super::animation::frame_at(animation, (time * 60.0) as i32) {
                draw_sprite(
                    app,
                    playback,
                    (frame.group, frame.index),
                    canvas,
                    layer.start[0] + layer.velocity[0] * time + frame.x,
                    layer.start[1] + layer.velocity[1] * time + frame.y,
                    frame.xscale,
                    frame.yscale,
                    frame.flip_h,
                    frame.flip_v,
                );
            }
        } else {
            draw_sprite(
                app,
                playback,
                layer.sprite,
                canvas,
                layer.start[0] + layer.velocity[0] * time,
                layer.start[1] + layer.velocity[1] * time,
                1.0,
                1.0,
                false,
                false,
            );
        }
    }
}

pub(super) fn draw_sprite(
    app: &mut App,
    playback: &Playback,
    key: (i32, i32),
    canvas: Canvas,
    x: f32,
    y: f32,
    sx: f32,
    sy: f32,
    flip_x: bool,
    flip_y: bool,
) {
    let Some(sprite) = playback.sprites.get(&key) else {
        return;
    };
    let scale_x = (sx.abs() * canvas.scale).max(0.001);
    let scale_y = (sy.abs() * canvas.scale).max(0.001);
    let [left, top] = canvas.point(x - sprite.axis[0], y - sprite.axis[1], &app.ui);
    draw_texture_ex(
        &sprite.texture,
        left,
        top,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(
                sprite.texture.width() * scale_x * app.ui.s,
                sprite.texture.height() * scale_y * app.ui.s,
            )),
            flip_x,
            flip_y,
            ..Default::default()
        },
    );
}

pub(super) fn draw_fade(app: &App, playback: &Playback, scene_index: usize, time: f32) {
    let scene = &playback.storyboard.scenes[scene_index];
    let ticks = time * 60.0;
    let end = scene.end_time.max(1) as f32;
    let fade_in = scene.fade_in.max(0) as f32;
    let fade_out = scene.fade_out.max(0) as f32;
    let alpha = if fade_in > 0.0 && ticks < fade_in {
        1.0 - ticks / fade_in
    } else if fade_out > 0.0 && ticks > end - fade_out {
        ((ticks - (end - fade_out)) / fade_out).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if alpha > 0.0 {
        let color = if ticks < fade_in {
            playback.fade_colors[scene_index].0
        } else {
            playback.fade_colors[scene_index].1
        }
        .unwrap_or([0, 0, 0]);
        app.ui.rect(
            0.0,
            0.0,
            1280.0,
            720.0,
            Color::from_rgba(color[0], color[1], color[2], (alpha * 255.0) as u8),
        );
    }
}
