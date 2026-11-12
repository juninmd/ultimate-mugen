use crate::gfx::Vec2;

pub(super) fn side_tint(side: usize) -> [f32; 3] {
    crate::tweaks::current().tint(side)
}

pub(super) fn poly_pt(p: impl Fn(f32, f32) -> Vec2, x: f32, y: f32) -> Vec2 {
    p(x, y)
}
