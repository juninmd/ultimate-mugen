use super::{glow_stroke, rgba, sol, BlendMode, Ctx, Rgb, PI, WHITE};

pub(super) fn draw(ctx: &mut Ctx, s: f64, ph: f64, col: Rgb, col2: Rgb) {
    let w = WHITE;
    ctx.st.blend = BlendMode::SourceOver;
    let wave = (ph * 2.0).sin() * s * 0.18;
    glow_stroke(ctx, rgba(col, 0.7), s * 0.6, 8.0, |c| {
        c.move_to(-s * 2.8, wave);
        c.quad_to(-s * 1.8, -s * 0.8 - wave, -s * 0.8, 0.0);
        c.quad_to(-s * 0.2, s * 0.35, s * 0.35, 0.0);
    });
    ctx.st.lw = s * 0.06;
    for i in 0..4 {
        let x = -s * (0.9 + i as f64 * 0.4);
        let y = -s * 0.22 + (ph * 2.0 + i as f64).sin() * s * 0.12;
        ctx.begin();
        ctx.move_to(x - s * 0.13, y - s * 0.1);
        ctx.line_to(x, y - s * 0.27);
        ctx.line_to(x + s * 0.13, y - s * 0.1);
        ctx.stroke(&sol([0xb4, 0xf5, 0xff], 0.85));
    }
    ctx.begin();
    {
        let c = &mut *ctx;
        c.move_to(-s * 0.85, -s * 0.18);
        c.quad_to(-s * 0.88, -s * 0.58, -s * 0.4, -s * 0.58);
        c.quad_to(s * 0.05, -s * 0.72, s * 0.38, -s * 0.4);
        c.quad_to(s * 0.55, -s * 0.24, s * 1.12, -s * 0.2);
        c.line_to(s * 1.22, s * 0.02);
        c.line_to(s * 0.46, s * 0.1);
        c.line_to(s * 0.98, s * 0.52);
        c.line_to(s * 0.22, s * 0.37);
        c.quad_to(-s * 0.2, s * 0.65, -s * 0.64, s * 0.34);
        c.close();
    }
    ctx.fill(&sol(col2, 1.0));
    ctx.stroke_w(&sol([0x1b, 0x62, 0x78], 1.0), s * 0.09);
    glow_stroke(ctx, rgba([0xff, 0xf5, 0xc4], 1.0), s * 0.15, 0.0, |c| {
        c.move_to(-s * 0.5, -s * 0.52);
        c.quad_to(-s * 0.95, -s * 0.86, -s * 1.1, -s * 1.06);
        c.move_to(-s * 0.12, -s * 0.59);
        c.quad_to(-s * 0.35, -s * 0.9, -s * 0.35, -s * 1.16);
    });
    ctx.begin();
    ctx.move_to(s * 0.04, -s * 0.32);
    ctx.line_to(s * 0.4, -s * 0.28);
    ctx.line_to(s * 0.25, -s * 0.12);
    ctx.close();
    ctx.fill(&sol([0x19, 0x31, 0x45], 1.0));
    ctx.begin();
    ctx.arc(s * 0.78, -s * 0.11, s * 0.04, 0.0, PI * 2.0);
    ctx.fill(&sol([0x19, 0x31, 0x45], 1.0));
    ctx.begin();
    ctx.move_to(s * 0.36, s * 0.06);
    ctx.line_to(s * 0.69, s * 0.25);
    ctx.line_to(s * 0.42, s * 0.18);
    ctx.line_to(s * 0.33, s * 0.28);
    ctx.line_to(s * 0.27, s * 0.13);
    ctx.close();
    ctx.fill(&sol(w, 1.0));
}
