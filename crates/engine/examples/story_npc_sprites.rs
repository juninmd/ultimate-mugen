use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};
use ume_engine::native::{
    native_frame_sprite_key,
    rig::{render_pose, set_art_res},
    NativeChar,
};

fn pixmap(image: &ume_engine::chardef::ImageData) -> Pixmap {
    let mut pixels = Pixmap::new(image.w, image.h).unwrap();
    for (dest, source) in pixels
        .data_mut()
        .chunks_exact_mut(4)
        .zip(image.rgba.chunks_exact(4))
    {
        for i in 0..3 {
            dest[i] = ((u16::from(source[i]) * u16::from(source[3]) + 127) / 255) as u8;
        }
        dest[3] = source[3];
    }
    pixels
}

fn pose_variation(pose: &mut Value, name: &str) {
    if name.ends_with("Recoil") {
        for (key, amount) in [
            ("head", -12.0),
            ("lean", 16.0),
            ("shoulderF", -22.0),
            ("elbowB", 18.0),
        ] {
            pose[key] = json!(pose[key].as_f64().unwrap_or(0.0) + amount);
        }
    }
    for field in ["aura", "flash", "fx"] {
        if let Some(values) = pose.as_object_mut() {
            values.remove(field);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_art_res(1.5);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars");
    for id in [
        "npc_brute",
        "npc_colossus",
        "npc_phantom",
        "npc_razor",
        "npc_spitfire",
    ] {
        let character: NativeChar =
            serde_json::from_slice(&fs::read(root.join(format!("{id}.json")))?)?;
        let character = character.with_sprite_animation_defaults();
        let style = &character.art.style;
        let mut poses = character.art.poses.clone();
        if !poses.contains_key("runMid") {
            let mut mid = poses["run1"].clone();
            for (key, value) in poses["run2"].as_object().unwrap() {
                if let (Some(a), Some(b)) = (mid[key].as_f64(), value.as_f64()) {
                    mid[key] = json!((a + b) / 2.0);
                }
            }
            poses.insert("runMid".to_string(), mid);
        }
        let mut frames = BTreeMap::new();
        for (_, animation) in character.anims.iter() {
            for frame in &animation.frames {
                if frame.effect.is_some() {
                    continue;
                }
                let key = native_frame_sprite_key(frame);
                let mut pose = match frame.pose.as_ref() {
                    Some(Value::String(name)) => poses
                        .get(name)
                        .unwrap_or_else(|| panic!("{id} missing pose {name}"))
                        .clone(),
                    Some(pose) => pose.clone(),
                    None => panic!("{id} missing pose for {key}"),
                };
                pose_variation(&mut pose, &key);
                frames.insert(key, pose);
            }
        }

        let folder = root.join("sprites").join(id);
        fs::create_dir_all(&folder)?;
        let idle = render_pose(style, &poses["idle0"], None, true, None);
        let idle_img = idle.image.as_ref().unwrap();
        let top = idle_img
            .rgba
            .chunks_exact(4)
            .position(|p| p[3] >= 128)
            .unwrap()
            / idle_img.w as usize;
        let nominal_scale = 320.0 / (idle.axis_y - top as f32);
        let mut geometry = BTreeMap::new();
        for (key, pose) in &frames {
            assert!(key.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'));
            let frame = render_pose(style, pose, None, true, None);
            let source = pixmap(frame.image.as_ref().unwrap());
            let scale = nominal_scale
                .min(392.0 / frame.width)
                .min(392.0 / frame.height);
            let min_axis_x = 4.0 + frame.axis_x * scale;
            let max_axis_x = 396.0 - (frame.width - frame.axis_x) * scale;
            let min_axis_y = 4.0 + frame.axis_y * scale;
            let max_axis_y = 396.0 - (frame.height - frame.axis_y) * scale;
            let axis_x = 200.0_f32.clamp(min_axis_x.min(max_axis_x), max_axis_x);
            let axis_y = 392.0_f32.clamp(min_axis_y.min(max_axis_y), max_axis_y);
            let mut canvas = Pixmap::new(400, 400).unwrap();
            let transform = Transform::from_row(
                scale,
                0.0,
                0.0,
                scale,
                axis_x - frame.axis_x * scale,
                axis_y - frame.axis_y * scale,
            );
            let paint = PixmapPaint {
                quality: FilterQuality::Nearest,
                ..Default::default()
            };
            canvas.draw_pixmap(0, 0, source.as_ref(), &paint, transform, None);
            canvas.save_png(folder.join(format!("{key}.png")))?;
            geometry.insert(
                key,
                json!({"axisX":axis_x,"axisY":axis_y,"refHeight":320.0 * scale / nominal_scale}),
            );
        }
        set_art_res(6.0);
        let portrait = render_pose(style, &poses["idle0"], None, true, None);
        let source = pixmap(portrait.image.as_ref().unwrap());
        let scale = 460.0 / source.width() as f32;
        let mut canvas = Pixmap::new(512, 512).unwrap();
        let transform = Transform::from_row(scale, 0.0, 0.0, scale, 26.0, 28.0);
        canvas.draw_pixmap(
            0,
            0,
            source.as_ref(),
            &PixmapPaint::default(),
            transform,
            None,
        );
        canvas.save_png(folder.join("portrait.png"))?;
        set_art_res(1.5);
        fs::write(
            folder.join("meta.json"),
            serde_json::to_vec_pretty(&json!({ "frames": geometry, "refHeight": 320 }))?,
        )?;
        println!("{id}: {} distinct sprite keys", frames.len());
    }
    Ok(())
}
