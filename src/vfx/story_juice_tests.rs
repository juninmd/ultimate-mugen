use super::super::Vfx;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

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
