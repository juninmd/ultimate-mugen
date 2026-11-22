use std::path::Path;
pub use ume_formats::png_encode::{encode_palette_png, encode_rgb_png};

pub fn encode_indexed_png(width: usize, height: usize, indices: &[u8], palette: &[[u8; 3]]) -> Vec<u8> {
    let rgba: Vec<[u8; 4]> = palette.iter().enumerate().map(|(i, c)| [c[0], c[1], c[2], if i == 0 { 0 } else { 255 }]).collect();
    encode_palette_png(width, height, indices, &rgba, 6)
}

pub fn write_indexed_png(path: &Path, width: usize, height: usize, indices: &[u8], palette: &[[u8; 3]]) -> Result<(), String> {
    std::fs::write(path, encode_indexed_png(width, height, indices, palette)).map_err(|e| format!("{}: {e}", path.display()))
}
