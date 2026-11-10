use super::{Canvas, Color};
use crate::batch::Vertex;

pub const MAX_SEGMENTS: usize = 96;
const FEATHER: f32 = 0.5;
const MITER_LIMIT: f32 = 3.0;
const AXIS_ALIGNED: f32 = 1e-3;
const MIN_INRADIUS: f32 = 1.0;

type Point = [f32; 2];

fn clear(c: Color) -> Color { Color([c.0[0], c.0[1], c.0[2], 0.0]) }

fn faded(c: Color, k: f32) -> Color { Color([c.0[0], c.0[1], c.0[2], c.0[3] * k]) }

fn outward_normal(a: Point, b: Point, centroid: Point) -> Option<(Point, f32)> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-4 { return None; }
    let mut n = [dy / len, -dx / len];
    if n[0] * (a[0] - centroid[0]) + n[1] * (a[1] - centroid[1]) < 0.0 { n = [-n[0], -n[1]]; }
    let feather = if n[0].abs() < AXIS_ALIGNED || n[1].abs() < AXIS_ALIGNED { 0.0 } else { FEATHER };
    Some((n, feather))
}

fn corner_offset((n1, h1): (Point, f32), (n2, h2): (Point, f32)) -> Point {
    let det = n1[0] * n2[1] - n1[1] * n2[0];
    let d = if det.abs() < 1e-4 {
        if h1 > 0.0 { [n1[0] * h1, n1[1] * h1] } else { [n2[0] * h2, n2[1] * h2] }
    } else {
        [(h1 * n2[1] - h2 * n1[1]) / det, (n1[0] * h2 - n2[0] * h1) / det]
    };
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let limit = MITER_LIMIT * FEATHER;
    if len > limit { [d[0] / len * limit, d[1] / len * limit] } else { d }
}

fn inradius(points: &[Point]) -> f32 {
    let n = points.len();
    let (mut area2, mut perimeter) = (0.0, 0.0);
    for i in 0..n {
        let (a, b) = (points[i], points[(i + 1) % n]);
        area2 += a[0] * b[1] - b[0] * a[1];
        perimeter += ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
    }
    if perimeter <= 0.0 { 0.0 } else { area2.abs() / perimeter }
}

impl Canvas {
    fn flush_shape(&mut self) {
        self.batch.push_tris(&self.shape_vertices, &self.shape_indices, self.blend);
    }

    fn begin_shape(&mut self) {
        self.shape_vertices.clear();
        self.shape_indices.clear();
    }

    fn shape_vertex(&mut self, p: Point, c: Color) -> u32 {
        let v = Vertex::new(p, self.white, crate::RGBA_SPRITE, c.0, [0.0; 4], 0.0);
        self.shape_vertices.push(v);
        self.shape_vertices.len() as u32 - 1
    }

    fn hard_fan(&mut self, points: &[Point], color: Color) {
        self.begin_shape();
        for p in points { self.shape_vertex(*p, color); }
        for i in 1..points.len() as u32 - 1 { self.shape_indices.extend_from_slice(&[0, i, i + 1]); }
        self.flush_shape();
    }

    pub fn convex(&mut self, points: &[Point], color: Color) {
        let n = points.len();
        if n < 3 { return; }
        if inradius(points) < MIN_INRADIUS { return self.hard_fan(points, color); }
        let k = 1.0 / n as f32;
        let centroid = points.iter().fold([0.0, 0.0], |c, p| [c[0] + p[0] * k, c[1] + p[1] * k]);
        self.begin_shape();
        for i in 0..n {
            let (prev, cur, next) = (points[(i + n - 1) % n], points[i], points[(i + 1) % n]);
            let (Some(e1), Some(e2)) = (outward_normal(prev, cur, centroid), outward_normal(cur, next, centroid)) else {
                return self.hard_fan(points, color);
            };
            let d = corner_offset(e1, e2);
            self.shape_vertex([cur[0] - d[0], cur[1] - d[1]], color);
            self.shape_vertex([cur[0] + d[0], cur[1] + d[1]], clear(color));
        }
        let n = n as u32;
        for i in 1..n - 1 { self.shape_indices.extend_from_slice(&[0, 2 * i, 2 * i + 2]); }
        for i in 0..n {
            let (a, b) = (2 * i, 2 * ((i + 1) % n));
            self.shape_indices.extend_from_slice(&[a, a + 1, b + 1, a, b + 1, b]);
        }
        self.flush_shape();
    }

    pub fn ellipse(&mut self, center: Point, radii: Point, rotation: f32, segments: u32, color: Color) {
        let n = (segments as usize).clamp(3, MAX_SEGMENTS);
        let (s, c) = rotation.sin_cos();
        let mut points = [[0.0f32; 2]; MAX_SEGMENTS];
        for (i, p) in points.iter_mut().take(n).enumerate() {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let (x, y) = (t.cos() * radii[0], t.sin() * radii[1]);
            *p = [center[0] + x * c - y * s, center[1] + x * s + y * c];
        }
        self.convex(&points[..n], color);
    }

    pub fn circle(&mut self, center: Point, radius: f32, color: Color) {
        self.ellipse(center, [radius, radius], 0.0, self.circle_segments, color);
    }

    fn band_profile(thickness: f32, color: Color) -> [(f32, Color); 4] {
        let half = thickness * 0.5;
        let core = (half - FEATHER).max(0.0);
        let solid = faded(color, thickness.min(1.0));
        [(half + FEATHER, clear(color)), (core, solid), (-core, solid), (-half - FEATHER, clear(color))]
    }

    pub fn circle_lines(&mut self, center: Point, radius: f32, thickness: f32, color: Color) {
        let n = self.circle_segments;
        let profile = Self::band_profile(thickness, color);
        self.begin_shape();
        for i in 0..n {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let (c, s) = (t.cos(), t.sin());
            for (offset, col) in profile {
                let r = (radius + offset).max(0.0);
                self.shape_vertex([center[0] + c * r, center[1] + s * r], col);
            }
        }
        for i in 0..n {
            let (a, b) = (i * 4, ((i + 1) % n) * 4);
            for k in 0..3 { self.shape_indices.extend_from_slice(&[a + k, b + k, a + k + 1, b + k, b + k + 1, a + k + 1]); }
        }
        self.flush_shape();
    }

    pub fn line(&mut self, a: Point, b: Point, thickness: f32, color: Color) {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 { return; }
        let (nx, ny) = (-dy / len, dx / len);
        if nx.abs() < AXIS_ALIGNED || ny.abs() < AXIS_ALIGNED {
            let (hx, hy) = (nx * thickness * 0.5, ny * thickness * 0.5);
            return self.hard_fan(&[[a[0] + hx, a[1] + hy], [b[0] + hx, b[1] + hy], [b[0] - hx, b[1] - hy], [a[0] - hx, a[1] - hy]], color);
        }
        self.begin_shape();
        for p in [a, b] {
            for (offset, col) in Self::band_profile(thickness, color) {
                self.shape_vertex([p[0] + nx * offset, p[1] + ny * offset], col);
            }
        }
        for k in 0..3 { self.shape_indices.extend_from_slice(&[k, 4 + k, 5 + k, k, 5 + k, k + 1]); }
        self.flush_shape();
    }

    pub fn radial_gradient(&mut self, center: Point, radius: f32, segments: u32, stops: &[(f32, Color)]) {
        let n = (segments as usize).clamp(3, MAX_SEGMENTS) as u32;
        let Some(((_, core), rings)) = stops.split_first() else { return };
        self.begin_shape();
        self.shape_vertex(center, *core);
        for (fraction, color) in rings {
            for i in 0..n {
                let t = i as f32 / n as f32 * std::f32::consts::TAU;
                self.shape_vertex([center[0] + t.cos() * radius * fraction, center[1] + t.sin() * radius * fraction], *color);
            }
        }
        for i in 0..n { self.shape_indices.extend_from_slice(&[0, 1 + i, 1 + (i + 1) % n]); }
        for ring in 1..rings.len() as u32 {
            let (inner, outer) = (1 + (ring - 1) * n, 1 + ring * n);
            for i in 0..n {
                let j = (i + 1) % n;
                self.shape_indices.extend_from_slice(&[inner + i, outer + i, outer + j, inner + i, outer + j, inner + j]);
            }
        }
        self.flush_shape();
    }
}

#[cfg(test)]
mod tests;
