use serde_json::Value;
use ume_engine::native::rig::{render_pose, set_art_res};
use std::path::PathBuf;

fn bounds(image: &[u8], width: usize, height: usize) -> Option<(usize, usize, usize, usize)> {
    let mut result = (width, height, 0, 0);
    for y in 0..height {
        for x in 0..width {
            if image[(y * width + x) * 4 + 3] > 16 {
                result.0 = result.0.min(x);
                result.1 = result.1.min(y);
                result.2 = result.2.max(x + 1);
                result.3 = result.3.max(y + 1);
            }
        }
    }
    (result.2 > result.0 && result.3 > result.1).then_some(result)
}

fn pose_size(style: &Value, pose: &Value) -> Result<(f32, f32), String> {
    let render = render_pose(style, pose, None, true, None);
    let image = render.image.ok_or("pose did not produce an image")?;
    let (left, top, right, bottom) = bounds(
        &image.rgba,
        image.w as usize,
        image.h as usize,
    )
    .ok_or("pose image has no opaque body")?;
    Ok(((right - left) as f32, (bottom - top) as f32))
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let character: Value = serde_json::from_slice(
        &std::fs::read(&args[1]).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let defaults_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/native/defaults.json");
    let defaults: Value = serde_json::from_slice(
        &std::fs::read(defaults_path).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let style = &character["art"]["style"];
    let mut poses = defaults["poses"].as_object().cloned().unwrap_or_default();
    if let Some(custom) = character.pointer("/art/poses").and_then(Value::as_object) {
        poses.extend(custom.clone());
    }
    let key = &args[2];
    let pose = poses.get(key).ok_or_else(|| format!("unknown pose {key}"))?;
    set_art_res(3.0);
    let (_, standing_height) = pose_size(
        style,
        poses.get("idle0").or_else(|| poses.get("idle")).unwrap_or(&Value::Null),
    )?;
    let (pose_width, pose_height) = pose_size(style, pose)?;
    let scale = 320.0 / standing_height.max(1.0);
    let fit = (396.0 / (pose_width * scale)).min(391.0 / (pose_height * scale)).min(1.0);
    println!(
        "{{\"width\":{:.1},\"height\":{:.1},\"axisX\":200.0,\"axisY\":392.0,\"refHeight\":320.0}}",
        pose_width * scale * fit,
        pose_height * scale * fit
    );
    Ok(())
}
