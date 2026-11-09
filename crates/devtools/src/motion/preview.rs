use super::frames::describe;
use crate::json_order::{parse, to_ascii_string, to_string, Json, Style};
use std::path::{Path, PathBuf};
use std::process::Command;
use ume_formats::png::decode_png;

fn temp_descriptor(character: &Path, sprites: &Path, animation: &str) -> Result<(PathBuf, Json), String> {
    let dir = std::env::temp_dir().join(format!("ume-motion-descriptor-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("asset.json");
    let _ = std::fs::remove_file(&path);
    let (_, descriptor) = describe(character, sprites, animation, &path)?;
    Ok((dir, descriptor))
}

fn frame_ticks(descriptor: &Json) -> Vec<i64> {
    descriptor.get("frames").and_then(Json::as_array).map(|f| f.iter().map(|x| (x.get("duration").and_then(Json::as_f64).unwrap_or(0.0) * 60.0).round_ties_even() as i64).collect()).unwrap_or_default()
}

fn frame_file(dir: &Path, descriptor: &Json, index: usize) -> PathBuf {
    let name = descriptor.get("frames").and_then(Json::as_array).and_then(|f| f[index].get("file")).and_then(Json::as_str).unwrap_or("");
    dir.join(name)
}

fn run(executable: &Path, args: &[&str]) -> Result<(), String> {
    let status = Command::new(executable).args(args).status().map_err(|e| format!("{}: {e}", executable.display()))?;
    status.success().then_some(()).ok_or_else(|| format!("{} exited with {status}", executable.display()))
}

pub fn preview_motion(character: &Path, sprites: &Path, animation: &str, destination: &Path, executable: &Path, looped: bool) -> Result<PathBuf, String> {
    if destination.exists() {
        return Err(format!("output already exists: {}", destination.display()));
    }
    let mut chars = animation.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic()) || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("animation must start with a letter and contain only letters, numbers or underscores".into());
    }
    if !executable.is_file() {
        return Err(format!("sprite-gen executable not found: {}", executable.display()));
    }
    let (dir, descriptor) = temp_descriptor(character, sprites, animation)?;
    let ticks = frame_ticks(&descriptor);
    if ticks.iter().any(|t| *t != ticks[0]) || ticks[0] <= 0 || 60 % ticks[0] != 0 || (ticks[0] * 100) % 60 != 0 {
        return Err("GIF preview requires equal durations representable exactly in fps and centiseconds".into());
    }
    let pngs = destination.join("pngs");
    std::fs::create_dir_all(&pngs).map_err(|e| e.to_string())?;
    for index in 0..ticks.len() {
        std::fs::copy(frame_file(&dir, &descriptor, index), pngs.join(format!("frame-{index:04}.png"))).map_err(|e| e.to_string())?;
    }
    let _ = std::fs::remove_dir_all(&dir);
    let run_dir = destination.join("run");
    run(executable, &["unpack-atlas", "--pngs-dir", &pngs.to_string_lossy(), "--state-name", animation, "--out-dir", &run_dir.to_string_lossy()])?;
    let request_path = run_dir.join("sprite-request.json");
    let mut request = parse(&std::fs::read_to_string(&request_path).map_err(|e| e.to_string())?)?;
    let state = request.get_mut("states").and_then(|s| s.get_mut(animation)).ok_or("sprite-request.json has no state for the animation")?;
    state.set("fps", Json::int(60 / ticks[0]));
    state.set("loop", Json::Bool(looped));
    std::fs::write(&request_path, to_string(&request, Style::Indent(2)) + "\n").map_err(|e| e.to_string())?;
    run(executable, &["preview", "--run-dir", &run_dir.to_string_lossy()])?;
    Ok(run_dir.join("qa").join(format!("{animation}.gif")))
}

pub fn preview_scene(character: &Path, sprites: &Path, animation: &str, destination: &Path, executable: &Path, cycles: i64) -> Result<PathBuf, String> {
    if destination.exists() {
        return Err(format!("output already exists: {}", destination.display()));
    }
    if !executable.is_file() {
        return Err(format!("sprite-gen executable not found: {}", executable.display()));
    }
    if !(1..=10).contains(&cycles) {
        return Err("cycles must be between 1 and 10".into());
    }
    let (dir, descriptor) = temp_descriptor(character, sprites, animation)?;
    let ticks = frame_ticks(&descriptor);
    let first = frame_file(&dir, &descriptor, 0);
    let png = decode_png(&std::fs::read(&first).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&dir);
    let (width, height) = (png.width as i64, png.height as i64);
    if width % 2 != 0 || height % 2 != 0 {
        return Err("MP4 output requires an even canvas width and height".into());
    }
    let total: i64 = ticks.iter().sum::<i64>() * cycles;
    if total > 10000 {
        return Err("requested scene exceeds the sprite-gen 10000-frame limit".into());
    }
    std::fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    let (_, descriptor) = describe(character, sprites, animation, &destination.join("asset.json"))?;
    let scene = Json::Obj(vec![
        ("kind".into(), Json::str("sprite-gen-scene")),
        ("version".into(), Json::int(1)),
        ("canvas".into(), Json::Obj(vec![("width".into(), Json::int(width)), ("height".into(), Json::int(height)), ("background".into(), Json::Arr([36, 40, 48, 255].map(Json::int).to_vec()))])),
        ("fps".into(), Json::int(60)),
        ("duration".into(), Json::float(total as f64 / 60.0)),
        ("assets".into(), Json::Obj(vec![("actor".into(), Json::Obj(vec![("source".into(), Json::str("asset.json"))]))])),
        ("layers".into(), Json::Arr(vec![Json::Obj(vec![("id".into(), Json::str("actor")), ("asset".into(), Json::str("actor")), ("at".into(), descriptor.get("anchor").cloned().unwrap_or(Json::Null)), ("loop".into(), Json::Bool(cycles > 1))])])),
    ]);
    let spec = destination.join("scene.json");
    std::fs::write(&spec, to_ascii_string(&scene, Style::Indent(2)) + "\n").map_err(|e| e.to_string())?;
    run(executable, &["scene-render", "--spec", &spec.to_string_lossy(), "--out-dir", &destination.join("render").to_string_lossy(), "--formats", "mp4"])?;
    Ok(destination.join("render").join("scene.mp4"))
}
