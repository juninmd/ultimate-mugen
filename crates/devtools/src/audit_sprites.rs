use serde_json::{json, Value};
use std::path::Path;
use ume_formats::png::{decode_png, png_indexed_to_rgba};

fn alpha_facts(path: &Path) -> Result<Value, String> {
    let png = decode_png(&std::fs::read(path).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", path.display()))?;
    let rgba = png.rgba.clone().unwrap_or_else(|| png_indexed_to_rgba(&png));
    let (width, height) = (png.width as usize, png.height as usize);
    let (mut left, mut top, mut right, mut bottom, mut any_transparent) = (width, height, 0, 0, false);
    for y in 0..height {
        for x in 0..width {
            if rgba[(y * width + x) * 4 + 3] == 0 {
                any_transparent = true;
            } else {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
    }
    let bounds = if right > left && bottom > top { json!([left, top, right, bottom]) } else { Value::Null };
    Ok(json!({ "size": [width, height], "alphaBounds": bounds, "transparentPixels": any_transparent }))
}

fn sorted_files(dir: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(dir).into_iter().flatten().filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == extension)).collect();
    files.sort();
    files
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn inspect(root: &Path) -> Result<Value, String> {
    let mut fighters = Vec::new();
    for source in sorted_files(&root.join("assets/chars"), "json") {
        let character = read_json(&source)?;
        let Some(id) = character.get("id").and_then(Value::as_str) else { continue };
        let folder = root.join("assets/chars/sprites").join(id);
        let meta = if folder.join("meta.json").exists() { read_json(&folder.join("meta.json"))? } else { json!({}) };
        let mut images = serde_json::Map::new();
        for path in sorted_files(&folder, "png") {
            images.insert(path.file_name().unwrap_or_default().to_string_lossy().into_owned(), alpha_facts(&path)?);
        }
        let mut frames = Vec::new();
        if let Some(anims) = character.get("anims").and_then(Value::as_object) {
            for (name, animation) in anims {
                for (index, frame) in animation.get("frames").and_then(Value::as_array).into_iter().flatten().enumerate() {
                    let pose = frame.get("sprite").filter(|v| !v.is_null()).or_else(|| frame.get("pose"));
                    let exact = pose.and_then(Value::as_str).is_some_and(|p| folder.join(format!("{p}.png")).is_file());
                    frames.push(json!({ "animation": name, "frame": index, "pose": pose, "effect": frame.get("effect").is_some_and(|e| !e.is_null()), "ticks": frame.get("ticks"), "exactFile": exact }));
                }
            }
        }
        fighters.push(json!({
            "id": id, "declaredHeight": character.pointer("/stats/height"), "referenceHeight": meta.get("refHeight"),
            "avatar": folder.join("portrait.png").is_file(), "images": images, "frames": frames,
        }));
    }
    Ok(json!({ "scope": "Exact-file inventory; semantic aliases and procedural effects must be reviewed separately. A missing exact file alone does not prove missing runtime art.", "fighters": fighters }))
}

pub fn summary_lines(report: &Value) -> Vec<String> {
    report["fighters"].as_array().into_iter().flatten().map(|f| {
        let body: Vec<&Value> = f["frames"].as_array().into_iter().flatten().filter(|frame| frame["effect"] == false).collect();
        let exact = body.iter().filter(|frame| frame["exactFile"] == true).count();
        format!("{} height {} avatar {} body frames {} exact files {}", f["id"].as_str().unwrap_or(""), f["declaredHeight"], f["avatar"], body.len(), exact)
    }).collect()
}
