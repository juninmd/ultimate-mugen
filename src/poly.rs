//! Polygon toolkit: convex fills with per-vertex colour (gradients), chamfered / slanted / hexagonal panels, bevel outlines and
//! adaptive circles. The HUD and the effects are built from these so edges stay crisp and shapes keep their facets at 4K.
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

/// Fill a convex polygon; `color_at` decides each vertex colour (and the centre), the GPU interpolates → smooth gradients.
pub fn fill(pts: &[Vec2], color_at: impl Fn(Vec2) -> Color) {
    if pts.len() < 3 { return; }
    let n = pts.len();
    let c = pts.iter().fold(Vec2::ZERO, |a, p| a + *p) / n as f32;
    let mut vertices = Vec::with_capacity(n + 1);
    vertices.push(Vertex::new(c.x, c.y, 0.0, 0.0, 0.0, color_at(c)));
    for p in pts { vertices.push(Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, color_at(*p))); }
    let mut indices = Vec::with_capacity(n * 3);
    for i in 0..n { indices.extend([0u16, 1 + i as u16, 1 + ((i + 1) % n) as u16]); }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}
/// Vertical gradient `top` → `bot` across the polygon's height.
pub fn grad_v(pts: &[Vec2], top: Color, bot: Color) {
    let (lo, hi) = pts.iter().fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.y), a.1.max(p.y)));
    let span = (hi - lo).max(0.001);
    fill(pts, |p| { let t = ((p.y - lo) / span).clamp(0.0, 1.0); lerp_c(top, bot, t) });
}
/// Horizontal gradient `left` → `right`.
pub fn grad_h(pts: &[Vec2], left: Color, right: Color) {
    let (lo, hi) = pts.iter().fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.x), a.1.max(p.x)));
    let span = (hi - lo).max(0.001);
    fill(pts, |p| { let t = ((p.x - lo) / span).clamp(0.0, 1.0); lerp_c(left, right, t) });
}
pub fn solid(pts: &[Vec2], c: Color) { fill(pts, |_| c); }
pub fn lerp_c(a: Color, b: Color, t: f32) -> Color { Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t) }
pub fn with_a(c: Color, a: f32) -> Color { Color::new(c.r, c.g, c.b, a) }
pub fn shade(c: Color, k: f32) -> Color { Color::new((c.r * k).min(1.0), (c.g * k).min(1.0), (c.b * k).min(1.0), c.a) }

/// Closed outline of `pts` with thickness `th`.
pub fn outline(pts: &[Vec2], th: f32, c: Color) {
    for i in 0..pts.len() { let (a, b) = (pts[i], pts[(i + 1) % pts.len()]); draw_line(a.x, a.y, b.x, b.y, th, c); }
}
/// Octagon-like rectangle with chamfered corners (`cut` px off each corner).
pub fn chamfer(x: f32, y: f32, w: f32, h: f32, cut: f32) -> Vec<Vec2> {
    let c = cut.min(w / 2.0).min(h / 2.0);
    vec![vec2(x + c, y), vec2(x + w - c, y), vec2(x + w, y + c), vec2(x + w, y + h - c), vec2(x + w - c, y + h), vec2(x + c, y + h), vec2(x, y + h - c), vec2(x, y + c)]
}
/// Parallelogram whose top edge is shifted by `skew` px relative to the bottom edge (positive = leans right).
pub fn slanted(x: f32, y: f32, w: f32, h: f32, skew: f32) -> Vec<Vec2> {
    let (tl, bl) = if skew >= 0.0 { (skew, 0.0) } else { (0.0, -skew) };
    vec![vec2(x + tl, y), vec2(x + tl + (w - skew.abs()), y), vec2(x + bl + (w - skew.abs()), y + h), vec2(x + bl, y + h)]
}
/// Regular polygon with `sides` corners.
pub fn regular(c: Vec2, r: f32, sides: usize, rot: f32) -> Vec<Vec2> {
    (0..sides).map(|i| { let a = rot + i as f32 / sides as f32 * std::f32::consts::TAU; c + vec2(a.cos(), a.sin()) * r }).collect()
}
/// Number of facets so a circle of radius `r` px looks round (more at 4K).
pub fn facets(r: f32) -> usize { (((r.max(1.0)).sqrt() * 6.0) as usize).clamp(12, 96) }
pub fn circle(c: Vec2, r: f32, col: Color) { solid(&regular(c, r, facets(r), 0.0), col); }
pub fn ring(c: Vec2, r: f32, th: f32, col: Color) {
    let n = facets(r);
    let (o, i) = (regular(c, r + th / 2.0, n, 0.0), regular(c, (r - th / 2.0).max(0.0), n, 0.0));
    let mut vertices = Vec::with_capacity(n * 2);
    for k in 0..n { vertices.push(Vertex::new(o[k].x, o[k].y, 0.0, 0.0, 0.0, col)); vertices.push(Vertex::new(i[k].x, i[k].y, 0.0, 0.0, 0.0, col)); }
    let mut indices = Vec::with_capacity(n * 6);
    for k in 0..n { let (a, b, c2, d) = ((2 * k) as u16, (2 * k + 1) as u16, (2 * ((k + 1) % n)) as u16, (2 * ((k + 1) % n) + 1) as u16); indices.extend([a, b, c2, b, d, c2]); }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}
