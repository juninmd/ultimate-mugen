use serde_json::{Map, Value};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use tiny_skia::{Pixmap, PremultipliedColorU8};
use ume_formats::png::decode_png;

const ALPHA_THRESHOLD: u8 = 16;
const MIN_COMPONENT_PIXELS: usize = 24;
const MAX_ATLAS_BYTES: u64 = 67_108_864;
const MAX_ATLAS_DIMENSION: u32 = 16_384;
const MAX_ATLAS_PIXELS: u64 = 8_000_000;
const MAX_CANVAS: u32 = 2_048;

struct Options {
    atlas: PathBuf,
    out: PathBuf,
    names: Vec<String>,
    reference_height: Option<f64>,
    canvas: u32,
    validate_only: bool,
    overwrite: bool,
}

struct Bounds {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("atlas import failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!("Usage: cargo run -p ume-engine --example import_sprite_atlas -- --atlas FILE --out DIR --cols 3 --rows 2 --names idle,walk,... [--canvas 400] [--reference-height PX] [--validate-only] [--overwrite]");
        return Ok(());
    }
    let options = parse_options()?;
    let file_size = fs::metadata(&options.atlas).map_err(|error| format!("cannot inspect {}: {error}", options.atlas.display()))?.len();
    if file_size > MAX_ATLAS_BYTES { return Err(format!("atlas file exceeds the {MAX_ATLAS_BYTES}-byte limit")); }
    let bytes = fs::read(&options.atlas).map_err(|error| format!("cannot read {}: {error}", options.atlas.display()))?;
    if bytes.len() < 24 || bytes.get(12..16) != Some(b"IHDR") { return Err("PNG is missing a valid IHDR header".into()); }
    let header_width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let header_height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if header_width > MAX_ATLAS_DIMENSION || header_height > MAX_ATLAS_DIMENSION || header_width as u64 * header_height as u64 > MAX_ATLAS_PIXELS {
        return Err(format!("atlas dimensions exceed limits of {MAX_ATLAS_DIMENSION} pixels per side and {MAX_ATLAS_PIXELS} total pixels"));
    }
    let decoded = decode_png(&bytes).map_err(|error| format!("invalid PNG: {error:?}"))?;
    if !matches!(decoded.color_type, 4 | 6) { return Err("atlas PNG must contain an alpha channel; RGB-only input is not accepted".into()); }
    let rgba = decoded.rgba.ok_or_else(|| "atlas PNG is missing decoded alpha data".to_string())?;
    let (width, height) = (decoded.width, decoded.height);
    if width < 3 || height < 2 || width % 3 != 0 || height % 2 != 0 {
        return Err(format!("atlas dimensions {width}x{height} must divide evenly into a 3x2 grid"));
    }
    let cell_width = width / 3;
    let cell_height = height / 2;
    let mut bounds = Vec::with_capacity(6);
    for index in 0..6 {
        let col = index % 3;
        let row = index / 3;
        let x0 = col as u32 * cell_width;
        let y0 = row as u32 * cell_height;
        let bound = alpha_bounds(&rgba, width, x0, y0, cell_width, cell_height)?;
        validate_cell(&rgba, width, x0, y0, cell_width, cell_height, &options.names[index], &bound)?;
        bounds.push(bound);
    }
    let source_reference = options.reference_height.unwrap_or(cell_height as f64 * 0.8);
    let common_height = options.canvas as f64 * 0.8;
    if !source_reference.is_finite() || source_reference <= 0.0 {
        return Err("reference height must be a finite positive pixel count".into());
    }
    let meta_path = options.out.join("meta.json");
    let mut meta = if meta_path.exists() {
        let bytes = fs::read(&meta_path).map_err(|error| format!("cannot read {}: {error}", meta_path.display()))?;
        serde_json::from_slice::<Value>(&bytes).map_err(|error| format!("invalid {}: {error}", meta_path.display()))?
    } else {
        Value::Object(Map::new())
    };
    if !meta.is_object() { return Err("meta.json root must be a JSON object".into()); }
    if meta.get("frames").is_some_and(|frames| !frames.is_object()) { return Err("meta.json frames must be a JSON object".into()); }
    for name in &options.names {
        let target = options.out.join(format!("{name}.png"));
        if target.exists() && !options.overwrite {
            return Err(format!("{} exists; pass --overwrite to replace it", target.display()));
        }
    }
    let mut encoded = Vec::with_capacity(6);
    for (index, name) in options.names.iter().enumerate() {
        let x0 = (index % 3) as u32 * cell_width;
        let y0 = (index / 3) as u32 * cell_height;
        let raw = crop(&rgba, width, x0, y0, cell_width, cell_height);
        let normalized = normalize_pose(cell_width, cell_height, options.canvas, source_reference, &bounds[index], &raw)?;
        let pixmap = pixmap_from_straight_rgba(options.canvas, options.canvas, &normalized)?;
        let png = pixmap.encode_png().map_err(|error| format!("PNG encode failed: {error}"))?;
        encoded.push((name, png));
    }
    if options.validate_only {
        println!("valid 3x2 RGBA atlas: {}x{}, six poses fit the {}px canvas; shared refHeight={common_height}", width, height, options.canvas);
        return Ok(());
    }
    let meta_object = meta.as_object_mut().unwrap();
    let frames = meta_object.entry("frames").or_insert_with(|| Value::Object(Map::new()));
    for name in &options.names {
        frames.as_object_mut().unwrap().insert(name.clone(), serde_json::json!({"refHeight": options.canvas as f64 * 0.8, "axisX": options.canvas as f64 / 2.0, "axisY": options.canvas as f64 - 8.0}));
    }
    fs::create_dir_all(&options.out).map_err(|error| format!("cannot create {}: {error}", options.out.display()))?;
    for (name, png) in encoded {
        fs::write(options.out.join(format!("{name}.png")), png).map_err(|error| format!("cannot write {name}.png: {error}"))?;
    }
    let serialized = serde_json::to_vec_pretty(&meta).map_err(|error| format!("cannot serialize metadata: {error}"))?;
    fs::write(meta_path, serialized).map_err(|error| format!("cannot write metadata: {error}"))?;
    println!("imported six transparent frames to {}", options.out.display());
    Ok(())
}

fn validate_cell(rgba: &[u8], image_width: u32, x0: u32, y0: u32, width: u32, height: u32, name: &str, bounds: &Bounds) -> Result<(), String> {
    let margin_x = (width / 100).max(2);
    let margin_y = (height / 100).max(2);
    let touches = bounds.left <= margin_x || bounds.top <= margin_y || bounds.right + margin_x >= width || bounds.bottom + margin_y >= height;
    if touches { return Err(format!("pose {name} alpha reaches the cell border; likely clipped or crossing grid; inspect the source atlas")); }
    let mut crossing = false;
    for y in 0..height {
        let first = alpha_at(rgba, image_width, x0, y0 + y) >= ALPHA_THRESHOLD;
        let last = alpha_at(rgba, image_width, x0 + width - 1, y0 + y) >= ALPHA_THRESHOLD;
        crossing |= first || last;
    }
    for x in 0..width {
        let first = alpha_at(rgba, image_width, x0 + x, y0) >= ALPHA_THRESHOLD;
        let last = alpha_at(rgba, image_width, x0 + x, y0 + height - 1) >= ALPHA_THRESHOLD;
        crossing |= first || last;
    }
    if crossing { return Err(format!("pose {name} appears to cross its grid cell; inspect adjoining cells")); }
    let mut visited = vec![false; (width * height) as usize];
    let mut components = Vec::new();
    for y in 0..height { for x in 0..width {
        let local = (y * width + x) as usize;
        if visited[local] || alpha_at(rgba, image_width, x0 + x, y0 + y) < ALPHA_THRESHOLD { continue; }
        let mut queue = VecDeque::from([(x, y)]);
        visited[local] = true;
        let mut component = 0;
        while let Some((cx, cy)) = queue.pop_front() {
            component += 1;
            for (nx, ny) in [(cx.wrapping_sub(1), cy), (cx + 1, cy), (cx, cy.wrapping_sub(1)), (cx, cy + 1)] {
                if nx >= width || ny >= height { continue; }
                let next = (ny * width + nx) as usize;
                if !visited[next] && alpha_at(rgba, image_width, x0 + nx, y0 + ny) >= ALPHA_THRESHOLD {
                    visited[next] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
        components.push(component);
    }}
    let body = components.iter().copied().max().unwrap_or(0);
    if body < MIN_COMPONENT_PIXELS { return Err(format!("pose {name} has no substantial alpha component at threshold {ALPHA_THRESHOLD}")); }
    let noise = components.iter().filter(|size| **size < MIN_COMPONENT_PIXELS).count();
    if noise > 24 { return Err(format!("pose {name} has {noise} isolated alpha-noise components; clean the sheet before import")); }
    let _ = (image_width, x0, y0);
    Ok(())
}

fn alpha_bounds(rgba: &[u8], image_width: u32, x0: u32, y0: u32, width: u32, height: u32) -> Result<Bounds, String> {
    let mut left = width; let mut top = height; let mut right = 0; let mut bottom = 0; let mut found = false;
    for y in 0..height { for x in 0..width {
        if alpha_at(rgba, image_width, x0 + x, y0 + y) >= ALPHA_THRESHOLD {
            found = true; left = left.min(x); top = top.min(y); right = right.max(x); bottom = bottom.max(y);
        }
    }}
    if !found { return Err("atlas contains an empty pose cell or lacks meaningful alpha; verify the transparent source".into()); }
    Ok(Bounds { left, top, right, bottom })
}

fn alpha_at(rgba: &[u8], width: u32, x: u32, y: u32) -> u8 { rgba[((y * width + x) * 4 + 3) as usize] }

fn crop(rgba: &[u8], source_width: u32, x0: u32, y0: u32, width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0; (width * height * 4) as usize];
    for y in 0..height {
        let source = ((y0 + y) * source_width + x0) as usize * 4;
        let target = (y * width) as usize * 4;
        out[target..target + width as usize * 4].copy_from_slice(&rgba[source..source + width as usize * 4]);
    }
    out
}

fn pixmap_from_straight_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Pixmap, String> {
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| "cell dimensions exceed tiny-skia limits".to_string())?;
    for (source, target) in rgba.chunks_exact(4).zip(pixmap.pixels_mut()) {
        let alpha = source[3] as u16;
        let premultiply = |channel: u8| ((channel as u16 * alpha + 127) / 255) as u8;
        *target = PremultipliedColorU8::from_rgba(premultiply(source[0]), premultiply(source[1]), premultiply(source[2]), source[3]).ok_or_else(|| "invalid premultiplied RGBA pixel".to_string())?;
    }
    Ok(pixmap)
}

fn normalize_pose(cell_width: u32, cell_height: u32, canvas: u32, source_reference: f64, bounds: &Bounds, source: &[u8]) -> Result<Vec<u8>, String> {
    let body_height = bounds.bottom - bounds.top + 1;
    let scale = canvas as f64 * 0.8 / source_reference;
    if !scale.is_finite() || scale <= 0.0 {
        return Err(format!("pose at alpha height {body_height} does not fit the fixed {canvas}px canvas with standing reference {source_reference}; set --reference-height to the standing-pose alpha height"));
    }
    let source_center_x = cell_width as f64 / 2.0;
    let baseline = canvas - 8;
    let target_body_bottom = baseline as f64;
    let transformed_left = ((bounds.left as f64 - source_center_x) * scale + canvas as f64 / 2.0).floor();
    let transformed_right = ((bounds.right as f64 + 1.0 - source_center_x) * scale + canvas as f64 / 2.0).ceil();
    let transformed_top = target_body_bottom - (bounds.bottom as f64 + 1.0 - bounds.top as f64) * scale;
    if transformed_left < 0.0 || transformed_right >= canvas as f64 || transformed_top < 0.0 {
        return Err(format!("opaque pose bounds transform outside the {canvas}px canvas; reduce --reference-height or increase --canvas"));
    }
    let mut output = vec![0; (canvas * canvas * 4) as usize];
    for y in 0..canvas { for x in 0..canvas {
        let sx = ((x as f64 - canvas as f64 / 2.0) / scale + source_center_x).floor() as i64;
        let sy = (bounds.bottom as f64 + 1.0 - (target_body_bottom - y as f64) / scale).floor() as i64;
        if sx < 0 || sy < 0 || sx >= cell_width as i64 || sy >= cell_height as i64 { continue; }
        let (sx, sy) = (sx as u32, sy as u32);
        let source_index = (sy * cell_width + sx) as usize * 4;
        let target_index = (y * canvas + x) as usize * 4;
        if source[source_index + 3] >= ALPHA_THRESHOLD {
            output[target_index..target_index + 4].copy_from_slice(&source[source_index..source_index + 4]);
        }
    }}
    Ok(output)
}

fn parse_options() -> Result<Options, String> {
    let mut args = std::env::args().skip(1);
    let mut atlas = None; let mut out = None; let mut names = None; let mut reference_height = None; let mut canvas = 400; let mut validate_only = false; let mut overwrite = false;
    while let Some(arg) = args.next() {
        let value = |args: &mut std::iter::Skip<std::env::Args>, key: &str| args.next().ok_or_else(|| format!("{key} requires a value"));
        match arg.as_str() {
            "--atlas" => atlas = Some(PathBuf::from(value(&mut args, "--atlas")?)),
            "--out" => out = Some(PathBuf::from(value(&mut args, "--out")?)),
            "--names" => names = Some(value(&mut args, "--names")?.split(',').map(str::trim).map(str::to_string).collect::<Vec<_>>()),
            "--reference-height" => reference_height = Some(value(&mut args, "--reference-height")?.parse::<f64>().map_err(|_| "reference height must be numeric".to_string())?),
            "--canvas" => canvas = value(&mut args, "--canvas")?.parse::<u32>().map_err(|_| "canvas must be an integer".to_string())?,
            "--cols" if value(&mut args, "--cols")? == "3" => {},
            "--rows" if value(&mut args, "--rows")? == "2" => {},
            "--validate-only" => validate_only = true,
            "--overwrite" => overwrite = true,
            "--help" | "-h" => return Err("Usage: cargo run -p ume-engine --example import_sprite_atlas -- --atlas FILE --out DIR --cols 3 --rows 2 --names idle,walk,... [--canvas 400] [--reference-height PX] [--validate-only] [--overwrite]".into()),
            "--cols" | "--rows" => return Err(format!("unsupported grid value for {arg}")),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    let names = names.ok_or_else(|| "--names is required with six comma-separated names".to_string())?;
    if names.len() != 6 || names.iter().any(|name| !valid_name(name)) { return Err("--names must contain six safe filename stems".into()); }
    let mut unique = names.clone(); unique.sort(); unique.dedup();
    if unique.len() != names.len() { return Err("pose names must be unique".into()); }
    if !(64..=MAX_CANVAS).contains(&canvas) { return Err(format!("canvas must be between 64 and {MAX_CANVAS} pixels")); }
    Ok(Options { atlas: atlas.ok_or_else(|| "--atlas is required".to_string())?, out: out.ok_or_else(|| "--out is required".to_string())?, names, reference_height, canvas, validate_only, overwrite })
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && name.chars().all(|char| char.is_ascii_alphanumeric() || char == '_' || char == '-') && Path::new(name).file_name().and_then(|value| value.to_str()) == Some(name)
}
