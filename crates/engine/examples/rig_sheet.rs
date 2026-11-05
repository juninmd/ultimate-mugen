//! Contact sheet of a native (UMF) character's rig frames with hurt (blue) / hit (red) boxes.
//!
//! `cargo run --release -p ume-engine --example rig_sheet -- <id> <out.png> [palette-index]`
//!
//! Layout mirrors the TypeScript reference harness: key animations first, then every move's
//! animation, projectiles and reactions; frames with an identical pose/effect/hit are shown once.
use serde_json::Value;
use std::collections::HashSet;
use std::time::Instant;
use tiny_skia::{
    BlendMode, Color, FilterQuality, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap,
    PixmapPaint, Point, Rect, SpreadMode, Stroke, Transform,
};
use ume_engine::chardef::ImageData;
use ume_engine::native::rig::{render_effect, render_pose, RigRender};
use ume_engine::native::ActiveSpec;

fn to_pixmap(img: &ImageData) -> Option<Pixmap> {
    let mut pm = Pixmap::new(img.w, img.h)?;
    for (d, s) in pm
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(img.rgba.chunks_exact(4))
    {
        let a = s[3] as u32;
        for i in 0..3 {
            d[i] = ((s[i] as u32 * a + 127) / 255) as u8;
        }
        d[3] = s[3];
    }
    Some(pm)
}

fn stroke_rect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, c: Color, width: f32) {
    if let Some(r) = Rect::from_xywh(x, y, w.max(0.5), h.max(0.5)) {
        let path = PathBuilder::from_rect(r);
        let mut p = Paint::default();
        p.set_color(c);
        p.anti_alias = true;
        pm.stroke_path(
            &path,
            &p,
            &Stroke {
                width,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // optional overrides (same meaning as the TS harness query params): RIG_SCALE, RIG_W, RIG_ANIMS=a,b,c, RIG_BOXES=0
    let env_f = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(d)
    };
    let (sc, sheet_w) = (env_f("RIG_SCALE", 1.6), env_f("RIG_W", 1800.0));
    let show_boxes = std::env::var("RIG_BOXES").map(|v| v != "0").unwrap_or(true);
    let key_anims: Option<Vec<String>> = std::env::var("RIG_ANIMS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect());
    let id = args.get(1).map(String::as_str).unwrap_or("kaito");
    let out = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| format!("sheet-{id}.png"));
    let path = format!(
        "{}/../../assets/chars/{id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let ch: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("character json"))
        .expect("json");
    let palette = args
        .get(3)
        .and_then(|i| i.parse::<usize>().ok())
        .and_then(|i| ch["palettes"].get(i).cloned());
    let style = &ch["art"]["style"];
    let poses = &ch["art"]["poses"];
    let anims = &ch["anims"];
    let moves = ch["moves"].as_array().cloned().unwrap_or_default();
    let projs = |m: &Value| -> Vec<Value> {
        match &m["projectile"] {
            Value::Array(a) => a.clone(),
            Value::Null => vec![],
            v => vec![v.clone()],
        }
    };

    let mut defaults: Vec<String> = [
        "idle",
        "walk",
        "crouch",
        "jumpUp",
        "jumpFwd",
        "jumpDown",
        "run",
        "hop",
        "guard",
        "crouchGuard",
        "airGuard",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    defaults.extend(
        moves
            .iter()
            .filter_map(|m| m["anim"].as_str().map(str::to_string)),
    );
    defaults.extend(
        moves
            .iter()
            .flat_map(projs)
            .filter_map(|p| p["anim"].as_str().map(str::to_string)),
    );
    defaults.extend(
        [
            "hitLight",
            "hitHeavy",
            "hitLowHeavy",
            "hitCrouch",
            "hitAir",
            "fall",
            "bounce",
            "lieDown",
            "getUp",
            "trip",
            "win",
            "lose",
            "intro",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
    let list = key_anims.clone().unwrap_or(defaults);

    let t0 = Instant::now();
    let mut seen = HashSet::new();
    let mut cells: Vec<(bool, RigRender)> = vec![];
    for name in &list {
        let Some(a) = anims.get(name) else {
            cells.push((
                false,
                render_pose(
                    style,
                    &Value::Object(Default::default()),
                    None,
                    true,
                    palette.as_ref(),
                ),
            ));
            continue;
        };
        for f in a["frames"].as_array().into_iter().flatten() {
            let key = serde_json::to_string(&(&f["pose"], &f["effect"], &f["hit"])).unwrap();
            if !seen.insert(key) && key_anims.is_none() {
                continue;
            }
            let r = if !f["effect"].is_null() {
                (true, render_effect(&f["effect"], true, palette.as_ref()))
            } else {
                let pose = match &f["pose"] {
                    Value::String(s) => poses.get(s).cloned().unwrap_or(Value::Null),
                    v => v.clone(),
                };
                let hit: Option<ActiveSpec> = if f["hit"].is_null() {
                    None
                } else {
                    serde_json::from_value(f["hit"].clone()).ok()
                };
                (
                    false,
                    render_pose(style, &pose, hit.as_ref(), true, palette.as_ref()),
                )
            };
            cells.push(r);
        }
    }
    let render_time = t0.elapsed();

    // layout
    let (mut x, mut y, mut row_h) = (10.0f32, 30.0f32, 0.0f32);
    let mut pos = vec![];
    for (_, r) in &cells {
        let w = (r.width * r.scale * sc).max(70.0) + 12.0;
        let h = (r.height * r.scale * sc).max(150.0 * sc / 1.6) + 26.0;
        if x + w > sheet_w {
            x = 10.0;
            y += row_h;
            row_h = 0.0;
        }
        pos.push((x, y));
        x += w;
        row_h = row_h.max(h);
    }
    let sheet_h = (y + row_h + 10.0).ceil() as u32;
    let mut pm = Pixmap::new(sheet_w as u32, sheet_h).unwrap();
    let bg = Paint {
        shader: LinearGradient::new(
            Point::from_xy(0.0, 0.0),
            Point::from_xy(0.0, sheet_h as f32),
            vec![
                GradientStop::new(0.0, Color::from_rgba8(0x3a, 0x4a, 0x66, 255)),
                GradientStop::new(1.0, Color::from_rgba8(0x1e, 0x24, 0x36, 255)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap(),
        ..Default::default()
    };
    pm.fill_rect(
        Rect::from_xywh(0.0, 0.0, sheet_w, sheet_h as f32).unwrap(),
        &bg,
        Transform::identity(),
        None,
    );

    for ((is_fx, r), &(px, py)) in cells.iter().zip(&pos) {
        let k = r.scale * sc;
        let w = (r.width * k).max(70.0);
        let h = (r.height * k).max(150.0 * sc / 1.6);
        let ox = px + r.axis_x * k;
        let gy = if *is_fx {
            py + r.axis_y * k
        } else {
            py + h - (r.height - r.axis_y) * k
        };
        // ground line
        let mut pb = PathBuilder::new();
        pb.move_to(px, gy);
        pb.line_to(px + w, gy);
        let mut lp = Paint::default();
        lp.set_color(Color::from_rgba(1.0, 1.0, 1.0, 0.18).unwrap());
        if let Some(p) = pb.finish() {
            pm.stroke_path(&p, &lp, &Stroke::default(), Transform::identity(), None);
        }
        let (dx, dy) = (ox - r.axis_x * k, gy - r.axis_y * k);
        if let Some(src) = r.image.as_ref().and_then(|i| to_pixmap(i)) {
            let paint = PixmapPaint {
                opacity: 1.0,
                blend_mode: if *is_fx {
                    BlendMode::Plus
                } else {
                    BlendMode::SourceOver
                },
                quality: FilterQuality::Bilinear,
            };
            pm.draw_pixmap(
                0,
                0,
                src.as_ref(),
                &paint,
                Transform::from_row(k, 0.0, 0.0, k, dx, dy),
                None,
            );
        }
        if !show_boxes {
            continue;
        }
        for b in &r.hurt {
            stroke_rect(
                &mut pm,
                ox + b[0] * sc,
                gy + b[1] * sc,
                (b[2] - b[0]) * sc,
                (b[3] - b[1]) * sc,
                Color::from_rgba(80.0 / 255.0, 160.0 / 255.0, 1.0, 0.9).unwrap(),
                1.0,
            );
        }
        for b in &r.hit {
            stroke_rect(
                &mut pm,
                ox + b[0] * sc,
                gy + b[1] * sc,
                (b[2] - b[0]) * sc,
                (b[3] - b[1]) * sc,
                Color::from_rgba8(255, 60, 60, 255),
                1.0,
            );
        }
        let mut yp = Paint::default();
        yp.set_color(Color::from_rgba8(255, 255, 0, 255));
        pm.fill_rect(
            Rect::from_xywh(ox - 2.0, gy - 2.0, 4.0, 4.0).unwrap(),
            &yp,
            Transform::identity(),
            None,
        );
    }
    pm.save_png(&out).expect("write png");
    println!(
        "{id}: {} cells rendered in {:?} -> {out}",
        cells.len(),
        render_time
    );
}
