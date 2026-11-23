use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, PixmapPaint, Transform};
use crate::chardef::{AnimFrame, Blend, CharDef};
use crate::matchs::Match;

pub const W: u32 = 420;
pub const H: u32 = 240;

fn pm(img: &crate::chardef::ImageData) -> Option<Pixmap> {
    let mut p = Pixmap::new(img.w.max(1), img.h.max(1))?;
    for (d, s) in p
        .data_mut()
        .chunks_exact_mut(4)
        .zip(img.rgba().chunks_exact(4))
    {
        let a = s[3] as u32;
        d[0] = (s[0] as u32 * a / 255) as u8;
        d[1] = (s[1] as u32 * a / 255) as u8;
        d[2] = (s[2] as u32 * a / 255) as u8;
        d[3] = s[3];
    }
    Some(p)
}

fn blit(
    canvas: &mut Pixmap,
    def: &CharDef,
    fr: &AnimFrame,
    pos: (f32, f32),
    facing: f32,
    scale: f32,
    camx: f32,
    alpha: f32,
) {
    let key = (fr.group, fr.index);
    let (Some(img), Some(sp)) = (def.rgba_sprite(key), def.sprites.get(&key)) else {
        return;
    };
    let Some(src) = pm(&img) else { return };
    let ground = H as f32 - 36.0;
    let sx = (W as f32 / 2.0) + (pos.0 - camx) * scale;
    let sy = ground + pos.1 * scale;
    let unit = if sp.scale > 0.0 { sp.scale } else { 1.0 };
    let pix = if sp.width > 0.0 { sp.width * unit / img.w as f32 } else { 1.0 };
    let (ax, ay, w) = (sp.axis_x * unit, sp.axis_y * unit, img.w as f32 * pix);
    let base = if fr.flip_h == (facing >= 0.0) { -(w - ax) } else { -ax };
    let wx = sx + (fr.x * facing.signum() + base) * scale;
    let wy = sy + (fr.y - ay) * scale;
    let ss = pix * scale;
    let t = if (facing >= 0.0) == fr.flip_h {
        Transform::from_row(-ss, 0.0, 0.0, ss, wx + w * scale, wy)
    } else {
        Transform::from_row(ss, 0.0, 0.0, ss, wx, wy)
    };
    let mut pp = PixmapPaint::default();
    pp.opacity = alpha;
    canvas.draw_pixmap(0, 0, src.as_ref(), &pp, t, None);
}

fn rect(
    canvas: &mut Pixmap,
    b: [f32; 4],
    pos: (f32, f32),
    facing: f32,
    scale: f32,
    camx: f32,
    rgba: (u8, u8, u8, u8),
) {
    let ground = H as f32 - 36.0;
    let (x1, x2) = if facing >= 0.0 {
        (b[0], b[2])
    } else {
        (-b[2], -b[0])
    };
    let l = (W as f32 / 2.0) + (pos.0 + x1 - camx) * scale;
    let r = (W as f32 / 2.0) + (pos.0 + x2 - camx) * scale;
    let t = ground + (pos.1 + b[1]) * scale;
    let bt = ground + (pos.1 + b[3]) * scale;
    let mut pb = PathBuilder::new();
    pb.move_to(l, t);
    pb.line_to(r, t);
    pb.line_to(r, bt);
    pb.line_to(l, bt);
    pb.close();
    if let Some(p) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(rgba.0, rgba.1, rgba.2, rgba.3));
        canvas.stroke_path(
            &p,
            &paint,
            &tiny_skia::Stroke {
                width: 1.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );
    }
}

pub fn frame(m: &Match, boxes: bool) -> Pixmap {
    let mut c = Pixmap::new(W, H).unwrap();
    c.fill(Color::from_rgba8(26, 30, 48, 255));
    let ground = H as f32 - 36.0;
    let mut pb = PathBuilder::new();
    pb.move_to(0.0, ground);
    pb.line_to(W as f32, ground);
    pb.line_to(W as f32, H as f32);
    pb.line_to(0.0, H as f32);
    pb.close();
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(52, 58, 80, 255));
    c.fill_path(
        &pb.finish().unwrap(),
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    let roots: Vec<_> = m.fighters.iter().filter(|f| !f.is_helper).collect();
    let camx = roots.iter().map(|f| f.pos.x).sum::<f32>() / roots.len().max(1) as f32;
    let scale = 1.0;
    for f in m.fighters.iter() {
        if let Some(ai) = f.after_image.as_ref() {
            for k in ai.params.shown(ai.trail.len(), ai.length) {
                let t = &ai.trail[k];
                let ghost = AnimFrame { group: t.group, index: t.index, x: t.fx, y: t.fy, time: 1, flip_h: t.flip_h, flip_v: false, blend: Blend::None, alpha: None, xscale: 1.0, yscale: 1.0, angle: 0.0, clsn1: vec![], clsn2: vec![] };
                blit(&mut c, &f.anim_def, &ghost, (t.x, t.y), t.facing, scale, camx, (0.55 - 0.05 * k as f32).max(0.12));
            }
        }
        if let Some(fr) = f.frame() {
            blit(
                &mut c,
                &f.anim_def,
                fr,
                (f.pos.x, f.pos.y),
                f.facing,
                scale,
                camx,
                1.0,
            );
        }
    }
    for p in m.projectiles.iter() {
        if let Some(fr) = p.cur.frame() {
            blit(
                &mut c,
                &p.cur.def,
                fr,
                (p.pos.x, p.pos.y),
                p.facing,
                scale * p.scale[0],
                camx,
                1.0,
            );
        }
    }
    for e in m.explods.iter() {
        if let Some(fr) = e.cur.frame() {
            let w = e.world_pos(m.camera);
            let (x, y) = if let Some(b) = e
                .bind_to
                .and_then(|id| m.fighters.iter().find(|f| f.id == id))
            {
                (b.pos.x + e.bind_off.x * b.facing, b.pos.y + e.bind_off.y)
            } else {
                (w.x, w.y)
            };
            blit(
                &mut c,
                &e.cur.def,
                fr,
                (x, y),
                e.facing,
                scale * e.scale[0],
                camx,
                if e.add { 0.8 } else { 1.0 },
            );
        }
    }
    if boxes {
        for f in m.fighters.iter() {
            for b in f.boxes(2) {
                rect(
                    &mut c,
                    [b[0] - f.pos.x * 0.0, b[1], b[2], b[3]],
                    (0.0, 0.0),
                    1.0,
                    scale,
                    camx,
                    (80, 160, 255, 200),
                );
            }
            for b in f.boxes(1) {
                rect(&mut c, b, (0.0, 0.0), 1.0, scale, camx, (255, 70, 70, 255));
            }
        }
    }
    c
}

pub fn sheet(frames: &[Pixmap], cols: usize) -> Pixmap {
    let rows = frames.len().div_ceil(cols).max(1);
    let mut s = Pixmap::new(W * cols as u32, H * rows as u32).unwrap();
    s.fill(Color::BLACK);
    for (i, f) in frames.iter().enumerate() {
        s.draw_pixmap(
            ((i % cols) as u32 * W) as i32,
            ((i / cols) as u32 * H) as i32,
            f.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
    s
}
