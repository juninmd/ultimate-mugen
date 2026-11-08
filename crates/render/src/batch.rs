use crate::atlas::Rect;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub pal: u32,
    pub mul: [u16; 4],
    pub add: [u16; 4],
    pub hue: f32,
}

const MUL_RGB_RANGE: f32 = 2.0;
const ADD_W_RANGE: f32 = 3.0;

fn unorm16(value: f32, range: f32) -> u16 {
    (value.clamp(0.0, range) / range * 65535.0).round() as u16
}

impl Vertex {
    pub fn new(
        pos: [f32; 2],
        uv: [f32; 2],
        pal: u32,
        mul: [f32; 4],
        add: [f32; 4],
        hue: f32,
    ) -> Self {
        Vertex {
            pos,
            uv,
            pal,
            mul: [
                unorm16(mul[0], MUL_RGB_RANGE),
                unorm16(mul[1], MUL_RGB_RANGE),
                unorm16(mul[2], MUL_RGB_RANGE),
                unorm16(mul[3], 1.0),
            ],
            add: [
                unorm16(add[0], 1.0),
                unorm16(add[1], 1.0),
                unorm16(add[2], 1.0),
                unorm16(add[3], ADD_W_RANGE),
            ],
            hue,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalFx {
    pub add: [f32; 3],
    pub mul: [f32; 3],
    pub color: f32,
    pub invert: bool,
    pub hue: f32,
}

impl Default for PalFx {
    fn default() -> Self {
        PalFx {
            add: [0.0; 3],
            mul: [1.0; 3],
            color: 1.0,
            invert: false,
            hue: 0.0,
        }
    }
}

impl PalFx {
    pub fn vertex(&self, tint: [f32; 4]) -> ([f32; 4], [f32; 4], f32) {
        let mul = [0, 1, 2].map(|k| tint[k] * self.mul[k]);
        let add = [0, 1, 2].map(|k| self.add[k] * mul[k]);
        let w = (1.0 - self.color.clamp(0.0, 1.0)) + if self.invert { 2.0 } else { 0.0 };
        (
            [mul[0], mul[1], mul[2], tint[3]],
            [add[0], add[1], add[2], w],
            self.hue,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Blend {
    #[default]
    Alpha,
    Add,
    Sub,
}

#[derive(Clone, Copy, Debug)]
pub struct Quad {
    pub dst: [f32; 4],
    pub src: Rect,
    pub atlas: [u32; 2],
    pub palette: u32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub mul: [f32; 4],
    pub add: [f32; 3],
    pub blend: Blend,
}

impl Quad {
    pub fn new(dst: [f32; 4], src: Rect, atlas: [u32; 2], palette: u32) -> Self {
        Quad {
            dst,
            src,
            atlas,
            palette,
            flip_h: false,
            flip_v: false,
            mul: [1.0; 4],
            add: [0.0; 3],
            blend: Blend::Alpha,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawCall {
    pub blend: Blend,
    pub first_index: u32,
    pub index_count: u32,
    pub scissor: Option<[u32; 4]>,
}

#[derive(Default)]
pub struct Batch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub calls: Vec<DrawCall>,
    pub scissor: Option<[u32; 4]>,
    pub overlay_from: Option<usize>,
    pub max_index: u32,
}

impl Batch {
    pub fn new() -> Self {
        Batch::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.calls.clear();
        self.scissor = None;
        self.overlay_from = None;
        self.max_index = 0;
    }

    pub fn fits_u16(&self) -> bool {
        self.max_index <= u16::MAX as u32
    }

    pub fn begin_overlay(&mut self) {
        self.overlay_from = Some(self.calls.len());
    }

    pub fn split(&self) -> usize {
        self.overlay_from
            .unwrap_or(self.calls.len())
            .min(self.calls.len())
    }

    pub fn push(&mut self, q: &Quad) {
        let [x, y, w, h] = q.dst;
        let (aw, ah) = (q.atlas[0] as f32, q.atlas[1] as f32);
        let (mut u0, mut v0) = (q.src.x as f32 / aw, q.src.y as f32 / ah);
        let (mut u1, mut v1) = (
            (q.src.x + q.src.w) as f32 / aw,
            (q.src.y + q.src.h) as f32 / ah,
        );
        if q.flip_h {
            std::mem::swap(&mut u0, &mut u1);
        }
        if q.flip_v {
            std::mem::swap(&mut v0, &mut v1);
        }
        let add = [q.add[0], q.add[1], q.add[2], 0.0];
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&[
            Vertex::new([x, y], [u0, v0], q.palette, q.mul, add, 0.0),
            Vertex::new([x + w, y], [u1, v0], q.palette, q.mul, add, 0.0),
            Vertex::new([x + w, y + h], [u1, v1], q.palette, q.mul, add, 0.0),
            Vertex::new([x, y + h], [u0, v1], q.palette, q.mul, add, 0.0),
        ]);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        self.max_index = self.max_index.max(base + 3);
        self.extend_call(q.blend, 6);
    }

    #[inline]
    pub fn push_quad_verts(&mut self, verts: &[Vertex; 4], blend: Blend) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(verts);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        self.max_index = self.max_index.max(base + 3);
        self.extend_call(blend, 6);
    }

    pub fn push_tris(&mut self, verts: &[Vertex], indices: &[u32], blend: Blend) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(verts);
        self.indices.extend(indices.iter().map(|i| base + i));
        let top = indices
            .iter()
            .copied()
            .max()
            .map_or(self.max_index, |i| base + i);
        self.max_index = self.max_index.max(top);
        self.extend_call(blend, indices.len() as u32);
    }

    fn extend_call(&mut self, blend: Blend, count: u32) {
        let first = self.indices.len() as u32 - count;
        let at_overlay = self.overlay_from == Some(self.calls.len());
        match self.calls.last_mut() {
            Some(c) if c.blend == blend && c.scissor == self.scissor && !at_overlay => {
                c.index_count += count
            }
            _ => self.calls.push(DrawCall {
                blend,
                first_index: first,
                index_count: count,
                scissor: self.scissor,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(blend: Blend, x: f32) -> Quad {
        let mut q = Quad::new(
            [x, 0.0, 8.0, 8.0],
            Rect {
                x: 0,
                y: 0,
                w: 8,
                h: 8,
            },
            [64, 64],
            0,
        );
        q.blend = blend;
        q
    }

    #[test]
    fn a_fight_screen_stays_within_the_draw_call_budget() {
        let mut b = Batch::new();
        for i in 0..12 {
            b.push(&quad(Blend::Alpha, i as f32));
        }
        b.push(&quad(Blend::Sub, 0.0));
        for i in 0..3 {
            b.push(&quad(Blend::Alpha, i as f32));
        }
        for i in 0..160 {
            b.push(&quad(Blend::Alpha, i as f32));
        }
        for i in 0..3 {
            b.push(&quad(Blend::Add, i as f32));
            b.push(&quad(Blend::Alpha, i as f32));
        }
        b.begin_overlay();
        b.scissor = Some([0, 0, 320, 240]);
        for i in 0..40 {
            b.push(&quad(Blend::Alpha, i as f32));
        }
        b.scissor = Some([320, 240, 320, 240]);
        for i in 0..40 {
            b.push(&quad(Blend::Alpha, i as f32));
        }
        assert!(
            b.calls.len() <= 12,
            "fight frame used {} calls",
            b.calls.len()
        );
        assert!(b.calls.len() <= crate::DRAW_CALL_BUDGET);
        assert_eq!(b.vertices.len(), (12 + 1 + 3 + 160 + 6 + 80) * 4);
    }

    #[test]
    fn a_flat_run_of_quads_is_one_call() {
        let mut b = Batch::new();
        for i in 0..300 {
            b.push(&quad(Blend::Alpha, i as f32));
        }
        assert_eq!(b.calls.len(), 1);
        assert_eq!(b.calls[0].index_count, 300 * 6);
    }

    #[test]
    fn merges_calls_with_same_blend() {
        let mut b = Batch::new();
        let r = Rect {
            x: 0,
            y: 0,
            w: 8,
            h: 8,
        };
        let mut q = Quad::new([0.0, 0.0, 8.0, 8.0], r, [64, 64], 0);
        b.push(&q);
        b.push(&q);
        q.blend = Blend::Add;
        b.push(&q);
        q.blend = Blend::Alpha;
        b.push(&q);
        assert_eq!(b.vertices.len(), 16);
        assert_eq!(
            b.calls,
            vec![
                DrawCall {
                    blend: Blend::Alpha,
                    first_index: 0,
                    index_count: 12,
                    scissor: None
                },
                DrawCall {
                    blend: Blend::Add,
                    first_index: 12,
                    index_count: 6,
                    scissor: None
                },
                DrawCall {
                    blend: Blend::Alpha,
                    first_index: 18,
                    index_count: 6,
                    scissor: None
                },
            ]
        );
    }

    #[test]
    fn overlay_starts_a_new_call() {
        let mut b = Batch::new();
        let q = Quad::new(
            [0.0, 0.0, 8.0, 8.0],
            Rect {
                x: 0,
                y: 0,
                w: 8,
                h: 8,
            },
            [64, 64],
            0,
        );
        b.push(&q);
        b.begin_overlay();
        b.push(&q);
        b.push(&q);
        assert_eq!(b.calls.len(), 2);
        assert_eq!(b.split(), 1);
        assert_eq!(b.calls[1].index_count, 12);
        b.clear();
        assert_eq!(b.split(), 0);
    }

    #[test]
    fn picks_the_narrow_index_format_until_it_does_not_fit() {
        let mut b = Batch::new();
        for i in 0..16384 {
            b.push(&quad(Blend::Alpha, i as f32));
        }
        assert_eq!(b.max_index, 65535);
        assert!(b.fits_u16());
        b.push(&quad(Blend::Alpha, 0.0));
        assert_eq!(b.max_index, 65539);
        assert!(!b.fits_u16());
        b.clear();
        assert!(b.fits_u16());
    }

    #[test]
    fn flips_uvs() {
        let mut b = Batch::new();
        let mut q = Quad::new(
            [10.0, 20.0, 4.0, 2.0],
            Rect {
                x: 8,
                y: 16,
                w: 8,
                h: 16,
            },
            [32, 32],
            3,
        );
        q.flip_h = true;
        b.push(&q);
        assert_eq!(b.vertices[0].uv, [0.5, 0.5]);
        assert_eq!(b.vertices[1].uv, [0.25, 0.5]);
        assert_eq!(b.vertices[2].pos, [14.0, 22.0]);
        assert_eq!(b.vertices[2].uv, [0.25, 1.0]);
        let mut again = Batch::new();
        again.push(&q);
        b.clear();
        assert!(b.vertices.is_empty() && b.calls.is_empty());
        assert_eq!(again.vertices[3].pal, 3);
    }
}
