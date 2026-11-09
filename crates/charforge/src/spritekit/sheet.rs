use super::axis::{assign_axes, AxisMode};
use super::foreground::{foreground_mask, MaskOptions};
use super::image::{load_rgba, Rgba};
use super::palette::{shared_palette, to_indexed};
use super::png_out::write_indexed_png;
use super::regions::{detect_auto, detect_grid, GridOptions};
use serde_json::{json, Value};
use std::path::Path;

pub enum SliceMode {
    Auto { merge_distance: usize },
    Grid(GridOptions),
}

pub struct SliceOptions {
    pub mode: SliceMode,
    pub group: i64,
    pub start_item: i64,
    pub axis: AxisMode,
    pub mask: MaskOptions,
    pub max_colors: usize,
}

impl Default for SliceOptions {
    fn default() -> Self {
        SliceOptions { mode: SliceMode::Auto { merge_distance: 4 }, group: 0, start_item: 0, axis: AxisMode::Feet, mask: MaskOptions::default(), max_colors: 256 }
    }
}

fn crop(image: &Rgba, left: usize, top: usize, right: usize, bottom: usize, mask: &super::regions::Mask) -> Rgba {
    let (width, height) = (right - left, bottom - top);
    let mut data = Vec::with_capacity(width * height * 4);
    for y in top..bottom {
        for x in left..right {
            let pixel = image.pixel(x, y);
            data.extend_from_slice(&[pixel[0], pixel[1], pixel[2], if mask.at(x, y) { 255 } else { 0 }]);
        }
    }
    Rgba { width, height, data }
}

pub fn slice_sheet(sheet: &Path, out_dir: &Path, options: &SliceOptions) -> Result<Value, String> {
    let image = load_rgba(sheet)?;
    let mask = foreground_mask(&image, &options.mask);
    let mut regions = match &options.mode {
        SliceMode::Auto { merge_distance } => detect_auto(&mask, *merge_distance, 12),
        SliceMode::Grid(grid) => detect_grid(&mask, grid)?,
    };
    assign_axes(&mut regions, &mask, options.axis, 6);
    std::fs::create_dir_all(out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;
    let crops: Vec<Rgba> = regions.iter().map(|r| crop(&image, r.left, r.top, r.right, r.bottom, &mask)).collect();
    let (palette, exact) = shared_palette(&crops, options.max_colors);
    let mut sprites = Vec::new();
    for (offset, (region, sprite)) in regions.iter().zip(&crops).enumerate() {
        let item = options.start_item + offset as i64;
        let file = format!("{}_{}.png", options.group, item);
        write_indexed_png(&out_dir.join(&file), sprite.width, sprite.height, &to_indexed(sprite, &palette), &palette)?;
        sprites.push(json!({
            "group": options.group, "item": item, "file": file,
            "box": [region.left, region.top, region.right, region.bottom],
            "axis": [region.axis_x, region.axis_y], "row": region.row,
        }));
    }
    let manifest = json!({ "source": sheet.display().to_string(), "exact_palette": exact, "sprites": sprites });
    let text = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(out_dir.join("manifest.json"), text).map_err(|e| e.to_string())?;
    Ok(manifest)
}
