use super::{DrawOpts, Gfx, MatchRenderer};
use crate::gfx::Color;
use ume_engine::chardef::AnimFrame;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

const PASSES: [(f32, f32, f32); 2] = [(1.5, -0.5, 0.5), (3.0, -1.0, 0.2)];

#[allow(clippy::too_many_arguments)]
pub fn draw(
    g: &mut Gfx,
    m: &Match,
    f: &Fighter,
    fr: &AnimFrame,
    x: f32,
    ds: [f32; 2],
    base: &DrawOpts<'_>,
    shx: f32,
    shy: f32,
) {
    let Some((look, strength)) = crate::stagelook::rim() else {
        return;
    };
    if !f.alive || f.trans.is_some() || g.fx_gain <= 0.0 {
        return;
    }
    let side = look.rim_side;
    for (dx, dy, weight) in PASSES {
        let o = DrawOpts {
            alpha: (strength * weight * g.fx_gain.min(1.0)).min(1.0),
            add: true,
            tint: Color::new(look.rim[0], look.rim[1], look.rim[2], 1.0),
            angle: base.angle,
            pal: base.pal,
            remap: base.remap,
            ..Default::default()
        };
        MatchRenderer::draw_sprite(
            g,
            m,
            &f.anim_def,
            fr,
            x + dx * side,
            f.pos.y + f.lane + f.draw_offset.y + dy,
            f.facing,
            f.lscl,
            ds,
            &o,
            shx,
            shy,
        );
    }
}
