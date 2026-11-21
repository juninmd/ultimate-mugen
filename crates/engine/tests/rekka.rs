mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn load(id: &str) -> (NativeChar, Arc<CharDef>) {
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap()).unwrap();
    let def = Arc::new(compile_native(&nc, 1, true));
    (nc, def)
}

fn state_of(def: &CharDef, name: &str) -> i32 { def.movelist.iter().find(|m| m.name == name).unwrap().state }

fn arena(def: &Arc<CharDef>, gap: f32) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    face_off(&mut m, gap);
    m
}

fn reaches(m: &mut Match, from: i32, wait: usize, button: u32, target: i32) -> bool {
    enter(m, 0, from);
    for _ in 0..wait { m.tick(&[0, 0]); }
    for _ in 0..4 {
        m.tick(&[button, 0]);
        if m.fighters[0].state_no == target { return true; }
    }
    for _ in 0..4 {
        m.tick(&[0, 0]);
        if m.fighters[0].state_no == target { return true; }
    }
    false
}

#[test]
fn a_follow_up_chains_from_its_source_inside_the_window_even_on_whiff() {
    let (nc, def) = load("jin");
    let source = state_of(&def, "Gust Dash");
    let follow = state_of(&def, "Gust Crossup");
    assert!(nc.moves.iter().any(|m| m.id == "gust_cross" && m.rekka.is_some()));
    let mut m = arena(&def, 220.0);
    assert!(reaches(&mut m, source, 8, k::B, follow), "follow-up did not start inside its window");
    let mut early = arena(&def, 220.0);
    assert!(!reaches(&mut early, source, 0, k::B, follow), "follow-up started before its window opened");
    let mut late = arena(&def, 220.0);
    assert!(!reaches(&mut late, source, 40, k::B, follow), "follow-up started after its window closed");
}

#[test]
fn a_follow_up_never_starts_from_neutral() {
    for (id, name, button) in [("jin", "Gust Crossup", k::B), ("lian", "Silk Lift", k::Y), ("kaito", "Storm Finish", k::X)] {
        let (_, def) = load(id);
        let follow = state_of(&def, name);
        let mut m = arena(&def, 60.0);
        for _ in 0..10 { m.tick(&[button, 0]); assert_ne!(m.fighters[0].state_no, follow, "{id} {name} started from neutral"); }
    }
}

#[test]
fn contact_gated_follow_up_needs_the_source_to_connect() {
    let (_, def) = load("kaito");
    let source = state_of(&def, "Storm Rush");
    let follow = state_of(&def, "Storm Finish");
    let mut whiff = arena(&def, 220.0);
    assert!(!reaches(&mut whiff, source, 16, k::X, follow), "follow-up started without contact");
    let mut hit = arena(&def, 40.0);
    assert!(reaches(&mut hit, source, 16, k::X, follow), "follow-up did not start after contact");
}

#[test]
fn lian_silk_lift_follows_a_connected_flow() {
    let (_, def) = load("lian");
    let source = state_of(&def, "Flowing Silk Kicks");
    let follow = state_of(&def, "Silk Lift");
    let mut whiff = arena(&def, 220.0);
    assert!(!reaches(&mut whiff, source, 14, k::Y, follow));
    let mut hit = arena(&def, 40.0);
    assert!(reaches(&mut hit, source, 14, k::Y, follow), "lift did not follow a connected flow");
}
