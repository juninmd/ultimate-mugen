use vello::kurbo::{Rect, RoundedRect};
use vello::peniko::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarSpec {
    pub rect: Rect,
    pub fraction: f32,
    pub trail: f32,
    pub fill: Color,
    pub trail_fill: Color,
    pub track: Color,
    pub radius: f64,
}

pub fn bar_shapes(spec: &BarSpec) -> [RoundedRect; 3] {
    let f = spec.fraction.clamp(0.0, 1.0) as f64;
    let t = spec.trail.clamp(0.0, 1.0) as f64;
    let r = spec.rect;
    let w = r.width();
    let track = RoundedRect::from_rect(r, spec.radius);
    let trail = RoundedRect::from_rect(Rect::new(r.x0, r.y0, r.x0 + w * t.max(f), r.y1), spec.radius);
    let fill = RoundedRect::from_rect(Rect::new(r.x0, r.y0, r.x0 + w * f, r.y1), spec.radius);
    [track, trail, fill]
}

pub fn colors(spec: &BarSpec) -> [Color; 3] {
    [spec.track, spec.trail_fill, spec.fill]
}
