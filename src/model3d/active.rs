use super::canvas::{self, Placement};
use super::{bindings, pose_pick};
use crate::render::Gfx;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;
use ume_model3d::Facing;

pub(super) fn draw(g: &mut Gfx, m: &Match, f: &Fighter, shx: f32, shy: f32) -> bool {
    let id = f.def.id.trim_start_matches("umf:");
    let feet = [
        f.pos.x + f.draw_offset.x * f.facing - m.camera.x + m.view_w / 2.0 + shx,
        f.pos.y + f.lane + f.draw_offset.y - m.camera.y + m.stage.zoffset + shy,
    ];
    let facing = if f.facing >= 0.0 {
        Facing::Right
    } else {
        Facing::Left
    };
    let slot = m
        .fighters
        .iter()
        .position(|o| std::ptr::eq(o, f))
        .unwrap_or(0);
    let awakened = f.form > 0;
    let at = Placement {
        facing,
        screen_scale: g.view_scale * f.lscl,
        feet: [g.ox + feet[0] * g.view_scale, g.oy + feet[1] * g.view_scale],
        samples: 1,
    };
    bindings::with(id, awakened, |b| {
        pose_pick::pose(b, f.anim_no, f.anim_elapsed, f.state_no, f.state_time);
        canvas::present(b, (id, awakened, slot), at);
    })
    .is_some()
}
