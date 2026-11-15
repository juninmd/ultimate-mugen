use crate::gfx::Vec2;

const COLORBLIND_TINTS: [[f32; 3]; 2] = [[0.0, 0.447, 0.698], [0.902, 0.624, 0.0]];

pub(super) fn side_tint(side: usize) -> [f32; 3] {
    let tweaks = crate::tweaks::current();
    if tweaks.colorblind {
        return COLORBLIND_TINTS[side.min(1)];
    }
    tweaks.tint(side)
}

pub(super) fn poly_pt(p: impl Fn(f32, f32) -> Vec2, x: f32, y: f32) -> Vec2 {
    p(x, y)
}
