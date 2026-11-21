use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k::*;
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};
use ume_engine::potential::{PotentialLevel, POTENTIAL_TICKS};

fn load(id: &str) -> NativeChar {
    serde_json::from_str(&std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap()).unwrap()
}

fn imported_duel() -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    let mut def = compile_native(&load("kaito"), 1, false);
    def.forms.clear();
    def.anims.remove(&196);
    m.add_player(Arc::new(def), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("shion"), 1, false)), 1, 0, 1);
    m
}

#[test]
fn imported_fighter_unleashes_potential_then_fury_once_per_round() {
    let mut m = imported_duel();
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    assert_eq!(m.fighters[0].form_count(), 0);
    let base_attack = m.fighters[0].attack_mul;
    let max = m.fighters[0].power_max;

    m.fighters[0].power = 0.0;
    for _ in 0..3 { m.tick(&[T, 0]); }
    assert_eq!(m.fighters[0].potential.level, PotentialLevel::Idle, "needs half a bar");

    m.fighters[0].power = max;
    for _ in 0..3 { m.tick(&[T, 0]); }
    let f = &m.fighters[0];
    assert_eq!(f.potential.level, PotentialLevel::Potential);
    assert!((f.power - max * 0.5).abs() < 10.0, "potential spends half a bar ({})", f.power);
    assert!(f.attack_mul > base_attack * 1.15, "potential boosts damage");

    m.fighters[0].power = max;
    for _ in 0..3 { m.tick(&[T, 0]); }
    let f = &m.fighters[0];
    assert_eq!(f.potential.level, PotentialLevel::Fury);
    assert!(f.power < 10.0, "fury spends a full bar ({})", f.power);
    assert!(f.attack_mul > base_attack * 1.35, "fury boosts damage more");

    m.fighters[0].power = max;
    for _ in 0..3 { m.tick(&[T, 0]); }
    assert_eq!(m.fighters[0].potential.level, PotentialLevel::Fury, "fury only once per round");

    for _ in 0..(POTENTIAL_TICKS as usize * 2) { m.tick(&[0, 0]); }
    let f = &m.fighters[0];
    assert_eq!(f.potential.level, PotentialLevel::Idle, "fury ends");
    assert!((f.attack_mul - base_attack).abs() < 1e-4, "stats return to base");

    m.fighters[0].power = max;
    for _ in 0..3 { m.tick(&[T, 0]); }
    assert_eq!(m.fighters[0].potential.level, PotentialLevel::Idle, "potential is once per round");
}

#[test]
fn gifts_respect_the_three_transformation_cap() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("shion"), 1, false)), 1, 0, 1);
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    let own = m.fighters[0].form_count();
    assert!(own >= 2, "kaito has own forms");
    m.fighters[0].form = own as u8;
    m.fighters[0].power = m.fighters[0].power_max;
    for _ in 0..3 { m.tick(&[T, 0]); }
    let expected = if own >= 3 { PotentialLevel::Idle } else { PotentialLevel::Potential };
    assert_eq!(m.fighters[0].potential.level, expected, "gifts fill up to three transformations");
}
