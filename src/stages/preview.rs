use super::*;

pub fn mugen_preview(st: &MugenStage) -> Option<Arc<ImageData>> {
    if st.layers.is_empty() || st.sprites.is_empty() {
        return None;
    }
    const W: usize = 320;
    const H: usize = 180;
    let (vw, vh) = (W as f32, H as f32);
    let scl = vw / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    let scl_y = vh / if st.local_h > 0.0 { st.local_h } else { 240.0 };
    let (cx, cy) = (st.start_x, st.start_y);
    let mut rgba = vec![0u8; W * H * 4];
    for p in rgba.as_chunks_mut::<4>().0 {
        p.copy_from_slice(&[20, 20, 30, 255]);
    }
    let mut drawn = false;
    let tiles = |t: i32, p0: f32, span: f32, step: f32, lim: f32| -> Vec<f32> {
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
                let i1 = (((lim - p0) / step).ceil() as i64)
                    .clamp(i0, (i0 + W as i64 + 2).min(n as i64));
                (i0..i1).map(|i| p0 + i as f32 * step).collect()
            }
            _ => vec![p0],
        }
    };
    for front in [false, true] {
        let mut root_off = [0.0f32; 2];
        let mut root_vis = true;
        for l in &st.layers {
            let mut off = [0.0f32; 2];
            let mut vis = true;
            let mut anim_override = None;
            for def in &st.ctrl_defs {
                for c in &def.ctrls {
                    if c.start <= 0 && 0 <= c.end && (c.ids.is_empty() || c.ids.contains(&l.id)) {
                        match c.kind {
                            ume_formats::stage::BgCtrlKind::Visible(v) => vis = v,
                            ume_formats::stage::BgCtrlKind::PosSet(x, y) => {
                                if let Some(x) = x {
                                    off[0] = x - l.start[0];
                                }
                                if let Some(y) = y {
                                    off[1] = y - l.start[1];
                                }
                            }
                            ume_formats::stage::BgCtrlKind::PosAdd(x, y) => {
                                if let Some(x) = x {
                                    off[0] += x;
                                }
                                if let Some(y) = y {
                                    off[1] += y;
                                }
                            }
                            ume_formats::stage::BgCtrlKind::Anim(a) => anim_override = Some(a),
                            _ => {}
                        }
                    }
                }
            }
            if l.position_link {
                off[0] += root_off[0];
                off[1] += root_off[1];
                vis &= root_vis;
            } else {
                root_off = off;
                root_vis = vis;
            }
            if (l.layer == 1) != front || !vis {
                continue;
            }
            let scroll = l.window_scroll();
            let (wx0, wy0, wx1, wy1) = match l.clip_window() {
                Some(win) if win[2] <= win[0] || win[3] <= win[1] => continue,
                Some(win) => (
                    ((win[0] - cx * scroll[0]) * scl + vw * 0.5)
                        .floor()
                        .max(0.0) as isize,
                    ((win[1] - cy * scroll[1]) * scl_y).floor().max(0.0) as isize,
                    (((win[2] + 1.0) - cx * scroll[0]) * scl + vw * 0.5)
                        .ceil()
                        .min(vw) as isize,
                    (((win[3] + 1.0) - cy * scroll[1]) * scl_y).ceil().min(vh) as isize,
                ),
                None => (0, 0, W as isize, H as isize),
            };
            let anim = anim_override
                .and_then(|a| st.anims.get(&a))
                .or(l.anim.as_ref());
            let key = anim
                .and_then(|a| a.frames.first())
                .map(|f| (f.group, f.index))
                .unwrap_or(l.sprite);
            let Some(meta) = st.sprites.get(&key) else {
                continue;
            };
            let (mw, mh) = (meta.w as usize, meta.h as usize);
            if mw == 0
                || mh == 0
                || mw
                    .checked_mul(mh)
                    .and_then(|n| n.checked_mul(4))
                    .is_none_or(|n| meta.rgba.len() < n)
            {
                continue;
            }
            let layer_scale = l.scale_at([cx, cy]);
            let (lx, ly) = (scl * layer_scale[0], scl_y * layer_scale[1]);
            let sxp = (l.start[0] + off[0] - cx * l.delta[0]) * scl + vw * 0.5;
            let syp = (l.start[1] + off[1] - cy * l.delta[1]) * scl_y;
            let (w, h) = (meta.w as f32 * lx, meta.h as f32 * ly);
            if w < 0.5 || h < 0.5 {
                continue;
            }
            let (axis_dx, axis_dy) = (meta.axis_x as f32 * lx, meta.axis_y as f32 * ly);
            let (x0, y0) = (sxp - axis_dx, syp - axis_dy);
            let (stx, sty) = (
                (w + l.tile_spacing[0] * scl).max(1.0),
                (h + l.tile_spacing[1] * scl_y).max(1.0),
            );
            if ![scl, scl_y, sxp, syp, w, h, x0, y0, stx, sty]
                .iter()
                .all(|v| v.is_finite())
            {
                continue;
            }
            let xs = tiles(l.tile[0], x0, w, stx, vw);
            let ys = tiles(l.tile[1], y0, h, sty, vh);
            let add = l.trans.starts_with("add") && l.trans != "addalpha";
            let add1 = l.trans == "add1";
            let sub = l.trans == "sub";
            let addalpha = l.trans == "addalpha";
            let sa = (l.alpha[0] / 256.0).clamp(0.0, 1.0);
            let da = (l.alpha[1] / 256.0).clamp(0.0, 1.0);
            let transformed = l.angle != 0.0 || l.x_shear != 0.0;
            let (sin_a, cos_a) = (-l.angle).to_radians().sin_cos();
            for &tx in &xs {
                for &ty in &ys {
                    let (bx0, bx1, by0, by1) = if transformed {
                        let (pivot_x, pivot_y) = (tx + axis_dx, ty + axis_dy);
                        let mut bounds = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                        for (cxp, cyp) in [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)] {
                            let (rx, ry) = (cxp - axis_dx, cyp - axis_dy);
                            let sheared = rx - l.x_shear * ry;
                            let (qx, qy) = (
                                pivot_x + sheared * cos_a - ry * sin_a,
                                pivot_y + sheared * sin_a + ry * cos_a,
                            );
                            bounds = (
                                bounds.0.min(qx),
                                bounds.1.max(qx),
                                bounds.2.min(qy),
                                bounds.3.max(qy),
                            );
                        }
                        if !bounds.0.is_finite()
                            || !bounds.1.is_finite()
                            || !bounds.2.is_finite()
                            || !bounds.3.is_finite()
                        {
                            continue;
                        }
                        (bounds.0, bounds.1, bounds.2, bounds.3)
                    } else {
                        (tx, tx + w, ty, ty + h)
                    };
                    let dx0 = (bx0.floor() as isize).max(wx0).max(0);
                    let dx1 = (bx1.ceil() as isize).min(wx1).min(W as isize);
                    let dy0 = (by0.floor() as isize).max(wy0).max(0);
                    let dy1 = (by1.ceil() as isize).min(wy1).min(H as isize);
                    if dx0 >= dx1 || dy0 >= dy1 {
                        continue;
                    }
                    for py in dy0..dy1 {
                        for px in dx0..dx1 {
                            let (u, v) = if transformed {
                                let (dx, dy) = (
                                    px as f32 + 0.5 - (tx + axis_dx),
                                    py as f32 + 0.5 - (ty + axis_dy),
                                );
                                let ry = -dx * sin_a + dy * cos_a;
                                let rx = dx * cos_a + dy * sin_a + l.x_shear * ry;
                                ((rx + axis_dx) / w, (ry + axis_dy) / h)
                            } else {
                                ((px as f32 + 0.5 - tx) / w, (py as f32 + 0.5 - ty) / h)
                            };
                            if transformed && (!(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v))
                            {
                                continue;
                            }
                            let sy = ((v.max(0.0) * mh as f32) as usize).min(mh - 1);
                            let sx = ((u.max(0.0) * mw as f32) as usize).min(mw - 1);
                            let s = &meta.rgba[(sy * mw + sx) * 4..][..4];
                            if s[3] == 0 {
                                continue;
                            }
                            let d = &mut rgba[(py as usize * W + px as usize) * 4..][..4];
                            if add1 {
                                for c in 0..3 {
                                    d[c] = (d[c] / 2).saturating_add(s[c]);
                                }
                            } else if add {
                                for c in 0..3 {
                                    d[c] = d[c].saturating_add(s[c]);
                                }
                            } else if sub {
                                for c in 0..3 {
                                    d[c] = d[c].saturating_sub(s[c]);
                                }
                            } else if addalpha {
                                for c in 0..3 {
                                    d[c] = (s[c] as f32 * sa + d[c] as f32 * da).clamp(0.0, 255.0)
                                        as u8;
                                }
                            } else if s[3] == 255 {
                                d[..3].copy_from_slice(&s[..3]);
                            } else {
                                let a = s[3] as u32;
                                let ia = 255 - a;
                                for c in 0..3 {
                                    d[c] = ((s[c] as u32 * a + d[c] as u32 * ia) / 255) as u8;
                                }
                            }
                            drawn = true;
                        }
                    }
                }
            }
        }
    }
    drawn.then(|| ImageData::new(W as u32, H as u32, rgba))
}
