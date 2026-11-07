//! Auto-Clsn: Generates proportional M.U.G.E.N Clsn2 (hurtbox) and Clsn1 (hitbox) boxes from sprite pixels.
//! Usage: cargo run --release -p ume-engine --example auto_clsn -- <image_or_dir> [--axis-x <X>] [--axis-y <Y>] [--attack] [--json]
use std::fs;
use std::path::{Path, PathBuf};
use ume_formats::png::{decode_png, DecodedPng};

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox { pub min_x: i32, pub min_y: i32, pub max_x: i32, pub max_y: i32 }

#[derive(Clone, Debug, Default)]
pub struct ClsnResult {
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub bounds: Option<BoundingBox>,
    pub hurtboxes: Vec<[i32; 4]>,
    pub hitboxes: Vec<[i32; 4]>,
}

fn is_opaque(img: &DecodedPng, x: u32, y: u32) -> bool {
    let idx = (y * img.width + x) as usize;
    if let Some(rgba) = &img.rgba {
        let p = idx * 4;
        p + 3 < rgba.len() && rgba[p + 3] > 20
    } else if let Some(indexed) = &img.indexed {
        idx < indexed.len() && indexed[idx] != 0
    } else {
        false
    }
}

pub fn analyze_silhouette(img: &DecodedPng) -> Option<BoundingBox> {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    let mut found = false;
    for y in 0..img.height {
        for x in 0..img.width {
            if is_opaque(img, x, y) {
                found = true;
                let (xi, yi) = (x as i32, y as i32);
                min_x = min_x.min(xi);
                min_y = min_y.min(yi);
                max_x = max_x.max(xi);
                max_y = max_y.max(yi);
            }
        }
    }
    if found { Some(BoundingBox { min_x, min_y, max_x, max_y }) } else { None }
}

pub fn generate_clsn(
    file: &str,
    img: &DecodedPng,
    axis_x: i32,
    axis_y: i32,
    is_attack: bool,
) -> ClsnResult {
    let mut res = ClsnResult { file: file.to_string(), width: img.width, height: img.height, ..Default::default() };
    let Some(bb) = analyze_silhouette(img) else { return res; };
    res.bounds = Some(bb);

    let h = (bb.max_y - bb.min_y + 1).max(1);
    let w = (bb.max_x - bb.min_x + 1).max(1);
    let pad_h = (w as f32 * 0.15) as i32;
    let pad_l = (w as f32 * 0.10) as i32;

    let head_bot = bb.min_y + (h as f32 * 0.22) as i32;
    let torso_bot = bb.min_y + (h as f32 * 0.65) as i32;

    let shift = |x1: i32, y1: i32, x2: i32, y2: i32| [x1 - axis_x, y1 - axis_y, x2 - axis_x, y2 - axis_y];

    // Proportional 3-box anatomy standard: Head, Torso, Legs
    res.hurtboxes.push(shift(bb.min_x + pad_h, bb.min_y, bb.max_x - pad_h, head_bot));
    res.hurtboxes.push(shift(bb.min_x, head_bot - 2, bb.max_x, torso_bot));
    res.hurtboxes.push(shift(bb.min_x + pad_l, torso_bot - 2, bb.max_x - pad_l, bb.max_y));

    if is_attack {
        // Forward attack active box: from center of torso extending forward
        let mid_y = bb.min_y + (h as f32 * 0.25) as i32;
        let reach_x = bb.max_x + (w as f32 * 0.35) as i32;
        res.hitboxes.push(shift(bb.min_x + (w as f32 * 0.4) as i32, mid_y, reach_x, torso_bot));
    }
    res
}

fn print_air(res: &ClsnResult) {
    println!("; Auto-Clsn generated for {}", res.file);
    if let Some(b) = res.bounds {
        println!("; Silhouette bounds: [{}, {}] to [{}, {}] ({}x{})", b.min_x, b.min_y, b.max_x, b.max_y, b.max_x - b.min_x + 1, b.max_y - b.min_y + 1);
    }
    if !res.hitboxes.is_empty() {
        println!("Clsn1: {}", res.hitboxes.len());
        for (i, b) in res.hitboxes.iter().enumerate() {
            println!("  Clsn1[{i}] = {}, {}, {}, {}", b[0], b[1], b[2], b[3]);
        }
    }
    if !res.hurtboxes.is_empty() {
        println!("Clsn2: {}", res.hurtboxes.len());
        for (i, b) in res.hurtboxes.iter().enumerate() {
            println!("  Clsn2[{i}] = {}, {}, {}, {}", b[0], b[1], b[2], b[3]);
        }
    }
    println!();
}

fn collect_pngs(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")) { vec![path.to_path_buf()] } else { vec![] }
    } else if let Ok(entries) = fs::read_dir(path) {
        let mut v: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png"))).collect();
        v.sort();
        v
    } else {
        vec![]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: auto_clsn <image.png | dir> [--axis-x <X>] [--axis-y <Y>] [--attack] [--json]");
        return Ok(());
    }
    let target = &args[1];
    let mut axis_x = 0;
    let mut axis_y = 0;
    let mut is_attack = false;
    let mut json_mode = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--axis-x" => { if i + 1 < args.len() { axis_x = args[i + 1].parse().unwrap_or(0); i += 1; } }
            "--axis-y" => { if i + 1 < args.len() { axis_y = args[i + 1].parse().unwrap_or(0); i += 1; } }
            "--attack" | "--clsn1" => is_attack = true,
            "--json" => json_mode = true,
            _ => {}
        }
        i += 1;
    }

    let files = collect_pngs(Path::new(target));
    if files.is_empty() {
        eprintln!("No PNG files found at: {target}");
        return Ok(());
    }

    for p in &files {
        let buf = match fs::read(p) { Ok(b) => b, Err(e) => { eprintln!("Error reading {}: {e}", p.display()); continue; } };
        let img = match decode_png(&buf) { Ok(im) => im, Err(e) => { eprintln!("Error decoding PNG {}: {e}", p.display()); continue; } };
        let res = generate_clsn(&p.display().to_string(), &img, axis_x, axis_y, is_attack);

        if json_mode {
            let json = serde_json::json!({
                "file": res.file,
                "hurt": res.hurtboxes,
                "hit": if res.hitboxes.is_empty() { serde_json::Value::Null } else { serde_json::json!({ "box": res.hitboxes[0] }) }
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            print_air(&res);
        }
    }
    Ok(())
}
