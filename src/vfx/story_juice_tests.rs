use super::super::Vfx;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn arena(lanes: bool) -> Match {
    let nc: NativeChar = serde_json::from_str(include_str!("../../assets/chars/kaito.json")).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.lanes.active = lanes;
    m
}

#[test]
fn a_defeated_foe_is_celebrated_exactly_once() {
    let mut m = arena(true);
    m.fighters[1].alive = false;
    let mut vfx = Vfx::new();
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 1);
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 1);
}

#[test]
fn versus_matches_without_lanes_get_no_story_effects() {
    let mut m = arena(false);
    m.fighters[1].alive = false;
    let mut vfx = Vfx::new();
    vfx.story_juice(&m);
    assert!(vfx.floats.is_empty());
}

#[test]
fn landing_after_a_scroll_locks_the_screen_with_a_shake() {
    let mut m = arena(true);
    m.lanes.target_x = 500.0;
    m.lanes.bound_span = 380.0;
    m.stage.bound_left = -380.0;
    let mut vfx = Vfx::new();
    assert!(m.is_lane_scrolling());
    vfx.story_juice(&m);
    assert_eq!(vfx.trauma, 0.0);
    m.stage.bound_left = 120.0;
    vfx.story_juice(&m);
    assert!(vfx.trauma > 0.0);
    let after = vfx.trauma;
    vfx.story_juice(&m);
    assert_eq!(vfx.trauma, after);
}

#[test]
fn a_nearly_dead_hero_pulses_the_vignette() {
    let mut m = arena(true);
    m.fighters[0].life = m.fighters[0].life_max * 0.1;
    let mut vfx = Vfx::new();
    vfx.story_juice(&m);
    assert!(vfx.vig > 0.0);
}

#[test]
fn a_grabbed_foe_is_announced_once_per_grab() {
    let mut m = arena(true);
    m.fighters[1].state_no = 3800;
    let mut vfx = Vfx::new();
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 1);
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 1);
    m.fighters[1].state_no = 0;
    vfx.story_juice(&m);
    m.fighters[1].state_no = 3800;
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 2);
}

#[test]
fn a_combo_is_announced_at_each_milestone_only() {
    let mut m = arena(true);
    let mut vfx = Vfx::new();
    m.fighters[0].combo_count = 2;
    vfx.story_juice(&m);
    assert!(vfx.floats.is_empty());
    m.fighters[0].combo_count = 3;
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 1);
    m.fighters[0].combo_count = 4;
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 1);
    m.fighters[0].combo_count = 5;
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 2);
}

#[test]
fn a_dropped_combo_can_be_announced_again() {
    let mut m = arena(true);
    let mut vfx = Vfx::new();
    m.fighters[0].combo_count = 3;
    vfx.story_juice(&m);
    m.fighters[0].combo_count = 0;
    vfx.story_juice(&m);
    m.fighters[0].combo_count = 3;
    vfx.story_juice(&m);
    assert_eq!(vfx.floats.len(), 2);
}
