use super::layers::draw_transformed_layer_quad;
use super::sky::SkyCache;
use super::Gfx;
use crate::gfx::*;
use crate::stages::MugenBg;
use ume_engine::matchs::Match;

pub(super) fn draw_mugen_stage(g: &mut Gfx, m: &Match, bg: &MugenBg, front: bool, shx: f32, shy: f32, sky: &mut SkyCache) {
    let s = g.view_scale;
    let scl = bg.scl;
    let scl_y = bg.scl_y;
    let (cx, cy) = (m.camera.x / scl, m.camera.y / scl_y);
    let state = m.bg.as_ref();
    let mut sky_done = false;
    for (k, l) in bg.stage.layers.iter().enumerate() {
        if !front && !sky_done && k >= 1 {
            sky.draw(g, m, &bg.stage);
            sky_done = true;
        }
        if (l.layer == 1) != front {
            continue;
        }
        if state.is_some_and(|st| !st.visible(k)) {
            continue;
        }
        let key = match state {
            Some(st) => st.sprite(k),
            None => l.sprite,
        };
        let (Some(img), Some(meta)) = (bg.images.get(&key), bg.stage.sprites.get(&key))
        else {
            continue;
        };
        let t = g.texture(img, true);
        let off = match state {
            Some(st) => st.offset(k),
            None => {
                let gt = m.game_time as f32;
                [l.velocity[0] * gt, l.velocity[1] * gt]
            }
        };
        let layer_scale = l.scale_at([cx, cy]);
        let (lx, ly) = (scl * layer_scale[0], scl_y * layer_scale[1]);
        let sxp = (l.start[0] + off[0] - cx * l.delta[0]) * scl + m.view_w / 2.0 + shx;
        let syp = (l.start[1] + off[1] - cy * l.delta[1]) * scl_y
            + (m.view_h - 240.0) / 2.0
            + shy;
        let (w, h) = (meta.w as f32 * lx, meta.h as f32 * ly);
        let (axis_dx, axis_dy) = (meta.axis_x as f32 * lx, meta.axis_y as f32 * ly);
        let (x0, y0) = (sxp - axis_dx, syp - axis_dy);
        let (stx, sty) = (w + l.tile_spacing[0] * scl, h + l.tile_spacing[1] * scl_y);
        let add = l.trans.starts_with("add") && l.trans != "addalpha";
        let sub = l.trans == "sub";
        let alpha = if l.trans == "addalpha" {
            (l.alpha[0] / 256.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        if add {
            set_blend(GfxBlend::Add);
        } else if sub {
            set_blend(GfxBlend::Sub);
        }
        let clip = l.clip_window();
        if let Some(win) = clip {
            let scroll = l.window_scroll();
            let wx = (win[0] - cx * scroll[0]) * scl + m.view_w / 2.0 + shx;
            let wy = (win[1] - cy * scroll[1]) * scl_y + (m.view_h - 240.0) / 2.0 + shy;
            let (ww, wh) = (
                (win[2] - win[0] + 1.0) * scl,
                (win[3] - win[1] + 1.0) * scl_y,
            );
            set_scissor(Some((
                (g.ox + wx * s) as i32,
                (g.oy + wy * s) as i32,
                (ww * s).max(0.0) as i32,
                (wh * s).max(0.0) as i32,
            )));
        }
        let tint = Color::new(1.0, 1.0, 1.0, alpha);
        let transformed = l.angle != 0.0 || l.x_shear != 0.0;
        let for_each_tile = |tile_mode: i32, p0: f32, step: f32, limit: f32, cb: &mut dyn FnMut(f32)| {
            match tile_mode {
                1 => {
                    if step > 0.0 {
                        let mut p = p0 - (p0 / step).ceil() * step;
                        while p < limit {
                            cb(p);
                            p += step;
                        }
                    } else {
                        cb(p0);
                    }
                }
                n if n > 1 => {
                    for i in 0..n {
                        cb(p0 + i as f32 * step);
                    }
                }
                _ => cb(p0),
            }
        };
        for_each_tile(l.tile[0], x0, stx, m.view_w, &mut |x| {
            for_each_tile(l.tile[1], y0, sty, m.view_h, &mut |y| {
                if transformed {
                    draw_transformed_layer_quad(
                        &t,
                        [g.ox + (x + axis_dx) * s, g.oy + (y + axis_dy) * s],
                        [axis_dx * s, axis_dy * s],
                        [w * s, h * s],
                        l.angle,
                        l.x_shear,
                        tint,
                    );
                } else {
                    draw_texture_ex(
                        &t,
                        g.ox + x * s,
                        g.oy + y * s,
                        tint,
                        DrawTextureParams {
                            dest_size: Some(vec2(w * s + 0.5, h * s + 0.5)),
                            ..Default::default()
                        },
                    );
                }
            });
        });
        if clip.is_some() {
            set_scissor(None);
        }
        if add || sub {
            set_blend(GfxBlend::Alpha);
        }
    }
    if !front && !sky_done {
        sky.draw(g, m, &bg.stage);
    }

}
