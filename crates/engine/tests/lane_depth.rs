mod support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const JAB_STATE: i32 = 201;

fn arena(lanes: bool) -> Match {
    let path = format!("{}/../../assets/chars/kaito.json", env!("CARGO_MANIFEST_DIR"));
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.lanes.active = lanes;
    m.lanes.far = -40.0;
    m.lanes.near = 40.0;
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    for _ in 0..240 {
        if m.fighters.iter().all(|f| f.ctrl) {
            break;
        }
        m.tick(&[0, 0]);
    }
    m
}

fn damage_after_jab(lane_gap: f32, lanes: bool) -> f32 {
    let mut m = arena(lanes);
    face_off(&mut m, 30.0);
    m.fighters[1].lane = lane_gap;
    let start = m.fighters[1].life;
    enter(&mut m, 0, JAB_STATE);
    ticks(&mut m, 60);
    start - m.fighters[1].life
}

#[test]
fn a_strike_only_lands_when_both_fighters_share_a_lane() {
    assert!(damage_after_jab(0.0, true) > 0.0, "same lane connects");
    assert!(damage_after_jab(8.0, true) > 0.0, "inside the tolerance still connects");
    assert_eq!(damage_after_jab(30.0, true), 0.0, "a far lane is out of reach");
}

#[test]
fn without_lane_mode_depth_is_ignored_so_other_modes_are_unchanged() {
    assert!(damage_after_jab(30.0, false) > 0.0);
}

#[test]
fn lane_buttons_walk_the_floor_band_and_stay_inside_it() {
    let mut m = arena(true);
    m.lanes.far = -12.0;
    m.lanes.near = 6.0;
    for _ in 0..60 {
        m.tick(&[k::LANE_UP, 0]);
    }
    assert_eq!(m.fighters[0].lane, -12.0);
    assert_eq!(m.fighters[1].lane, 0.0, "the other fighter keeps its lane");
    for _ in 0..60 {
        m.tick(&[k::LANE_DOWN, 0]);
    }
    assert_eq!(m.fighters[0].lane, 6.0);
}

#[test]
fn fighters_on_different_lanes_do_not_push_each_other() {
    let mut m = arena(true);
    face_off(&mut m, 4.0);
    m.fighters[1].lane = 30.0;
    let (x0, x1) = (m.fighters[0].pos.x, m.fighters[1].pos.x);
    ticks(&mut m, 5);
    assert_eq!((m.fighters[0].pos.x - x0).abs() + (m.fighters[1].pos.x - x1).abs(), 0.0);
}

fn defender_damage(lanes: bool, defender_input: u32) -> (f32, bool) {
    let mut m = arena(lanes);
    face_off(&mut m, 30.0);
    enter(&mut m, 1, 0);
    m.fighters[1].ctrl = true;
    ticks(&mut m, 2);
    let start = m.fighters[1].life;
    enter(&mut m, 0, JAB_STATE);
    let mut guarded = false;
    for _ in 0..60 {
        m.tick(&[0, defender_input]);
        guarded |= (120..=155).contains(&m.fighters[1].state_no);
    }
    (start - m.fighters[1].life, guarded)
}

#[test]
fn the_guard_button_blocks_without_holding_back_and_only_in_lane_mode() {
    let (open, open_guarded) = defender_damage(true, 0);
    let (blocked, blocked_guarded) = defender_damage(true, k::GUARD);
    assert!(!open_guarded && open > 0.0);
    assert!(blocked_guarded, "the guard button puts the defender in a guard state");
    assert!(blocked < open, "guarded damage ({blocked}) is below open damage ({open})");
    let (outside, outside_guarded) = defender_damage(false, k::GUARD);
    assert!(!outside_guarded && outside > 0.0, "the button does nothing outside lane mode");
}
