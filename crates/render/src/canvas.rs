use crate::atlas::{Atlas, Rect};
use crate::batch::{Batch, Blend, Quad, Vertex};
use crate::gpu::Renderer;
use crate::RGBA_SPRITE;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub [f32; 4]);

impl Color {
    pub const WHITE: Color = Color([1.0; 4]);
    pub const BLACK: Color = Color([0.0, 0.0, 0.0, 1.0]);
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color { Color([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]) }
}

pub struct Canvas {
    pub batch: Batch,
    pub blend: Blend,
    atlas: [u32; 2],
    white: [f32; 2],
    circle_segments: u32,
}

impl Canvas {
    pub fn new(renderer: &Renderer, queue: &wgpu::Queue, rgba_atlas: &mut Atlas) -> Option<Canvas> {
        let r = rgba_atlas.alloc(1, 1)?;
        renderer.upload_rgba(queue, r, &[255; 4]);
        Some(Canvas::with_white(r, renderer.atlas_size))
    }

    pub fn with_white(white: Rect, atlas: [u32; 2]) -> Canvas {
        let white = [(white.x as f32 + 0.5) / atlas[0] as f32, (white.y as f32 + 0.5) / atlas[1] as f32];
        Canvas { batch: Batch::new(), blend: Blend::Alpha, atlas, white, circle_segments: 32 }
    }

    pub fn clear(&mut self) { self.batch.clear(); }

    pub fn white_uv(&self) -> [f32; 2] { self.white }

    fn v(&self, x: f32, y: f32, c: Color) -> Vertex { Vertex::new([x, y], self.white, RGBA_SPRITE, c.0, [0.0; 4], 0.0) }

    pub fn triangle(&mut self, a: [f32; 2], b: [f32; 2], c: [f32; 2], color: Color) {
        let vs = [self.v(a[0], a[1], color), self.v(b[0], b[1], color), self.v(c[0], c[1], color)];
        self.batch.push_tris(&vs, &[0, 1, 2], self.blend);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let vs = [self.v(x, y, color), self.v(x + w, y, color), self.v(x + w, y + h, color), self.v(x, y + h, color)];
        self.batch.push_tris(&vs, &[0, 1, 2, 0, 2, 3], self.blend);
    }

    pub fn rect_lines(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        let t = thickness;
        self.rect(x, y, w, t, color);
        self.rect(x, y + h - t, w, t, color);
        self.rect(x, y + t, t, h - 2.0 * t, color);
        self.rect(x + w - t, y + t, t, h - 2.0 * t, color);
    }

    pub fn line(&mut self, a: [f32; 2], b: [f32; 2], thickness: f32, color: Color) {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 { return; }
        let (nx, ny) = (-dy / len * thickness * 0.5, dx / len * thickness * 0.5);
        let vs = [self.v(a[0] + nx, a[1] + ny, color), self.v(b[0] + nx, b[1] + ny, color), self.v(b[0] - nx, b[1] - ny, color), self.v(a[0] - nx, a[1] - ny, color)];
        self.batch.push_tris(&vs, &[0, 1, 2, 0, 2, 3], self.blend);
    }

    pub fn circle(&mut self, center: [f32; 2], radius: f32, color: Color) {
        let n = self.circle_segments;
        let mut vs = Vec::with_capacity(n as usize + 1);
        vs.push(self.v(center[0], center[1], color));
        for i in 0..n {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            vs.push(self.v(center[0] + t.cos() * radius, center[1] + t.sin() * radius, color));
        }
        let idx: Vec<u32> = (0..n).flat_map(|i| [0, 1 + i, 1 + (i + 1) % n]).collect();
        self.batch.push_tris(&vs, &idx, self.blend);
    }

    pub fn circle_lines(&mut self, center: [f32; 2], radius: f32, thickness: f32, color: Color) {
        let n = self.circle_segments;
        let (ro, ri) = (radius + thickness * 0.5, (radius - thickness * 0.5).max(0.0));
        let mut vs = Vec::with_capacity(n as usize * 2);
        for i in 0..n {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let (c, s) = (t.cos(), t.sin());
            vs.push(self.v(center[0] + c * ro, center[1] + s * ro, color));
            vs.push(self.v(center[0] + c * ri, center[1] + s * ri, color));
        }
        let idx: Vec<u32> = (0..n).flat_map(|i| { let (a, b) = (i * 2, ((i + 1) % n) * 2); [a, b, a + 1, b, b + 1, a + 1] }).collect();
        self.batch.push_tris(&vs, &idx, self.blend);
    }

    pub fn mesh(&mut self, points: &[[f32; 2]], colors: &[Color], indices: &[u32]) {
        let vs: Vec<Vertex> = points.iter().zip(colors.iter().cycle()).map(|(p, c)| self.v(p[0], p[1], *c)).collect();
        self.batch.push_tris(&vs, indices, self.blend);
    }

    pub fn sprite(&mut self, q: Quad) {
        let mut q = q;
        q.atlas = self.atlas;
        if q.blend == Blend::Alpha { q.blend = self.blend; }
        self.batch.push(&q);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_produce_expected_geometry() {
        let mut c = Canvas::with_white(Rect { x: 0, y: 0, w: 1, h: 1 }, [16, 16]);
        c.rect(0.0, 0.0, 10.0, 5.0, Color::WHITE);
        assert_eq!((c.batch.vertices.len(), c.batch.indices.len()), (4, 6));
        c.line([0.0, 0.0], [10.0, 0.0], 2.0, Color::BLACK);
        assert_eq!(c.batch.vertices[4].pos, [0.0, 1.0]);
        assert_eq!(c.batch.vertices[6].pos, [10.0, -1.0]);
        c.line([1.0, 1.0], [1.0, 1.0], 2.0, Color::BLACK);
        assert_eq!(c.batch.vertices.len(), 8);
        c.circle([0.0, 0.0], 4.0, Color::WHITE);
        assert_eq!(c.batch.vertices.len(), 8 + 33);
        assert_eq!(c.batch.indices.len(), 12 + 96);
        assert_eq!(c.batch.calls.len(), 1);
        c.blend = Blend::Add;
        c.triangle([0.0, 0.0], [1.0, 0.0], [0.0, 1.0], Color::WHITE);
        assert_eq!(c.batch.calls.len(), 2);
        assert!(c.batch.indices.iter().all(|&i| (i as usize) < c.batch.vertices.len()));
        assert!(c.batch.vertices.iter().all(|v| v.uv == [0.5 / 16.0, 0.5 / 16.0]));
    }
}
