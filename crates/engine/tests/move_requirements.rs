mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn load(id: &str) -> Arc<CharDef> {
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, true))
}

fn state_of(def: &CharDef, name: &str) -> i32 { def.movelist.iter().find(|m| m.name == name).unwrap().state }

fn arena(def: &Arc<CharDef>, gap: f32) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    face_off(&mut m, gap);
    m.fighters[0].power = 3000.0;
    m
}

fn quarter_back(button: u32) -> Vec<u32> {
    let mut frames = vec![k::D; 3];
    frames.extend([k::D | k::L; 3]);
    frames.extend([k::L; 3]);
    frames.extend([button; 3]);
    frames
}

fn quarter_forward(button: u32) -> Vec<u32> {
    let mut frames = vec![k::D; 3];
    frames.extend([k::D | k::R; 3]);
    frames.extend([k::R; 3]);
    frames.extend([button; 3]);
    frames
}

fn reached(m: &mut Match, frames: Vec<u32>, states: &[i32]) -> Option<i32> {
    for bits in frames.into_iter().chain(std::iter::repeat_n(0, 12)) {
        m.tick(&[bits, 0]);
        let s = m.fighters[0].state_no;
        if states.contains(&s) { return Some(s); }
    }
    None
}

#[test]
fn the_same_input_picks_the_close_range_version_only_when_close() {
    let def = load("daichi");
    let close = state_of(&def, "Landslide Point Blank");
    let far = state_of(&def, "Landslide Heel");
    let mut near = arena(&def, 40.0);
    assert_eq!(reached(&mut near, quarter_back(k::A), &[close, far]), Some(close));
    let mut apart = arena(&def, 160.0);
    assert_eq!(reached(&mut apart, quarter_back(k::A), &[close, far]), Some(far));
}

#[test]
fn the_same_input_picks_the_anti_air_version_only_against_a_jumping_enemy() {
    let def = load("bruno");
    let catch = state_of(&def, "Bull Catch");
    let rush = state_of(&def, "Bulldozer Rush");
    let mut grounded = arena(&def, 80.0);
    assert_eq!(reached(&mut grounded, quarter_back(k::A), &[catch, rush]), Some(rush));
    let mut airborne = arena(&def, 80.0);
    enter(&mut airborne, 1, 40);
    for _ in 0..14 { airborne.tick(&[0, 0]); }
    assert_eq!(airborne.fighters[1].state_type, 'A', "the enemy never left the ground");
    assert_eq!(reached(&mut airborne, quarter_back(k::A), &[catch, rush]), Some(catch));
}
