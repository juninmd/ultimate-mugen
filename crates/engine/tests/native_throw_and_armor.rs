mod support;
use serde_json::json;
use std::sync::Arc;
use support::*;
use ume_engine::native::{compile_native, NativeChar};

fn fighter_with(moves: serde_json::Value, anims: serde_json::Value) -> NativeChar {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    value["id"] = json!("lab");
    value["moves"] = moves;
    for (name, anim) in anims.as_object().unwrap() {
        value["anims"][name] = anim.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn throw_fighter() -> NativeChar {
    fighter_with(
        json!([
            {"id": "toss", "name": "Toss", "input": "x+a", "kind": "normal", "anim": "toss", "throw": {"damage": 90, "bind": [30, 0], "launch": [-4, -6]}},
            {"id": "jab", "name": "Jab", "input": "x", "kind": "normal", "anim": "jab", "hits": [{"at": 2, "damage": 40, "level": "light"}]}
        ]),
        json!({"toss": {"frames": [
            {"pose": "jab1", "ticks": 3, "hit": {"part": "handF", "radius": 28}},
            {"pose": "jab1", "ticks": 8}, {"pose": "jab0", "ticks": 8}, {"pose": "idle0", "ticks": 6}]}}),
    )
}

#[test]
fn throw_grabs_damages_and_launches_without_sticking() {
    let def = Arc::new(compile_native(&throw_fighter(), 1, true));
    assert!(
        def.states.contains_key(&3800),
        "generated victim state exists"
    );
    let mut m = duel(def.clone(), def);
    face_off(&mut m, 36.0);
    let before = m.fighters[1].life;
    enter(&mut m, 0, 200);
    let mut launched = false;
    for _ in 0..200 {
        m.tick(&[0, 0]);
        if m.fighters[1].vel.y < -1.0 || m.fighters[1].state_type == 'A' {
            launched = true;
        }
    }
    let dealt = before - m.fighters[1].life;
    assert!(dealt >= 85.0, "throw damage applied once, got {dealt}");
    assert!(dealt <= 100.0, "throw damage not repeated, got {dealt}");
    assert!(launched, "victim was launched after the release frame");
    assert!(
        m.fighters[1].ctrl && m.fighters[1].state_no == 0,
        "victim recovered, state {}",
        m.fighters[1].state_no
    );
}

#[test]
fn throw_whiffs_when_the_target_is_out_of_reach() {
    let def = Arc::new(compile_native(&throw_fighter(), 1, true));
    let mut m = duel(def.clone(), def);
    face_off(&mut m, 200.0);
    let before = m.fighters[1].life;
    enter(&mut m, 0, 200);
    ticks(&mut m, 120);
    assert_eq!(m.fighters[1].life, before);
}

fn armor_fighter(hits: u32) -> NativeChar {
    fighter_with(
        json!([
            {"id": "bull", "name": "Bull", "input": "~D, DF, F, y", "kind": "special", "anim": "bull", "armor": {"hits": hits}, "hits": [{"at": 4, "damage": 60, "level": "heavy"}]},
            {"id": "jab", "name": "Jab", "input": "x", "kind": "normal", "anim": "jab", "hits": [{"at": 2, "damage": 40, "level": "light"}]}
        ]),
        json!({"bull": {"frames": [
            {"pose": "jab0", "ticks": 10}, {"pose": "jab0", "ticks": 10}, {"pose": "jab0", "ticks": 10},
            {"pose": "jab1", "ticks": 6, "hit": {"part": "handF", "radius": 24}}, {"pose": "idle0", "ticks": 10}]}}),
    )
}

fn armored_duel(hits: u32) -> ume_engine::matchs::Match {
    let armored = Arc::new(compile_native(&armor_fighter(hits), 1, true));
    let mut m = duel(armored.clone(), armored);
    face_off(&mut m, 36.0);
    enter(&mut m, 0, 1000);
    ticks(&mut m, 4);
    m
}

#[test]
fn armor_absorbs_a_hit_with_chip_damage_and_keeps_the_move() {
    let mut m = armored_duel(1);
    let before = m.fighters[0].life;
    enter(&mut m, 1, 200);
    ticks(&mut m, 6);
    let lost = before - m.fighters[0].life;
    assert!(
        lost > 0.0 && lost < 40.0 * 0.5,
        "chip damage only, lost {lost}"
    );
    assert_eq!(
        m.fighters[0].state_no, 1000,
        "armored move is not interrupted"
    );
}

#[test]
fn armor_breaks_after_its_hit_budget_is_spent() {
    let mut m = armored_duel(1);
    enter(&mut m, 1, 200);
    ticks(&mut m, 6);
    assert_eq!(m.fighters[0].state_no, 1000);
    ticks(&mut m, 4);
    enter(&mut m, 1, 200);
    ticks(&mut m, 8);
    assert_ne!(
        m.fighters[0].state_no, 1000,
        "second hit breaks the armor, state {}",
        m.fighters[0].state_no
    );
}
