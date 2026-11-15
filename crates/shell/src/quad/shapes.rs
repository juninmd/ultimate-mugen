use super::{with, Blend, Color, Vec2};

pub const MAX_CONVEX_POINTS: usize = 16;

pub fn current_blend() -> Blend { with(|f| f.canvas.blend) }

pub fn draw_convex(points: &[Vec2], c: Color) {
    let n = points.len().min(MAX_CONVEX_POINTS);
    let mut flat = [[0.0f32; 2]; MAX_CONVEX_POINTS];
    for (slot, p) in flat.iter_mut().zip(points) { *slot = [p.x, p.y]; }
    with(|f| f.canvas.convex(&flat[..n], c.render()));
}

pub fn draw_ellipse_segments(x: f32, y: f32, rx: f32, ry: f32, rotation_radians: f32, segments: u32, c: Color) {
    with(|f| f.canvas.ellipse([x, y], [rx, ry], rotation_radians, segments, c.render()));
}

pub fn draw_radial_gradient(x: f32, y: f32, radius: f32, segments: u32, stops: &[(f32, Color)]) {
    const MAX_STOPS: usize = 8;
    let n = stops.len().min(MAX_STOPS);
    let mut rendered = [(0.0f32, ume_render::Color([0.0; 4])); MAX_STOPS];
    for (slot, (fraction, color)) in rendered.iter_mut().zip(stops) { *slot = (*fraction, color.render()); }
    with(|f| f.canvas.radial_gradient([x, y], radius, segments, &rendered[..n]));
}
