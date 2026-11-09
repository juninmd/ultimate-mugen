use crate::pack::{lanczos_resize, Image};
use std::path::Path;
use ume_charforge::spritekit::encode_rgb_png;
use ume_formats::png::{decode_png, png_indexed_to_rgba};

pub const THUMB_WIDTH: usize = 240;
pub const THUMB_HEIGHT: usize = 135;

pub fn make_thumbnails(stages: &Path) -> Result<usize, String> {
    let out = stages.join("thumbs");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut sources: Vec<_> = std::fs::read_dir(stages).map_err(|e| format!("{}: {e}", stages.display()))?.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "png")).collect();
    sources.sort();
    for source in &sources {
        let png = decode_png(&std::fs::read(source).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", source.display()))?;
        let rgba = png.rgba.clone().unwrap_or_else(|| png_indexed_to_rgba(&png));
        let small = lanczos_resize(&Image { width: png.width as usize, height: png.height as usize, rgba }, THUMB_WIDTH, THUMB_HEIGHT);
        let rgb: Vec<u8> = small.rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
        let target = out.join(source.file_name().unwrap_or_default());
        std::fs::write(&target, encode_rgb_png(THUMB_WIDTH, THUMB_HEIGHT, &rgb, 9)).map_err(|e| format!("{}: {e}", target.display()))?;
    }
    Ok(sources.len())
}
