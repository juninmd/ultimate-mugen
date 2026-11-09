use super::hud::draw_hud;
use ume_engine::matchs::Match;
use ume_formats::stage::MugenStage;

const W: usize = 640;
const H: usize = 360;

fn tiles(t: i32, p0: f32, span: f32, step: f32, lim: f32) -> Vec<f32> {
    match t {
        1 => {
            let st = p0.rem_euclid(step) - step;
            if !st.is_finite() {
                return vec![];
            }
            let (mut v, mut p) = (vec![], st);
            while p < lim && v.len() <= W + 2 {
                if p + span > 0.0 {
                    v.push(p);
                }
                p += step;
            }
            v
        }
        n if n > 1 => {
            let i0 = (((-span - p0) / step).floor() as i64).clamp(0, n as i64);
            let i1 =
                (((lim - p0) / step).ceil() as i64).clamp(i0, (i0 + W as i64 + 2).min(n as i64));
            (i0..i1).map(|i| p0 + i as f32 * step).collect()
        }
        _ => vec![p0],
    }
}

fn blit_sprite(
    rgba: &mut [u8],
    img: &[u8],
    sw: usize,
    sh: usize,
    x0: f32,
    y0: f32,
    hflip: bool,
    alpha: f32,
) {
    let (vw, vh) = (W as isize, H as isize);
    let (ix0, iy0) = (x0.round() as isize, y0.round() as isize);
    let a_mul = alpha.clamp(0.0, 1.0);

    for sy in 0..sh as isize {
        let dy = iy0 + sy;
        if dy < 0 || dy >= vh {
            continue;
        }
        for sx in 0..sw as isize {
            let dx = if hflip {
                ix0 + (sw as isize - 1 - sx)
            } else {
                ix0 + sx
            };
            if dx < 0 || dx >= vw {
                continue;
            }

            let src_idx = (sy as usize * sw + sx as usize) * 4;
            let sa = (img[src_idx + 3] as f32 / 255.0) * a_mul;
            if sa <= 0.01 {
                continue;
            }

            let dst_idx = (dy as usize * W + dx as usize) * 4;
            let inv = 1.0 - sa;
            rgba[dst_idx] = (img[src_idx] as f32 * sa + rgba[dst_idx] as f32 * inv) as u8;
            rgba[dst_idx + 1] =
                (img[src_idx + 1] as f32 * sa + rgba[dst_idx + 1] as f32 * inv) as u8;
            rgba[dst_idx + 2] =
                (img[src_idx + 2] as f32 * sa + rgba[dst_idx + 2] as f32 * inv) as u8;
            rgba[dst_idx + 3] = 255;
        }
    }
}

pub fn render_frame(m: &Match, st: &MugenStage, stage_name: &str, hits: i32) -> Vec<u8> {
    let (vw, vh) = (W as f32, H as f32);
    let scl = vw / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    let scl_y = vh / if st.local_h > 0.0 { st.local_h } else { 240.0 };
    let (cx, cy) = (m.camera.x / scl, m.camera.y / scl_y);

    let mut rgba = vec![0u8; W * H * 4];
    for p in rgba.as_chunks_mut::<4>().0 {
        p.copy_from_slice(&[15, 18, 25, 255]);
    }

    // 1. Stage Background Layers (layer == 0)
    draw_stage_layers(&mut rgba, st, m, 0, cx, cy, scl, scl_y, vw, vh);

    // 2. Fighters
    let roots = m.root_indices();
    for &idx in &roots {
        let f = &m.fighters[idx];
        if !f.alive || f.destroyed {
            continue;
        }
        let Some(fr) = f.frame() else { continue };
        let Some(spr) = f.def.sprites.get(&(fr.group, fr.index)) else {
            continue;
        };
        let Some(img) = &spr.image else { continue };

        let fx = (f.pos.x - m.camera.x) * scl + vw * 0.5;
        let fy = st.zoffset * scl_y + f.pos.y * scl_y - m.camera.y * scl_y;
        let hflip = f.facing < 0.0;
        let draw_x = if hflip {
            fx - (img.w as f32 - spr.axis_x)
        } else {
            fx - spr.axis_x
        };
        let draw_y = fy - spr.axis_y;

        blit_sprite(
            &mut rgba,
            &img.rgba,
            img.w as usize,
            img.h as usize,
            draw_x,
            draw_y,
            hflip,
            1.0,
        );
    }

    // 3. Stage Foreground Layers (layer == 1)
    draw_stage_layers(&mut rgba, st, m, 1, cx, cy, scl, scl_y, vw, vh);

    // 4. HUD
    let l1 = if roots.len() > 0 {
        m.fighters[roots[0]].life / m.fighters[roots[0]].life_max.max(1.0)
    } else {
        1.0
    };
    let l2 = if roots.len() > 1 {
        m.fighters[roots[1]].life / m.fighters[roots[1]].life_max.max(1.0)
    } else {
        1.0
    };
    draw_hud(&mut rgba, W, H, stage_name, l1, l2, hits);

    rgba
}

fn draw_stage_layers(
    rgba: &mut [u8],
    st: &MugenStage,
    m: &Match,
    layer_target: i32,
    cx: f32,
    cy: f32,
    scl: f32,
    scl_y: f32,
    vw: f32,
    vh: f32,
) {
    let state = m.bg.as_ref();
    for (k, l) in st.layers.iter().enumerate() {
        if l.layer != layer_target {
            continue;
        }
        if state.is_some_and(|s| !s.visible(k)) {
            continue;
        }

        let key = state.map(|s| s.sprite(k)).unwrap_or(l.sprite);
        let Some(meta) = st.sprites.get(&key) else {
            continue;
        };
        let (mw, mh) = (meta.w as usize, meta.h as usize);
        if mw == 0 || mh == 0 || meta.rgba.len() < mw * mh * 4 {
            continue;
        }

        let off = state.map(|s| s.offset(k)).unwrap_or([0.0, 0.0]);
        let sxp = (l.start[0] + off[0] - cx * l.delta[0]) * scl + vw * 0.5;
        let syp = (l.start[1] + off[1] - cy * l.delta[1]) * scl_y;
        let (w, h) = (meta.w as f32 * scl, meta.h as f32 * scl_y);
        let (x0, y0) = (
            sxp - meta.axis_x as f32 * scl,
            syp - meta.axis_y as f32 * scl_y,
        );
        let (stx, sty) = (
            (w + l.tile_spacing[0] * scl).max(1.0),
            (h + l.tile_spacing[1] * scl_y).max(1.0),
        );

        let xs = tiles(l.tile[0], x0, w, stx, vw);
        let ys = tiles(l.tile[1], y0, h, sty, vh);
        let alpha = (l.alpha[0] / 256.0).clamp(0.1, 1.0);

        for &tx in &xs {
            for &ty in &ys {
                blit_sprite(rgba, &meta.rgba, mw, mh, tx, ty, false, alpha);
            }
        }
    }
}
