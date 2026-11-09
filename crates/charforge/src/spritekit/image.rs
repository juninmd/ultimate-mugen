use std::path::Path;
use ume_formats::png::{decode_png, png_indexed_to_rgba};

pub struct Rgba {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Rgba {
    pub fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let i = (y * self.width + x) * 4;
        [self.data[i], self.data[i + 1], self.data[i + 2], self.data[i + 3]]
    }
}

pub fn load_rgba(path: &Path) -> Result<Rgba, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let png = decode_png(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let data = match &png.rgba {
        Some(rgba) => rgba.clone(),
        None => png_indexed_to_rgba(&png),
    };
    Ok(Rgba { width: png.width as usize, height: png.height as usize, data })
}
