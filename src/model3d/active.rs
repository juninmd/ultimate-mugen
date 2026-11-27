use super::canvas::{self, Placement};
use super::{aura, bindings, pose_pick};
use crate::gfx::WHITE;
use crate::render::Gfx;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::{Match, ENGINE_STATE_AWAKENING};

use ume_model3d::Facing;

const HOLLOW_REVEAL_TICK: i32 = 25;

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
    let transforming = id == "ichigo3d" && f.state_no == ENGINE_STATE_AWAKENING;
    let awakened = f.form > 0 && (!transforming || f.state_time >= HOLLOW_REVEAL_TICK);
    let view = crate::cinematic::view_for(m, f.side);
    let mut at = Placement {
        facing,
        screen_scale: g.view_scale * f.lscl,
        feet: [g.ox + feet[0] * g.view_scale, g.oy + feet[1] * g.view_scale],
        samples: 1,
        yaw: view.yaw,
        pitch: view.pitch,
        lens: view.lens,
        aim: 0.0,
    };
    let feet_px = at.feet;
    bindings::with(id, awakened, |b| {
        at.aim = crate::cinematic::close_up(&mut at.screen_scale, &mut at.feet[1], b.height, &view);
        if transforming {
            b.model.pose_for_state(&b.states, f.state_no, f.state_time, &mut b.posed);
        } else {
            pose_pick::pose(b, f.anim_no, f.anim_elapsed, f.state_no, f.state_time);
        }
        let height = b.height * at.screen_scale;
        let drawn = canvas::present(b, (id, awakened, slot), at);
        aura::trail(g, f, &drawn, awakened);
        drawn.draw([0.0, 0.0], WHITE);
        aura::guard(f, feet_px, height, awakened);
    })
    .is_some()
}
