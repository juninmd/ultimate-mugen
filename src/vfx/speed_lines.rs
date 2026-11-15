use crate::gfx::*;

pub fn speed_lines(focus: (f32, f32), a: f32, col: [f32; 3], seed: u32, n: usize) {
    let (w, h) = (screen_width(), screen_height());
    let reach = (w * w + h * h).sqrt();
    let seed = seed as f32;
    set_blend(GfxBlend::Alpha);
    for i in 0..n {
        let h1 = ((i as f32 * 12.9898 + seed * 7.13).sin() * 43_758.547)
            .fract()
            .abs();
        let h2 = ((i as f32 * 78.233 + seed * 3.7).sin() * 12543.1)
            .fract()
            .abs();
        let ang = (i as f32 + h1 * 0.8) / n as f32 * std::f32::consts::TAU;
        let (r0, r1, wd) = (
            reach * (0.28 + h2 * 0.18),
            reach * (0.75 + h1 * 0.3),
            0.012 + h2 * 0.018,
        );
        let (c, s) = (ang.cos(), ang.sin());
        let (pc, ps) = ((ang + wd).cos(), (ang + wd).sin());
        draw_triangle(
            vec2(focus.0 + c * r0, focus.1 + s * r0),
            vec2(focus.0 + c * r1, focus.1 + s * r1),
            vec2(focus.0 + pc * r1, focus.1 + ps * r1),
            Color::new(col[0], col[1], col[2], a * (0.4 + h1 * 0.6)),
        );
    }
}
