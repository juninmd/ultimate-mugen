use serde_json::Value;
use std::path::PathBuf;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};
use ume_engine::chardef::ImageData;
use ume_engine::native::rig::{render_pose, set_art_res};

fn to_pixmap(img: &ImageData) -> Option<Pixmap> {
    let mut pm = Pixmap::new(img.w, img.h)?;
    for (d, s) in pm.data_mut().chunks_exact_mut(4).zip(img.rgba.chunks_exact(4)) {
        let a = s[3] as u32;
        for i in 0..3 {
            d[i] = ((s[i] as u32 * a + 127) / 255) as u8;
        }
        d[3] = s[3];
    }
    Some(pm)
}

fn alpha_bounds(pm: &Pixmap) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = pm.width() as i32;
    let mut max_x = -1;
    let mut min_y = pm.height() as i32;
    let mut max_y = -1;
    for y in 0..pm.height() as i32 {
        for x in 0..pm.width() as i32 {
            let p = pm.pixel(x as u32, y as u32)?;
            if p.alpha() > 16 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }
    if max_x >= min_x && max_y >= min_y {
        Some((min_x as f32, max_x as f32, min_y as f32, max_y as f32))
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_art_res(3.0);
    let chars_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars");
    let sprites_dir = chars_dir.join("sprites");
    let defaults_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/native/defaults.json");
    let defaults: Value = serde_json::from_slice(&std::fs::read(&defaults_path)?)?;
    let default_poses = defaults.get("poses").cloned().unwrap_or(Value::Null);

    let char_ids = [
        "akio", "bruno", "daichi", "garan", "haruki", "jin", "kaito", "kaname",
        "lian", "marlow", "oboro", "shion", "suzume", "sylene", "valdric", "volta",
    ];

    for id in &char_ids {
        let char_path = chars_dir.join(format!("{id}.json"));
        if !char_path.is_file() {
            continue;
        }
        let char_json: Value = serde_json::from_slice(&std::fs::read(&char_path)?)?;
        let style = &char_json["art"]["style"];
        let poses = &char_json["art"]["poses"];

        let char_sprites_dir = sprites_dir.join(id);
        std::fs::create_dir_all(&char_sprites_dir)?;

        let meta_path = char_sprites_dir.join("meta.json");
        let mut meta: Value = if meta_path.is_file() {
            serde_json::from_slice(&std::fs::read(&meta_path)?)?
        } else {
            serde_json::json!({"frames": {}})
        };
        if meta.get("frames").is_none() {
            meta["frames"] = serde_json::json!({});
        }

        // Determine reference standing height for scaling
        let idle_pose = poses.get("idle0").or_else(|| poses.get("idle")).unwrap_or(&Value::Null);
        let ref_render = render_pose(style, idle_pose, None, true, None);
        let mut standing_h = 320.0f32;
        if let Some(img) = ref_render.image.as_deref().and_then(to_pixmap) {
            if let Some((_, _, min_y, max_y)) = alpha_bounds(&img) {
                standing_h = (max_y - min_y + 1.0).max(100.0);
            }
        }

        // Targets to verify and generate if missing
        let mut targets: Vec<(String, Value)> = Vec::new();

        // Locomotion targets: walk0..5 and run0..3
        for i in 0..6 {
            let name = format!("walk{i}");
            if !char_sprites_dir.join(format!("{name}.png")).is_file() {
                if let Some(p) = poses.get(&name) {
                    targets.push((name, p.clone()));
                }
            }
        }
        for i in 0..4 {
            let name = format!("run{i}");
            if !char_sprites_dir.join(format!("{name}.png")).is_file() {
                if let Some(p) = poses.get(&name) {
                    targets.push((name, p.clone()));
                }
            }
        }

        if targets.is_empty() {
            println!("[{id}] all required locomotion and state frames present");
            continue;
        }

        println!("[{id}] generating {} missing frames...", targets.len());

        for (name, pose_val) in targets {
            let render = render_pose(style, &pose_val, None, true, None);
            let Some(src) = render.image.as_deref().and_then(to_pixmap) else {
                eprintln!("  failed to render {name} for {id}");
                continue;
            };

            let Some((min_x, max_x, _min_y, max_y)) = alpha_bounds(&src) else {
                eprintln!("  empty alpha for {name} on {id}");
                continue;
            };

            let target_canvas = 400u32;
            let mut canvas = Pixmap::new(target_canvas, target_canvas).ok_or("cannot alloc canvas")?;

            let scale = 320.0 / standing_h;
            let center_x = (min_x + max_x) / 2.0;
            let dx = 200.0 - (center_x * scale);
            let dy = 391.0 - (max_y * scale);

            let paint = PixmapPaint {
                quality: FilterQuality::Nearest,
                ..Default::default()
            };
            let transform = Transform::from_row(scale, 0.0, 0.0, scale, dx, dy);
            canvas.draw_pixmap(0, 0, src.as_ref(), &paint, transform, None);

            let out_png = char_sprites_dir.join(format!("{name}.png"));
            canvas.save_png(&out_png)?;

            meta["frames"][&name] = serde_json::json!({
                "axisX": 200.0,
                "axisY": 392.0,
                "refHeight": 320.0
            });
            println!("  saved {out_png:?}");
        }

        std::fs::write(&meta_path, serde_json::to_vec_pretty(&meta)?)?;
        println!("[{id}] updated meta.json");
    }

    println!("All missing frames processed successfully.");
    Ok(())
}
