use super::*;

/// Deterministic skyline: (x, width, height) of each tower, shared by the baked art and the animated blink lights.
pub fn skyline(
    seed: u32,
    total: f32,
    wmin: f32,
    wmax: f32,
    hmin: f32,
    hmax: f32,
    gap: f32,
) -> Vec<(f32, f32, f32)> {
    let (mut x, mut i, mut v) = (0.0, 0u32, vec![]);
    while x < total {
        let w = wmin + hash(i, seed) * (wmax - wmin);
        let h = hmin + hash(i, seed + 1) * (hmax - hmin);
        v.push((x, w, h));
        x += w + hash(i, seed + 2) * gap;
        i += 1;
    }
    v
}
/// Neon signs on the mid city layer: (x, y, w, h, colour index). Baked glow + animated flicker use the same list.
pub fn neon_signs() -> Vec<(f32, f32, f32, f32, usize)> {
    skyline(41, 1000.0, 28.0, 56.0, 90.0, 190.0, 54.0)
        .iter()
        .enumerate()
        .filter(|(i, _)| hash(*i as u32, 7) < 0.85)
        .map(|(i, (x, w, h))| {
            let sw = 9.0 + hash(i as u32, 8) * 7.0;
            (
                x + w / 2.0 - sw / 2.0,
                205.0 - h + 10.0 + hash(i as u32, 9) * 40.0,
                sw,
                24.0 + hash(i as u32, 10) * 34.0,
                (hash(i as u32, 11) * 5.0) as usize % 5,
            )
        })
        .collect()
}
pub const NEON: [&str; 5] = ["#ff2fd0", "#22e8ff", "#ffe23a", "#7d5bff", "#3dff8a"];
/// Floor glow cracks (obsidian): polylines in the floor layer's coordinates, shared by the baked seams and the animated pulse.
pub fn lava_cracks(w: f32) -> Vec<Vec<(f32, f32)>> {
    (0..22u32)
        .map(|i| {
            let (mut x, mut y) = (hash(i, 61) * w, 36.0 + hash(i, 62) * 76.0);
            let mut v = vec![(x, y)];
            for k in 0..5u32 {
                x += (hash(i, 70 + k) - 0.3) * 24.0;
                y = (y + (hash(i, 80 + k) - 0.5) * 14.0).clamp(33.0, 118.0);
                v.push((x, y));
            }
            v
        })
        .collect()
}
/// Stone-lantern x positions on the spirit stage's lantern layer.
pub fn toro_xs() -> Vec<f32> {
    let (mut x, mut v, mut i) = (40.0, vec![], 0u32);
    while x < 1000.0 {
        v.push(x);
        x += 80.0 + hash(i, 3) * 36.0;
        i += 1;
    }
    v
}
/// Rail lantern x positions on the dojo floor layer.
pub fn dojo_lanterns() -> Vec<f32> {
    (0..16).map(|k| 10.0 + 70.0 * k as f32 + 2.5).collect()
}

pub(super) fn pine(pm: &mut Pixmap, t: Transform, x: f32, by: f32, h: f32, col: &str) {
    let p = solid(rgba(col, 1.0));
    rect(pm, t, x - 1.0, by - h * 0.25, 2.0, h * 0.25, &p);
    for k in 0..5 {
        let f = k as f32 / 5.0;
        let y = by - h * 0.18 - f * h * 0.8;
        let w = h * 0.28 * (1.0 - f * 0.8);
        fill(
            pm,
            t,
            poly(&[
                (x - w, y + h * 0.16),
                (x, y - h * 0.1),
                (x + w, y + h * 0.16),
            ]),
            &p,
        );
    }
}
