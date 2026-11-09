mod support;
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar, PassiveSpec, RageSpec, TimeSpec};
use ume_engine::passives::TOTAL_ATTACK_CAP;

fn native(id: &str) -> NativeChar {
    serde_json::from_str(&std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap())
        .unwrap()
}

fn compiled(id: &str, passives: Option<PassiveSpec>) -> Arc<CharDef> {
    let mut nc = native(id);
    if passives.is_some() {
        nc.passives = passives;
    }
    Arc::new(compile_native(&nc, 1, false))
}

fn duel(a: Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a, 0, 0, 1);
    m.add_player(compiled("kaito", None), 1, 0, 2);
    m.start_round();
    m
}

fn tick_for(m: &mut Match, seconds: f32) {
    for _ in 0..(seconds * 60.0) as i32 {
        m.fighters[0].tick_passive(true);
    }
}

#[test]
fn time_curve_is_flat_then_ramps_to_the_maximum() {
    let rule = TimeSpec {
        start_seconds: Some(10.0),
        ramp_seconds: Some(100.0),
        ..Default::default()
    }
    .rule();
    assert_eq!(rule.multiplier(0), 1.0);
    assert_eq!(rule.multiplier(600), 1.0);
    assert!((rule.multiplier(600 + 3000) - 3.0).abs() < 1e-4);
    assert!((rule.multiplier(600 + 6000) - 5.0).abs() < 1e-4);
    assert_eq!(rule.multiplier(1_000_000), 5.0);
    let mut previous = 1.0;
    for tick in (600..6600).step_by(60) {
        let next = rule.multiplier(tick);
        assert!(next >= previous);
        previous = next;
    }
}

#[test]
fn time_parameters_are_clamped_and_shaped() {
    let slow = TimeSpec {
        exponent: Some(2.0),
        ramp_seconds: Some(100.0),
        ..Default::default()
    }
    .rule();
    assert!((slow.multiplier(3000) - 2.0).abs() < 1e-4);
    let huge = TimeSpec {
        max_multiplier: Some(99.0),
        ..Default::default()
    }
    .rule();
    assert_eq!(huge.max_multiplier, 6.0);
}

#[test]
fn oni_reaches_five_times_damage_and_resets_each_round() {
    let mut m = duel(compiled("garan", None));
    let base = m.fighters[0].attack_mul;
    assert!(m.fighters[0].base_def.passives.time.is_some());
    assert_eq!(m.fighters[0].effective_attack(), base);
    tick_for(&mut m, 15.0);
    assert!((m.fighters[0].effective_attack() - base).abs() < 1e-4);
    tick_for(&mut m, 52.5);
    assert!((m.fighters[0].effective_attack() / base - 3.0).abs() < 0.01);
    tick_for(&mut m, 200.0);
    assert!((m.fighters[0].effective_attack() / base - 5.0).abs() < 1e-3);
    assert!(m.fighters[0].passive_view().unwrap().aura > 0.99);
    m.fighters[0].reset_passive_for_round();
    assert_eq!(m.fighters[0].effective_attack(), base);
}

#[test]
fn time_does_not_advance_outside_the_fight_or_when_dead() {
    let mut m = duel(compiled("garan", None));
    for _ in 0..6000 {
        m.fighters[0].tick_passive(false);
    }
    assert_eq!(m.fighters[0].passive.fight_ticks, 0);
    m.fighters[0].alive = false;
    tick_for(&mut m, 10.0);
    assert_eq!(m.fighters[0].passive.fight_ticks, 0);
}

#[test]
fn time_composes_with_rage_under_the_global_cap() {
    let both = PassiveSpec {
        rage: Some(RageSpec::default()),
        enrage_time: Some(TimeSpec::default()),
        ..Default::default()
    };
    let mut m = duel(compiled("kaito", Some(both)));
    let base = m.fighters[0].attack_mul;
    tick_for(&mut m, 120.0);
    assert!((m.fighters[0].effective_attack() / base - 5.0).abs() < 1e-3);
    m.fighters[0].life = m.fighters[0].life_max * 0.01;
    assert!(m.fighters[0].effective_attack() <= TOTAL_ATTACK_CAP + 1e-4);
    assert!(m.fighters[0].effective_attack() > 5.0 * base);
}

#[test]
fn five_times_survives_an_awakened_form_multiplier() {
    let mut m = duel(compiled("garan", None));
    m.fighters[0].attack_mul = 1.3 * 0.95;
    tick_for(&mut m, 200.0);
    assert!((m.fighters[0].effective_attack() - 5.0 * 1.3 * 0.95).abs() < 1e-3);
}
