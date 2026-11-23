use super::canvas::{self, Placement};
use super::{bindings, pose_pick};
use ume_model3d::Facing;

const FIT_SHARE: f32 = 0.84;
const FIT_SLOT: usize = 100;

pub(super) fn draw(
    id: &str,
    anim: i32,
    ticks: i32,
    center_x: f32,
    bottom: f32,
    height: f32,
    mirror: bool,
) -> bool {
    let facing = if mirror { Facing::Left } else { Facing::Right };
    let id = id.trim_start_matches("umf:");
    bindings::with(id, false, |b| {
        pose_pick::pose(b, anim, ticks, 0, ticks);
        let at = Placement {
            facing,
            screen_scale: height * FIT_SHARE / b.height,
            feet: [center_x, bottom],
            samples: 2,
        };
        canvas::present(b, (id, false, FIT_SLOT + mirror as usize), at).draw([0.0, 0.0], crate::gfx::WHITE);
    })
    .is_some()
}
