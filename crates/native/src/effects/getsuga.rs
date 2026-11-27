use super::{glow_fill, rgba, sol, BlendMode, Ctx, Rgb};

pub(super) fn draw(ctx: &mut Ctx, s: f64, ph: f64, col: Rgb, col2: Rgb) {
    ctx.st.blend = BlendMode::SourceOver;
    let crescent = |c: &mut Ctx| {
        c.move_to(-s * 0.2, -s);
        c.quad_to(s * 1.10, -s * 0.22, -s * 0.2, s);
        c.quad_to(s * (0.32 + ph.sin() * 0.05), s * 0.05, -s * 0.2, -s);
        c.close();
    };
    glow_fill(ctx, rgba(col2, 0.9), 7.0, crescent);
    ctx.begin();
    crescent(ctx);
    ctx.fill(&sol(col, 1.0));
    ctx.stroke_w(&sol(col2, 0.95), 2.0);
}
