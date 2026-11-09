mod support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k::*;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const JUMP_FALL_STATE: i32 = 50;
const HEIGHT: f32 = -70.0;
const KIT: [&str; 16] = ["kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian", "garan", "marlow", "kaname", "jin", "daichi", "valdric", "haruki", "oboro"];

fn load(id: &str) -> NativeChar {
    let path = format!("{}/../../assets/chars/{id}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn air_attack(id: &str, input: u32) -> (i32, bool) {
    let def = Arc::new(compile_native(&load(id), 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    face_off(&mut m, 50.0);
    m.fighters[0].pos.y = HEIGHT;
    enter(&mut m, 0, JUMP_FALL_STATE);
    m.fighters[0].ctrl = true;
    let life = m.fighters[1].life;
    let mut attacked = 0;
    for _ in 0..4 { m.tick(&[input, 0]); if m.fighters[0].move_type == 'A' && m.fighters[0].state_type == 'A' { attacked = m.fighters[0].state_no; } }
    for _ in 0..30 { m.tick(&[0, 0]); }
    (attacked, m.fighters[1].life < life)
}

#[test]
fn every_button_attacks_in_the_air_and_down_kick_is_a_separate_move() {
    for id in KIT {
        let mut states = std::collections::HashSet::new();
        for input in [X, Y, A, B, D | B] {
            let (state, _) = air_attack(id, input);
            assert!(state != 0, "{id} input {input:#x} did not start an air attack");
            states.insert(state);
        }
        assert_eq!(states.len(), 5, "{id} air attacks must be distinct: {states:?}");
    }
}

#[test]
fn the_down_kick_lands_on_a_close_opponent() {
    for id in KIT {
        assert!(air_attack(id, D | B).1, "{id} down kick did not hit");
    }
}
