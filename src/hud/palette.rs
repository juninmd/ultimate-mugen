use crate::gfx::Vec2;

pub(super) fn side_tint(side: usize) -> [f32; 3] {
    if side == 0 {
        [0.3, 0.72, 1.0]
    } else {
        [1.0, 0.38, 0.52]
    }
}

pub(super) fn poly_pt(p: impl Fn(f32, f32) -> Vec2, x: f32, y: f32) -> Vec2 {
    p(x, y)
}
