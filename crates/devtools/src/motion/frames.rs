use super::relative::relative_path;
use crate::json_order::{parse, to_ascii_string, Json, Style};
use std::path::{Path, PathBuf};
use ume_formats::png::decode_png;

pub struct MotionFrame {
    pub path: PathBuf,
    pub ticks: i64,
    pub width: usize,
    pub height: usize,
    pub axis_x: Json,
    pub axis_y: Json,
    pub reference: Json,
    pub offset: [f64; 2],
}

fn finite(value: &Json) -> Option<f64> {
    value.as_f64().filter(|v| v.is_finite())
}

pub fn effective_ticks(ticks: Option<&Json>, speed: Option<&Json>) -> Result<i64, String> {
    let ticks = ticks.filter(|t| t.is_integer()).and_then(Json::as_f64).filter(|t| *t > 0.0 && *t <= 2147483647.0).ok_or("frame must have a positive i32 tick duration")? as i64;
    let Some(speed) = speed.filter(|s| !matches!(s, Json::Null)) else { return Ok(ticks) };
    let speed = finite(speed).ok_or("character animation speed must be finite")? as f32;
    if !speed.is_finite() {
        return Err("character animation speed must fit f32".into());
    }
    if speed > 1.0 {
        let value = ((ticks as f32) as f64 / speed as f64) as f32;
        return Ok(((value as f64 + 0.5).floor() as i64).clamp(1, 2147483647));
    }
    Ok(ticks)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn read_json(path: &Path) -> Result<Json, String> {
    parse(&std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?)
}

pub fn load_frames(character: &Path, sprites: &Path, animation: &str) -> Result<(Json, Vec<MotionFrame>), String> {
    let data = read_json(character)?;
    let meta_path = sprites.join("meta.json");
    let metadata = if meta_path.exists() { read_json(&meta_path)? } else { Json::Obj(Vec::new()) };
    let frames = data.get("anims").and_then(|a| a.get(animation)).and_then(|a| a.get("frames")).and_then(Json::as_array).filter(|f| !f.is_empty()).ok_or("animation must contain frames")?;
    let mut out = Vec::new();
    for (index, frame) in frames.iter().enumerate() {
        if !matches!(frame, Json::Obj(_)) || frame.get("effect").is_some_and(|e| !matches!(e, Json::Null)) {
            return Err(format!("frame {index} is not a body sprite"));
        }
        let offset = match frame.get("offset") {
            None | Some(Json::Null) => [0.0, 0.0],
            Some(Json::Arr(v)) if v.len() == 2 && v.iter().all(|x| finite(x).is_some()) => [finite(&v[0]).unwrap_or(0.0), finite(&v[1]).unwrap_or(0.0)],
            _ => return Err(format!("frame {index} has an invalid offset")),
        };
        let name = frame.get("sprite").and_then(Json::as_str).filter(|s| !s.is_empty()).or_else(|| frame.get("pose").and_then(Json::as_str)).filter(|n| valid_name(n))
            .ok_or_else(|| format!("frame {index} needs an explicit named sprite or pose"))?;
        let ticks = effective_ticks(frame.get("ticks"), data.get("speed"))?;
        let path = sprites.join(format!("{name}.png"));
        if !path.is_file() {
            return Err(format!("exact sprite missing: {}; engine aliases are not inferred", path.display()));
        }
        let png = decode_png(&std::fs::read(&path).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", path.display()))?;
        let (width, height) = (png.width as usize, png.height as usize);
        let geometry = metadata.get("frames").and_then(|f| f.get(name)).cloned().unwrap_or(Json::Obj(Vec::new()));
        if !matches!(geometry, Json::Obj(_)) {
            return Err(format!("invalid geometry for {name}"));
        }
        let axis_x = geometry.get("axisX").cloned().unwrap_or_else(|| Json::float(width as f64 / 2.0));
        let axis_y = geometry.get("axisY").cloned().unwrap_or_else(|| Json::int(height as i64));
        let reference = geometry.get("refHeight").or_else(|| metadata.get("refHeight")).cloned().unwrap_or_else(|| Json::int(height as i64));
        let numbers = [&axis_x, &axis_y, &reference].map(|v| finite(v));
        let [Some(ax), Some(ay), Some(rh)] = numbers else { return Err(format!("non-finite geometry for {name}")) };
        if !(0.0..=width as f64).contains(&ax) || !(0.0..=height as f64).contains(&ay) || rh <= 0.0 {
            return Err(format!("invalid geometry for {name}"));
        }
        out.push(MotionFrame { path: path.canonicalize().map_err(|e| e.to_string())?, ticks, width, height, axis_x, axis_y, reference, offset });
    }
    Ok((data, out))
}

fn same_canvas(a: &MotionFrame, b: &MotionFrame) -> bool {
    let n = |j: &Json| j.as_f64();
    a.width == b.width && a.height == b.height && n(&a.axis_x) == n(&b.axis_x) && n(&a.axis_y) == n(&b.axis_y) && n(&a.reference) == n(&b.reference)
}

pub fn describe(character: &Path, sprites: &Path, animation: &str, destination: &Path) -> Result<(usize, Json), String> {
    let (_, frames) = load_frames(character, sprites, animation)?;
    let base = std::env::current_dir().map_err(|e| e.to_string())?;
    let absolute = if destination.is_absolute() { destination.to_path_buf() } else { base.join(destination) };
    let directory = absolute.parent().map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf())).unwrap_or_default();
    let mut entries = Vec::new();
    for frame in &frames {
        if frame.offset != [0.0, 0.0] {
            return Err("frame has an offset; a single-anchor descriptor would misrepresent runtime geometry".into());
        }
        if !same_canvas(&frames[0], frame) {
            return Err("frames use different canvas sizes, axes or reference heights; a single-anchor descriptor would misrepresent runtime geometry".into());
        }
        let file = relative_path(&frame.path, &directory).to_string_lossy().into_owned();
        entries.push(Json::Obj(vec![("file".into(), Json::Str(file)), ("duration".into(), Json::float(frame.ticks as f64 / 60.0))]));
    }
    let descriptor = Json::Obj(vec![
        ("kind".into(), Json::str("sprite-gen-asset")),
        ("version".into(), Json::int(1)),
        ("anchor".into(), Json::Arr(vec![frames[0].axis_x.clone(), frames[0].axis_y.clone()])),
        ("frames".into(), Json::Arr(entries)),
    ]);
    let text = to_ascii_string(&descriptor, Style::Indent(2)) + "\n";
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(destination).map_err(|e| format!("{}: {e}", destination.display()))?;
    std::io::Write::write_all(&mut file, text.as_bytes()).map_err(|e| e.to_string())?;
    Ok((entries_len(&descriptor), descriptor))
}

fn entries_len(descriptor: &Json) -> usize {
    descriptor.get("frames").and_then(Json::as_array).map(Vec::len).unwrap_or(0)
}
