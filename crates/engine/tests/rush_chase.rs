mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const JAB_STATE: i32 = 200;
const AIR_HIT_STATE: i32 = 5030;

fn fighter(id: &str) -> Arc<CharDef> {
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, true))
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    m
}

fn juggle_setup() -> Match {
    let mut m = arena(&fighter("akio"));
    face_off(&mut m, 200.0);
    enter(&mut m, 1, AIR_HIT_STATE);
    m.fighters[1].pos.y = -90.0;
    enter(&mut m, 0, JAB_STATE);
    m.fighters[0].move_hit = 1;
    m.fighters[0].move_contact = 1;
    m.fighters[0].power = 1000.0;
    m
}

#[test]
fn double_tap_up_chases_a_juggled_enemy_for_meter() {
    use ume_engine::rush::{CHASE_COST, CHASE_GAP};
    let mut m = juggle_setup();
    let before = m.fighters[0].power;
    for bits in [k::U, 0, k::U, 0] { m.tick(&[bits, 0]); }
    let (me, foe) = (&m.fighters[0], &m.fighters[1]);
    assert!((me.pos.x - foe.pos.x).abs() < CHASE_GAP + 24.0, "still {} away", (me.pos.x - foe.pos.x).abs());
    assert!(me.pos.y < -20.0, "chaser stayed on the ground at {}", me.pos.y);
    assert!(me.power < before - CHASE_COST + 40.0 && me.power > before - CHASE_COST - 60.0, "power {}", me.power);
    assert!(me.ctrl, "no control after the chase");
}

#[test]
fn chase_needs_a_juggled_enemy_and_a_hit() {
    let mut grounded = juggle_setup();
    grounded.fighters[1].state_type = 'S';
    grounded.fighters[1].move_type = 'I';
    grounded.fighters[1].pos.y = 0.0;
    for bits in [k::U, 0, k::U, 0] { grounded.tick(&[bits, 0]); }
    assert!((grounded.fighters[0].pos.x - grounded.fighters[1].pos.x).abs() > 100.0, "chased a standing enemy");
    let mut whiff = juggle_setup();
    whiff.fighters[0].move_hit = 0;
    for bits in [k::U, 0, k::U, 0] { whiff.tick(&[bits, 0]); }
    assert!((whiff.fighters[0].pos.x - whiff.fighters[1].pos.x).abs() > 100.0, "chased without a hit");
}
