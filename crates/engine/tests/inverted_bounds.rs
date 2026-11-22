//! Content we do not control must never abort the simulation: M.U.G.E.N. packs ship inverted camera bounds, inverted
//! lane limits and life values a character script can push out of order. `f32::clamp(min > max)` panics, and in the
//! browser build a panic inside the frame loop kills the whole page (`RuntimeError: unreachable executed`).
mod support;
use support::*;

const CNS: &str = "\n[Statedef -1]\ntype = S\nmovetype = I\nphysics = S\nanim = 0\nctrl = 1\n\n[State -1, idle]\ntype = ChangeState\ntrigger1 = 1\nvalue = 0\n";

fn build() -> ume_engine::matchs::Match {
    duel(mugen_char("a", CNS), mugen_char("b", CNS))
}

#[test]
fn life_above_a_negative_max_does_not_abort_the_tick() {
    let mut m = build();
    for f in m.fighters.iter_mut() {
        f.life = 10.0;
        f.life_max = -5.0;
        f.ikemen.red_life = 40.0;
    }
    ticks(&mut m, 30);
    for f in &m.fighters {
        assert!(f.life.is_finite() && f.ikemen.red_life.is_finite());
    }
}

#[test]
fn inverted_camera_bounds_survive_the_camera_tick() {
    let mut m = build();
    m.stage.bound_left = 10.0;
    m.stage.bound_right = -5.0;
    m.stage.bound_high = 0.0;
    m.stage.bound_low = -9.0;
    ticks(&mut m, 30);
    assert!(m.camera.x.is_finite() && m.camera.y.is_finite());
}

#[test]
fn inverted_lane_limits_survive_the_lane_step() {
    let mut m = build();
    m.lanes = ume_engine::lane::LaneRules { active: true, far: 10.0, near: -5.0, ..Default::default() };
    ticks(&mut m, 30);
    assert!(m.fighters.iter().all(|f| f.lane.is_finite()));
}

#[test]
fn a_zoom_range_that_would_invert_keeps_the_camera_finite() {
    let mut m = build();
    m.stage.zoom_out = 2.0;
    m.set_zoom_for_teams(5, 0.5);
    ticks(&mut m, 30);
    assert!(m.zoom.is_finite() && m.zoom > 0.0);
}
