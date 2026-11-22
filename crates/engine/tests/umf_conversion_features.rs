mod support;
use serde_json::{json, Value};
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn kaito(edit: impl FnOnce(&mut Value)) -> Arc<CharDef> {
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string("../../assets/chars/kaito.json").unwrap()).unwrap();
    edit(&mut v);
    let nc: NativeChar = serde_json::from_value(v).unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    face_off(&mut m, 200.0);
    m
}

fn press(m: &mut Match, seq: &[u32]) {
    for &bits in seq {
        m.tick(&[bits, 0]);
    }
}

#[test]
fn charge_plays_the_original_start_loop_and_end_animations() {
    let def = kaito(|v| v["abilities"]["charge"] = json!({ "auraColor": [90, 70, 15], "anim": "powerUp", "startAnim": "crouchStart", "endAnim": "crouchEnd", "sound": [1, 0], "soundAt": [1], "startSound": [7, 0] }));
    let mut m = arena(&def);
    m.fighters[0].power = 0.0;
    press(&mut m, &[k::S; 2]);
    assert_eq!(m.fighters[0].state_no, 3920);
    assert_eq!(m.fighters[0].anim_no, 10, "the charge must open with the original start animation");
    press(&mut m, &[k::S; 60]);
    assert!(m.fighters[0].power > 0.0);
    press(&mut m, &[0; 2]);
    assert_eq!(m.fighters[0].state_no, 3921, "releasing must play the original end animation");
    press(&mut m, &[0; 40]);
    assert_eq!(m.fighters[0].state_no, 0);
    assert!(m.fighters[0].ctrl);
}

#[test]
fn teleport_ability_accepts_a_motion_input() {
    let def = kaito(|v| v["abilities"]["teleport"] = json!({ "input": "~B, D, B, x" }));
    let mut m = arena(&def);
    press(&mut m, &[k::L, k::L, 0, k::D, k::D, k::L, k::L, k::X, 0]);
    let entered = (0..20).any(|_| {
        m.tick(&[0, 0]);
        m.fighters[0].state_no == 3910
    });
    assert!(entered, "a MUGEN motion command must reach the teleport ability (state {})", m.fighters[0].state_no);
}

#[test]
fn follow_up_moves_without_input_are_not_listed_or_pressable() {
    let def = kaito(|v| {
        let moves = v["moves"].as_array_mut().unwrap();
        let mut fu = moves[0].clone();
        fu["id"] = json!("jab_followup");
        fu["input"] = json!("");
        moves.push(fu);
    });
    assert!(def.movelist.iter().all(|mv| mv.command != "m_jab_followup"));
    assert!(def.commands.iter().all(|c| c.name != "m_jab_followup"));
}

#[test]
fn fly_moves_start_from_flight_and_return_to_it() {
    let def = kaito(|v| {
        let moves = v["moves"].as_array_mut().unwrap();
        let mut fm = moves[0].clone();
        fm["id"] = json!("fly_jab");
        fm["input"] = json!("z");
        fm["when"] = json!("fly");
        moves.push(fm);
    });
    let sid = def.movelist.iter().find(|mv| mv.command == "m_fly_jab").unwrap().state;
    let mut m = arena(&def);
    m.fighters[0].power = 3000.0;
    press(&mut m, &[k::Z, 0]);
    assert_ne!(m.fighters[0].state_no, sid, "a fly move must not fire on the ground");
    press(&mut m, &[0; 30]);
    press(&mut m, &[k::B | k::C, k::B | k::C, 0]);
    press(&mut m, &[0; 20]);
    assert_eq!(m.fighters[0].state_no, 3900, "fighter should be flying");
    press(&mut m, &[k::Z, 0]);
    assert_eq!(m.fighters[0].state_no, sid);
    let y = m.fighters[0].pos.y;
    let back = (0..120).any(|_| {
        m.tick(&[0, 0]);
        m.fighters[0].state_no == 3900
    });
    assert!(back, "the fly move must return to flight");
    assert!(m.fighters[0].pos.y < 0.0 && (m.fighters[0].pos.y - y).abs() < 30.0, "a fly move hovers instead of falling");
}
