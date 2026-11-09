use super::boxes::{anatomical_boxes, attack_boxes, Box4};
use super::image::load_rgba;
use super::regions::Mask;
use serde_json::Value;
use std::path::Path;

pub fn sprite_mask(path: &Path) -> Result<Mask, String> {
    let image = load_rgba(path)?;
    let mut mask = Mask::new(image.width, image.height);
    for y in 0..image.height {
        for x in 0..image.width {
            mask.set(x, y, image.pixel(x, y)[3] > 0);
        }
    }
    Ok(mask)
}

pub fn box_lines(kind: u8, boxes: &[Box4]) -> Vec<String> {
    if boxes.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![format!("Clsn{kind}: {}", boxes.len())];
    lines.extend(boxes.iter().enumerate().map(|(i, b)| format!("  Clsn{kind}[{i}] = {},{},{},{}", b.0, b.1, b.2, b.3)));
    lines
}

pub fn read_manifest(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn action_text(manifest_path: &Path, action: i64, ticks: i64, attack: bool, zones: usize) -> Result<String, String> {
    let manifest = read_manifest(manifest_path)?;
    let base = manifest_path.parent().unwrap_or(Path::new("."));
    let mut lines = vec![format!("[Begin Action {action}]")];
    for entry in manifest["sprites"].as_array().ok_or("manifest has no sprites")? {
        let mask = sprite_mask(&base.join(entry["file"].as_str().unwrap_or_default()))?;
        let axis = (entry["axis"][0].as_i64().unwrap_or(0) as i32, entry["axis"][1].as_i64().unwrap_or(0) as i32);
        lines.extend(box_lines(2, &anatomical_boxes(&mask, axis, zones, 2)));
        if attack {
            lines.extend(box_lines(1, &attack_boxes(&mask, axis, 2)));
        }
        lines.push(format!("{}, {}, 0, 0, {ticks}", entry["group"], entry["item"]));
    }
    Ok(lines.join("\n") + "\n")
}
