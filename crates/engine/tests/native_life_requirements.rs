mod support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const BUTTON_Z: u32 = 512;
const LAST_RESORT_FIRST_STATE: i32 = 200;
const LAST_RESORT_END_STATE: i32 = 5000;

fn fighter_with_requirement(requires: serde_json::Value) -> Arc<ume_engine::chardef::CharDef> {
    let mut nc: NativeChar = serde_json::from_str(&std::fs::read_to_string("../../assets/chars/kaito.json").unwrap()).unwrap();
    let mut last_resort = nc.moves.iter().find(|m| m.id == "strong").unwrap().clone();
    last_resort.id = "last_resort".into();
    last_resort.name = "Last Resort".into();
    last_resort.input = "z".into();
    last_resort.cancels = None;
    last_resort.requires = Some(serde_json::from_value(requires).unwrap());
    nc.moves.push(last_resort);
    Arc::new(compile_native(&nc, 1, true))
}

fn arena(def: &Arc<ume_engine::chardef::CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    face_off(&mut m, 80.0);
    m
}

fn performs_last_resort(def: &Arc<ume_engine::chardef::CharDef>, own_ratio: f32, enemy_ratio: f32) -> bool {
    let mut m = arena(def);
    m.fighters[0].life = m.fighters[0].life_max * own_ratio;
    m.fighters[1].life = m.fighters[1].life_max * enemy_ratio;
    let mut reached = false;
    for step in 0..40 {
        m.tick(&[if step < 3 { BUTTON_Z } else { 0 }, 0]);
        reached |= (LAST_RESORT_FIRST_STATE..LAST_RESORT_END_STATE).contains(&m.fighters[0].state_no);
    }
    reached
}

#[test]
fn own_life_ceiling_gates_the_move() {
    let def = fighter_with_requirement(serde_json::json!({ "life": { "max": 0.5 } }));
    assert!(!performs_last_resort(&def, 1.0, 1.0));
    assert!(performs_last_resort(&def, 0.4, 1.0));
}

#[test]
fn enemy_life_ceiling_gates_the_move() {
    let def = fighter_with_requirement(serde_json::json!({ "enemyLife": { "max": 0.5 } }));
    assert!(!performs_last_resort(&def, 1.0, 1.0));
    assert!(performs_last_resort(&def, 1.0, 0.4));
}

#[test]
fn both_conditions_must_hold() {
    let def = fighter_with_requirement(serde_json::json!({ "life": { "max": 0.5 }, "enemyLife": { "max": 0.5 } }));
    assert!(!performs_last_resort(&def, 0.4, 1.0));
    assert!(!performs_last_resort(&def, 1.0, 0.4));
    assert!(performs_last_resort(&def, 0.4, 0.4));
}
