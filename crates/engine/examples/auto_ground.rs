//! Auto-Ground: Automatic floor anchoring (Y=0) and horizontal axis centering from sprite pixels.
//! Usage: cargo run --release -p ume-engine --example auto_ground -- <image_or_dir> [--json] [--box-center]
use std::fs;
use std::path::{Path, PathBuf};
use ume_formats::png::{decode_png, DecodedPng};

#[derive(Clone, Debug, Default)]
pub struct GroundResult {
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub center_x: i32,
    pub ground_y: i32,
    pub opaque_pixels: usize,
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

pub fn analyze_ground(file: &str, img: &DecodedPng, use_box_center: bool) -> Option<GroundResult> {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    let mut sum_x: i64 = 0;
    let mut count: usize = 0;

    for y in 0..img.height {
        for x in 0..img.width {
            if is_opaque(img, x, y) {
                let (xi, yi) = (x as i32, y as i32);
                min_x = min_x.min(xi);
                min_y = min_y.min(yi);
                max_x = max_x.max(xi);
                max_y = max_y.max(yi);
                sum_x += xi as i64;
                count += 1;
            }
        }
    }
    if count == 0 { return None; }

    let center_x = if use_box_center {
        (min_x + max_x) / 2
    } else {
        (sum_x / count as i64) as i32
    };

    Some(GroundResult {
        file: file.to_string(),
        width: img.width,
        height: img.height,
        min_x,
        min_y,
        max_x,
        max_y,
        center_x,
        ground_y: max_y,
        opaque_pixels: count,
    })
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
        eprintln!("Usage: auto_ground <image.png | dir> [--json] [--box-center]");
        return Ok(());
    }
    let target = &args[1];
    let mut json_mode = false;
    let mut box_center = false;

    for a in &args[2..] {
        match a.as_str() {
            "--json" => json_mode = true,
            "--box-center" => box_center = true,
            _ => {}
        }
    }

    let files = collect_pngs(Path::new(target));
    if files.is_empty() {
        eprintln!("No PNG files found at: {target}");
        return Ok(());
    }

    let mut results = vec![];
    for p in &files {
        let buf = match fs::read(p) { Ok(b) => b, Err(e) => { eprintln!("Read err {}: {e}", p.display()); continue; } };
        let img = match decode_png(&buf) { Ok(im) => im, Err(e) => { eprintln!("Decode err {}: {e}", p.display()); continue; } };
        if let Some(res) = analyze_ground(&p.display().to_string(), &img, box_center) {
            results.push(res);
        } else {
            eprintln!("Warning: {} is completely transparent", p.display());
        }
    }

    if json_mode {
        let json_items: Vec<_> = results.iter().map(|r| serde_json::json!({
            "file": r.file,
            "size": [r.width, r.height],
            "bounds": [r.min_x, r.min_y, r.max_x, r.max_y],
            "axis_x": r.center_x,
            "axis_y": r.ground_y,
            "opaque_pixels": r.opaque_pixels
        })).collect();
        println!("{}", serde_json::to_string_pretty(&json_items)?);
    } else {
        println!("{:<45} {:>9} {:>17} {:>14} {:>10}", "Sprite File", "Dim", "BBox [X, Y]", "Axis (X, Y)", "Opaque");
        println!("{}", "-".repeat(100));
        for r in &results {
            let dim = format!("{}x{}", r.width, r.height);
            let bbox = format!("[{}..{}, {}..{}]", r.min_x, r.max_x, r.min_y, r.max_y);
            let axis = format!("{}, {}", r.center_x, r.ground_y);
            println!("{:<45} {:>9} {:>17} {:>14} {:>10}", r.file, dim, bbox, axis, r.opaque_pixels);
        }
        if !results.is_empty() {
            println!("{}", "-".repeat(100));
            println!("Recommended global idle floor axis_y: {}", results.iter().map(|r| r.ground_y).max().unwrap_or(0));
        }
    }
    Ok(())
}
