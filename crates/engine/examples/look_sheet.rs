//! Visual check of the native roster: renders chosen animation frames of every character side by side.
//!
//! `LOOK_RES=6 LOOK_ZOOM=1 LOOK_ANIMS=idle,win cargo run --release -p ume-engine --example look_sheet -- <out_dir> [id...]`
//! Writes `<out_dir>/<id>.png` (one row per character) and `<out_dir>/all.png` (every character, first anim only).
//! `LOOK_CROP=head|torso` zooms the per-character sheets onto that region of the first frame.
use serde_json::Value;
use tiny_skia::{
    Color, FilterQuality, GradientStop, LinearGradient, Paint, Pixmap, PixmapPaint, Point, Rect,
    SpreadMode, Transform,
};
use ume_engine::chardef::ImageData;
use ume_engine::native::rig::{render_pose, set_art_res, RigRender};

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

fn backdrop(w: u32, h: u32) -> Pixmap {
    let mut pm = Pixmap::new(w.max(1), h.max(1)).unwrap();
    let bg = Paint {
        shader: LinearGradient::new(
            Point::from_xy(0.0, 0.0),
            Point::from_xy(0.0, h as f32),
            vec![
                GradientStop::new(0.0, Color::from_rgba8(0x5a, 0x6a, 0x86, 255)),
                GradientStop::new(1.0, Color::from_rgba8(0x2e, 0x34, 0x48, 255)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap(),
        ..Default::default()
    };
    pm.fill_rect(
        Rect::from_xywh(0.0, 0.0, w as f32, h as f32).unwrap(),
        &bg,
        Transform::identity(),
        None,
    );
    pm
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().unwrap_or_else(|| "look".into());
    std::fs::create_dir_all(&out).unwrap();
    let env_f = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(d)
    };
    let res = env_f("LOOK_RES", 6.0);
    let zoom = env_f("LOOK_ZOOM", 1.0);
    let crop = std::env::var("LOOK_CROP").unwrap_or_default();
    set_art_res(res);
    let anims: Vec<String> = std::env::var("LOOK_ANIMS")
        .unwrap_or_else(|_| "idle,win".into())
        .split(',')
        .map(str::to_string)
        .collect();
    let dir = std::env::var("LOOK_DIR")
        .unwrap_or_else(|_| format!("{}/../../assets/chars", env!("CARGO_MANIFEST_DIR")));
    let mut ids: Vec<String> = args[2.min(args.len())..].to_vec();
    if ids.is_empty() {
        ids = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
            })
            .filter(|s| s != "index")
            .collect();
        ids.sort();
    }
    let t0 = std::time::Instant::now();
    let mut all: Vec<(String, RigRender)> = vec![];
    let k = zoom;
    for id in &ids {
        let ch: Value =
            serde_json::from_str(&std::fs::read_to_string(format!("{dir}/{id}.json")).unwrap())
                .unwrap();
        let style = &ch["art"]["style"];
        let mut cells = vec![];
        for an in &anims {
            let Some(f) = ch["anims"][an.as_str()]["frames"]
                .as_array()
                .and_then(|a| a.first())
            else {
                continue;
            };
            let pose = match &f["pose"] {
                Value::String(s) => ch["art"]["poses"][s.as_str()].clone(),
                v => v.clone(),
            };
            cells.push(render_pose(style, &pose, None, true, None));
        }
        if cells.is_empty() {
            continue;
        }
        // optional crop of the first frame: (top fraction, bottom fraction) of its height
        let (f0, f1) = match crop.as_str() {
            "head" => (0.0, 0.3),
            "torso" => (0.2, 0.62),
            _ => (0.0, 1.0),
        };
        let cw: Vec<f32> = cells
            .iter()
            .enumerate()
            .map(|(_i, r)| r.width * k + 20.0)
            .map(|w| if crop.is_empty() || w == 0.0 { w } else { w })
            .collect();
        let hmax = cells.iter().map(|r| r.height * k).fold(0.0, f32::max);
        let h = if crop.is_empty() {
            hmax + 20.0
        } else {
            hmax * (f1 - f0) + 20.0
        };
        let w: f32 = if crop.is_empty() {
            cw.iter().sum()
        } else {
            cw[0]
        };
        let mut pm = backdrop(w as u32, h as u32);
        let mut x = 0.0;
        for (i, (r, cwi)) in cells.iter().zip(&cw).enumerate() {
            if !crop.is_empty() && i > 0 {
                break;
            }
            if let Some(src) = r.image.as_ref().and_then(|i| to_pixmap(i)) {
                let y = if crop.is_empty() {
                    h - 10.0 - r.height * k
                } else {
                    10.0 - r.height * k * f0
                };
                pm.draw_pixmap(
                    0,
                    0,
                    src.as_ref(),
                    &PixmapPaint {
                        quality: FilterQuality::Bilinear,
                        ..Default::default()
                    },
                    Transform::from_row(k, 0.0, 0.0, k, x + 10.0, y),
                    None,
                );
            }
            x += cwi;
        }
        pm.save_png(format!("{out}/{id}.png")).unwrap();
        if let Some(r) = cells.into_iter().next() {
            all.push((id.clone(), r));
        }
    }
    println!("rendered {} chars in {:?}", ids.len(), t0.elapsed());
    // lineup of every first frame, shared ground line
    let w: f32 = all.iter().map(|(_, r)| r.width * k + 6.0).sum::<f32>() + 10.0;
    let h = all.iter().map(|(_, r)| r.height * k).fold(0.0, f32::max) + 20.0;
    let mut pm = backdrop(w as u32, h as u32);
    let mut x = 5.0;
    for (_, r) in &all {
        if let Some(src) = r.image.as_ref().and_then(|i| to_pixmap(i)) {
            let y = h - 10.0 - (r.height - r.axis_y) * k - r.axis_y * k;
            pm.draw_pixmap(
                0,
                0,
                src.as_ref(),
                &PixmapPaint {
                    quality: FilterQuality::Bilinear,
                    ..Default::default()
                },
                Transform::from_row(k, 0.0, 0.0, k, x, y),
                None,
            );
        }
        x += r.width * k + 6.0;
    }
    pm.save_png(format!("{out}/all.png")).unwrap();
}
