use crate::Gpu;
use std::collections::HashMap;
use ume_render::{Canvas, Color, Quad, Rect};

#[derive(Clone, Copy)]
struct Glyph { rect: Option<Rect>, left: f32, top: f32, advance: f32 }

pub struct Font { font: fontdue::Font, glyphs: HashMap<(char, u16), Glyph> }

fn quantize(px: f32) -> u16 { px.round().clamp(1.0, 512.0) as u16 }

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Font, String> {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).map_err(|e| e.to_string())?;
        Ok(Font { font, glyphs: HashMap::new() })
    }

    pub fn ascent(&self, px: f32) -> f32 {
        let px = quantize(px) as f32;
        self.font.horizontal_line_metrics(px).map(|m| m.ascent).unwrap_or(px * 0.8)
    }

    pub fn measure(&self, text: &str, px: f32) -> [f32; 2] {
        let size = quantize(px) as f32;
        let width = text.chars().map(|c| self.font.metrics(c, size).advance_width).sum();
        let height = self.font.horizontal_line_metrics(size).map(|m| m.new_line_size).unwrap_or(size);
        [width, height]
    }

    fn glyph(&mut self, gpu: &mut Gpu, c: char, px: u16) -> Glyph {
        if let Some(g) = self.glyphs.get(&(c, px)) { return *g; }
        let (m, coverage) = self.font.rasterize(c, px as f32);
        let rect = (m.width > 0 && m.height > 0).then(|| gpu.rgba_atlas.alloc(m.width as u32, m.height as u32)).flatten();
        if let Some(r) = rect {
            let rgba: Vec<u8> = coverage.iter().flat_map(|&a| [255, 255, 255, a]).collect();
            gpu.renderer.upload_rgba(&gpu.queue, r, &rgba);
        }
        let g = Glyph { rect, left: m.xmin as f32, top: -(m.ymin as f32 + m.height as f32), advance: m.advance_width };
        self.glyphs.insert((c, px), g);
        g
    }

    pub fn draw(&mut self, gpu: &mut Gpu, canvas: &mut Canvas, text: &str, x: f32, baseline: f32, px: f32, color: Color) -> f32 {
        let q = quantize(px);
        let scale = px / q as f32;
        let mut pen = x;
        for c in text.chars() {
            let g = self.glyph(gpu, c, q);
            if let Some(r) = g.rect {
                let dst = [pen + g.left * scale, baseline + g.top * scale, r.w as f32 * scale, r.h as f32 * scale];
                let mut quad = Quad::new(dst, r, gpu.renderer.atlas_size, ume_render::RGBA_SPRITE);
                quad.mul = color.0;
                canvas.sprite(quad);
            }
            pen += g.advance * scale;
        }
        pen - x
    }

    pub fn cached_glyphs(&self) -> usize { self.glyphs.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_without_rasterizing() {
        let f = Font::from_bytes(include_bytes!("../../../assets/fonts/Rajdhani.ttf")).unwrap();
        let [w1, h] = f.measure("A", 32.0);
        let [w3, _] = f.measure("AAA", 32.0);
        assert!(w1 > 0.0 && h > 0.0);
        assert!((w3 - 3.0 * w1).abs() < 0.01);
        assert!(f.ascent(32.0) > 0.0);
        assert_eq!(f.cached_glyphs(), 0);
    }
}
