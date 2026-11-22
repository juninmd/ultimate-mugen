use super::Shape;
use super::Shape::{Disc, Half, Poly, Rect, Star};

mod africa;
mod americas_europe;
mod asia_pacific;
mod fictional;

pub(super) const fn rect(x: f32, y: f32, w: f32, h: f32, color: u32) -> Shape {
    Rect { x, y, w, h, color }
}
pub(super) const fn disc(x: f32, y: f32, r: f32, color: u32) -> Shape {
    Disc { x, y, r, color }
}
pub(super) const fn half(x: f32, y: f32, r: f32, top: bool, color: u32) -> Shape {
    Half {
        x,
        y,
        r,
        top,
        color,
    }
}
pub(super) const fn star(x: f32, y: f32, r: f32, points: u8, color: u32) -> Shape {
    Star {
        x,
        y,
        r,
        points,
        color,
    }
}
pub(super) const fn poly(points: &'static [[f32; 2]], color: u32) -> Shape {
    Poly { points, color }
}

type Table = &'static [(&'static str, &'static [Shape])];

const TABLES: [Table; 4] = [
    asia_pacific::FLAGS,
    africa::FLAGS,
    americas_europe::FLAGS,
    fictional::FLAGS,
];

pub(super) fn all() -> impl Iterator<Item = (&'static str, &'static [Shape])> {
    TABLES.iter().flat_map(|table| table.iter().copied())
}
