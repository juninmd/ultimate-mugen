//! Procedurally generated common effects (replaces fightfx.sff): hit sparks, guard sparks, super flash, dust.
use crate::chardef::{AnimFrame, Animation, Blend, CharDef, ImageData, Sprite};
use crate::raster::*;
use std::f32::consts::PI;
use std::panic::RefUnwindSafe;
use std::sync::Arc;

const S: f32 = 3.0;

fn star(n: usize, r1: f32, r2: f32, rot: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    for i in 0..n * 2 {
        let r = if i % 2 == 1 { r2 } else { r1 };
        let a = rot + i as f32 / (n * 2) as f32 * PI * 2.0;
        if i == 0 {
            pb.move_to(a.cos() * r, a.sin() * r)
        } else {
            pb.line_to(a.cos() * r, a.sin() * r)
        }
    }
    pb.close();
    pb.finish()
}

fn spark(size: f32, t: f32, core: Color, mid: Color, outer: Color, scale: f32) -> Pixmap {
    let px = (size * S) as u32;
    let mut pm = Pixmap::new(px, px).unwrap();
    let tr = Transform::from_translate(px as f32 / 2.0, px as f32 / 2.0).pre_scale(S, S);
    let r = size / 2.0 * scale;
    let rad = r * (0.4 + t * 0.6);
    let fade = 1.0 - t * 0.7;
    let mut c0 = core;
    c0.apply_opacity(fade);
    let mut c1 = mid;
    c1.apply_opacity(fade * 0.8);
    if let Some(sh) = RadialGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(0.0, 0.0),
        rad,
        vec![
            GradientStop::new(0.0, c0),
            GradientStop::new(0.35, c1),
            GradientStop::new(1.0, Color::TRANSPARENT),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    ) {
        let mut p = Paint::default();
        p.shader = sh;
        p.anti_alias = true;
        if let Some(c) = circle(0.0, 0.0, rad) {
            pm.fill_path(&c, &p, FillRule::Winding, tr, None);
        }
    }
    let mut m = mid;
    m.apply_opacity(fade);
    if let Some(sp) = star(8, r * (0.5 + t * 0.5), r * 0.12, t * 0.6) {
        pm.fill_path(&sp, &solid(m), FillRule::Winding, tr, None);
    }
    let mut c = core;
    c.apply_opacity(fade);
    if let Some(sp) = star(4, r * (0.7 + t * 0.4) * (1.0 - t * 0.3), r * 0.06, 0.4 + t) {
        pm.fill_path(&sp, &solid(c), FillRule::Winding, tr, None);
    }
    let stroke = Stroke {
        width: 1.2,
        line_cap: LineCap::Round,
        ..Default::default()
    };
    let mut o = outer;
    o.apply_opacity(fade);
    for i in 0..10 {
        let a = i as f32 * 2.39996;
        let (r0, r1) = (
            r * (0.3 + t * 0.9),
            r * (0.3 + t * 0.9) + r * 0.25 * (1.0 - t),
        );
        let mut pb = PathBuilder::new();
        pb.move_to(a.cos() * r0, a.sin() * r0);
        pb.line_to(a.cos() * r1, a.sin() * r1);
        if let Some(p) = pb.finish() {
            pm.stroke_path(&p, &solid(o), &stroke, tr, None);
        }
    }
    if t > 0.2 {
        let mut m2 = mid;
        m2.apply_opacity((1.0 - t) * 0.8);
        let st = Stroke {
            width: 2.0 * (1.0 - t) + 0.5,
            ..Default::default()
        };
        if let Some(c) = circle(0.0, 0.0, r * t * 1.1) {
            pm.stroke_path(&c, &solid(m2), &st, tr, None);
        }
    }
    pm
}

fn dust(size: f32, t: f32) -> Pixmap {
    let px = (size * S) as u32;
    let mut pm = Pixmap::new(px, px).unwrap();
    let tr = Transform::from_translate(px as f32 / 2.0, px as f32 / 2.0).pre_scale(S, S);
    for i in 0..7 {
        let x = (i as f32 - 3.0) * size * 0.09 * (1.0 + t * 1.5);
        let r = size * (0.08 + 0.08 * (i as f32 * 1.7).sin().powi(2)) * (0.6 + t);
        if let Some(c) = circle(x, -r * 0.6 - t * size * 0.08, r) {
            pm.fill_path(
                &c,
                &solid(rgba("#d8c8a8", 0.55 * (1.0 - t))),
                FillRule::Winding,
                tr,
                None,
            );
        }
    }
    pm
}

fn super_flash(size: f32, t: f32) -> Pixmap {
    let px = (size * S) as u32;
    let mut pm = Pixmap::new(px, px).unwrap();
    let tr = Transform::from_translate(px as f32 / 2.0, px as f32 / 2.0).pre_scale(S, S);
    let r = size / 2.0;
    let a = 1.0 - t * 0.6;
    if let Some(sh) = RadialGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(0.0, 0.0),
        r * (0.3 + t * 0.7),
        vec![
            GradientStop::new(0.0, rgba("#ffffff", 0.95 * a)),
            GradientStop::new(0.3, rgba("#ffdc78", 0.7 * a)),
            GradientStop::new(1.0, Color::TRANSPARENT),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    ) {
        let mut p = Paint::default();
        p.shader = sh;
        p.anti_alias = true;
        if let Some(c) = circle(0.0, 0.0, r) {
            pm.fill_path(&c, &p, FillRule::Winding, tr, None);
        }
    }
    if let Some(sp) = star(12, r * (0.4 + t * 0.6), r * 0.05, t * 2.0) {
        pm.fill_path(
            &sp,
            &solid(rgba("#ffffdc", 0.9 * a)),
            FillRule::Winding,
            tr,
            None,
        );
    }
    pm
}

type Art = Box<dyn Fn(f32, f32) -> Pixmap + Send + Sync + RefUnwindSafe>;

fn add(d: &mut CharDef, with_images: bool, no: i32, size: f32, n: usize, ticks: i32, art: Art) {
    let art = Arc::new(art);
    let mut frames = vec![];
    for i in 0..n {
        let t = i as f32 / (n - 1).max(1) as f32;
        let px = size * S;
        let image = with_images.then(|| {
            let art = art.clone();
            ImageData::lazy(px as u32, px as u32, move || pixmap_to_image(&art(size, t)).rgba().clone())
        });
        d.sprites.insert((no, i as i32), Sprite { image, width: px, height: px, axis_x: px / 2.0, axis_y: px / 2.0, scale: 1.0 / S });
        frames.push(AnimFrame {
            group: no,
            index: i as i32,
            x: 0.0,
            y: 0.0,
            time: ticks,
            flip_h: false,
            flip_v: false,
            blend: Blend::Add,
            alpha: None,
            xscale: 1.0,
            yscale: 1.0,
            angle: 0.0,
            clsn1: vec![],
            clsn2: vec![],
        });
    }
    let mut a = Animation { id: no, frames, loop_start: 0, total_time: 0 };
    a.compute_total();
    d.anims.insert(no, a);
}

pub fn build_fx(with_images: bool) -> Arc<CharDef> {
    let mut d = CharDef::empty("fx");
    let w = |h| rgba(h, 1.0);
    let mut add = |no, size, n, ticks, art: Art| add(&mut d, with_images, no, size, n, ticks, art);
    add(0, 40.0, 5, 2, Box::new(move |s, t| spark(s, t, w("#ffffff"), w("#ffe08a"), w("#ffb347"), 0.8)));
    add(1, 56.0, 6, 2, Box::new(move |s, t| spark(s, t, w("#ffffff"), w("#ffd06a"), w("#ff8c2a"), 0.9)));
    add(2, 84.0, 7, 2, Box::new(move |s, t| spark(s, t, w("#ffffff"), w("#ffb84a"), w("#ff5a1f"), 1.0)));
    add(3, 84.0, 7, 2, Box::new(move |s, t| spark(s, t, w("#ffffff"), w("#b8e0ff"), w("#5aa8ff"), 1.0)));
    add(40, 50.0, 5, 2, Box::new(move |s, t| spark(s, t, w("#ffffff"), w("#8fd3ff"), w("#2d7dff"), 0.85)));
    add(30, 130.0, 8, 3, Box::new(super_flash));
    add(120, 60.0, 6, 3, Box::new(dust));
    for (from, to) in [
        (4, 1),
        (5, 2),
        (6, 0),
        (7, 1),
        (8, 2),
        (9, 2),
        (41, 40),
        (42, 40),
        (43, 40),
    ] {
        let mut a = d.anims[&to].clone();
        a.id = from;
        d.anims.insert(from, a);
    }
    Arc::new(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_art_is_drawn_on_first_read_at_the_declared_size() {
        let fx = build_fx(true);
        assert!(fx.sprites.values().all(|sprite| sprite.image.as_ref().is_some_and(|image| image.is_pending())));
        for (key, sprite) in &fx.sprites {
            let image = sprite.image.as_ref().unwrap();
            assert_eq!(image.rgba().len(), image.w as usize * image.h as usize * 4, "sprite {key:?}");
        }
        for group in fx.sprites.keys().map(|key| key.0) {
            let drawn = fx.sprites.iter().any(|(key, sprite)| key.0 == group && sprite.image.as_ref().is_some_and(|image| image.rgba().iter().any(|byte| *byte != 0)));
            assert!(drawn, "effect {group} draws something");
        }
    }

    #[test]
    fn imageless_effects_have_no_art() {
        assert!(build_fx(false).sprites.values().all(|sprite| sprite.image.is_none()));
    }
}
