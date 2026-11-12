//! Renders every native animation pose that lacks a dedicated PNG into the fighter's sprite folder.
//!
//! `cargo run --release -p ume-engine --example generate_missing_sprites -- [--dry-run]`
//!
//! Targets (mirroring `native_sprite_audit --require-complete`):
//! - named poses that would resolve to the idle canvas (`pose_stem` == `01_idle`),
//! - `df*` defeat poses, which otherwise render through the rig at runtime (`procedural-body`),
//! - inline pose objects, whose frame key is `inline_<hash>`.
//!
//! Art comes from the merged `defaults.json` + character pose table, so each fighter keeps its own
//! style. Output canvas and metadata follow the established convention: 400x400 canvas, axis
//! (200, 392), `refHeight` 320. Resolution classification lives in `pose_map.rs`.
mod pose_map;
mod pixelize;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde_json::Value;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};
use ume_engine::native::rig::{render_pose, set_art_res};

use pose_map::{frame_targets, to_pixmap};

const CANVAS: u32 = 400;
const AXIS_X: f32 = 200.0;
const AXIS_Y: f32 = 392.0;
const REF_HEIGHT: f32 = 320.0;
const MAX_SOURCE_BYTES: u64 = 8_000_000;
const MAX_TARGETS: usize = 4096;

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
    let dry_run = std::env::args().any(|arg| arg == "--dry-run");
    let attack_only = std::env::args().any(|arg| arg == "--attack-keyframes");
    let rebuild_list = std::env::args()
        .find_map(|arg| arg.strip_prefix("--rebuild-list=").map(str::to_string))
        .map(std::fs::read_to_string)
        .transpose()?
        .unwrap_or_default();
    let chars_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars");
    let sprites_dir = chars_dir.join("sprites");
    let defaults_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/native/defaults.json");
    let defaults: Value = serde_json::from_slice(&std::fs::read(&defaults_path)?)?;
    let default_poses = defaults.get("poses").cloned().unwrap_or(Value::Null);
    let default_anims = defaults.get("anims").cloned().unwrap_or(Value::Null);

    let mut char_paths: Vec<PathBuf> = Vec::new();
    let mut roots: Vec<PathBuf> = vec![chars_dir.clone(), chars_dir.join("../umf"), chars_dir.join("../mugen/umf")];
    if let Some(dir) = std::env::var_os("UME_UMF_DIR") {
        roots.push(PathBuf::from(dir));
    }
    for root in &roots {
        let Ok(entries) = std::fs::read_dir(root) else { continue };
        char_paths.extend(entries.filter_map(Result::ok).map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                && path.file_stem().is_some_and(|stem| stem != "index")));
    }
    char_paths.sort();
    char_paths.dedup();
    if char_paths.is_empty() {
        return Ok(());
    }

    for char_path in &char_paths {
        if std::fs::metadata(char_path).map(|meta| meta.len() > MAX_SOURCE_BYTES).unwrap_or(true) {
            eprintln!("[{}] skipped (source JSON larger than {} bytes)", char_path.display(), MAX_SOURCE_BYTES);
            continue;
        }
        let char_json: Value = match std::fs::read(char_path).map_err(|error| error.to_string()).and_then(|bytes| serde_json::from_slice(&bytes).map_err(|error| error.to_string())) {
            Ok(json) => json,
            Err(error) => {
                eprintln!("[{}] skipped ({error})", char_path.display());
                continue;
            }
        };
        let Some(id) = char_json.get("id").and_then(Value::as_str).map(str::to_string) else { continue };
        if !pose_map::safe_name(&id) {
            eprintln!("[{id}] skipped (id is not a plain identifier)");
            continue;
        }
        if char_json.pointer("/art/type").and_then(Value::as_str) != Some("rig") {
            println!("[{id}] non-rig character, skipped");
            continue;
        }
        let style = char_json.pointer("/art/style").cloned().unwrap_or(Value::Null);
        let mut poses: Value = default_poses.clone();
        if let (Some(target), Some(source)) = (poses.as_object_mut(), char_json.pointer("/art/poses").and_then(Value::as_object)) {
            for (key, value) in source {
                target.insert(key.clone(), value.clone());
            }
        }

        let char_sprites_dir = sprites_dir.join(&id);
        std::fs::create_dir_all(&char_sprites_dir)?;
        let meta_path = char_sprites_dir.join("meta.json");
        let mut meta: Value = if meta_path.is_file() {
            match serde_json::from_slice(&std::fs::read(&meta_path)?) {
                Ok(meta) => meta,
                Err(error) => {
                    eprintln!("[{id}] skipped (meta.json unreadable: {error})");
                    continue;
                }
            }
        } else {
            serde_json::json!({"frames": {}})
        };
        if meta.get("frames").is_none() {
            meta["frames"] = serde_json::json!({});
        }

        let mut files: BTreeSet<String> = char_sprites_dir
            .read_dir()?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                name.strip_suffix(".png").map(str::to_string)
            })
            .collect();
        for name in rebuild_list.lines().filter_map(|entry| {
            entry.strip_prefix(&format!("{id}/")).and_then(|name| name.strip_suffix(".png"))
        }) {
            files.remove(name);
        }

        let mut targets: BTreeMap<String, Value> = BTreeMap::new();
        let attack_anims = pose_map::attack_animations(&char_json);
        for anims_value in [Some(&default_anims), char_json.get("anims")] {
            let Some(anims) = anims_value.and_then(Value::as_object) else { continue };
            for (anim_name, anim) in anims {
                if attack_only && !attack_anims.contains(anim_name) { continue; }
                let Some(frames) = anim.get("frames").and_then(Value::as_array) else { continue };
                for frame in frames {
                    if let Some(target) = frame_targets(frame, &files, &poses, attack_only) {
                        targets.insert(target.0, target.1);
                    }
                }
            }
        }

        if targets.is_empty() {
            println!("[{id}] every pose has dedicated art");
            continue;
        }

        if targets.len() > MAX_TARGETS {
            eprintln!("[{id}] skipped ({} frames requested, limit {MAX_TARGETS})", targets.len());
            continue;
        }

        println!("[{id}] generating {} missing frames...", targets.len());
        if dry_run {
            for name in targets.keys() {
                println!("  would render {name}");
            }
            continue;
        }

        let idle_pose = poses.get("idle0").or_else(|| poses.get("idle")).unwrap_or(&Value::Null);
        let ref_render = render_pose(&style, idle_pose, None, true, None);
        let mut standing_h = 320.0f32;
        if let Some(img) = ref_render.image.as_deref().and_then(to_pixmap) {
            if let Some((_, _, min_y, max_y)) = alpha_bounds(&img) {
                standing_h = (max_y - min_y + 1.0).max(100.0);
            }
        }
        let base_scale = REF_HEIGHT / standing_h;

        for (name, pose_val) in &targets {
            if !pose_map::safe_name(name) {
                eprintln!("  unsafe pose name {name} skipped for {id}");
                continue;
            }
            let mut pose_val = pose_val.clone();
            if attack_only {
                if let Some(pose) = pose_val.as_object_mut() {
                    pose.remove("aura");
                    pose.remove("glow");
                }
            }
            let render = render_pose(&style, &pose_val, None, true, None);
            let Some(src) = render.image.as_deref().and_then(to_pixmap) else {
                eprintln!("  failed to render {name} for {id}");
                continue;
            };
            let Some((min_x, max_x, min_y, max_y)) = alpha_bounds(&src) else {
                eprintln!("  empty alpha for {name} on {id}");
                continue;
            };

            let mut scale = base_scale;
            let margin = 4.0;
            let content_w = (max_x - min_x).max(1.0) * base_scale;
            let content_h = (max_y - min_y).max(1.0) * base_scale;
            let limit_w = CANVAS as f32 - 2.0 * margin;
            let limit_h = AXIS_Y - margin;
            let fit = (limit_w / content_w).min(limit_h / content_h);
            if fit < 1.0 {
                scale *= fit;
            }

            let mut canvas = Pixmap::new(CANVAS, CANVAS).ok_or("cannot alloc canvas")?;
            let center_x = (min_x + max_x) / 2.0;
            let dx = AXIS_X - center_x * scale;
            let dy = AXIS_Y - 1.0 - max_y * scale;
            let paint = PixmapPaint { quality: FilterQuality::Nearest, ..Default::default() };
            let transform = Transform::from_row(scale, 0.0, 0.0, scale, dx, dy);
            canvas.draw_pixmap(0, 0, src.as_ref(), &paint, transform, None);

            let out_png = char_sprites_dir.join(format!("{name}.png"));
            pixelize::apply(&canvas).ok_or("cannot quantize attack frame")?.save_png(&out_png)?;
            meta["frames"][name.as_str()] = serde_json::json!({"axisX": AXIS_X, "axisY": AXIS_Y, "refHeight": REF_HEIGHT});
        }

        let staged_meta = char_sprites_dir.join("meta.json.tmp");
        std::fs::write(&staged_meta, serde_json::to_vec_pretty(&meta)?)?;
        std::fs::rename(&staged_meta, &meta_path)?;
        println!("[{id}] updated meta.json");
    }

    println!("sprite generation pass complete");
    Ok(())
}
