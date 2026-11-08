mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar, PassiveSpec, RageSpec, StackSpec};
use ume_engine::passives::{RageRule, TOTAL_ATTACK_CAP};

fn native(id: &str) -> NativeChar {
    serde_json::from_str(&std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap())
        .unwrap()
}

fn with_passives(id: &str, passives: PassiveSpec) -> Arc<CharDef> {
    let mut nc = native(id);
    nc.passives = Some(passives);
    Arc::new(compile_native(&nc, 1, false))
}

fn duel(a: Arc<CharDef>, b: Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a, 0, 0, 1);
    m.add_player(b, 1, 0, 2);
    m.start_round();
    m
}

fn at_life_ratio(m: &mut Match, i: usize, ratio: f32) {
    let f = &mut m.fighters[i];
    f.life = f.life_max * ratio;
}

fn cast(m: &mut Match, i: usize, state: i32) {
    m.fighters[i].state_no = state;
    m.fighters[i].tick_passive(true);
}

#[test]
fn rage_curve_is_smooth_and_hits_four_times_at_one_percent() {
    let rule = RageSpec::default().rule();
    let at = |ratio: f32| rule.multiplier(ratio);
    assert_eq!(at(1.0), 1.0);
    assert!((at(0.5) - 2.515).abs() < 0.01, "{}", at(0.5));
    assert!((at(0.1) - 3.727).abs() < 0.01, "{}", at(0.1));
    assert!((at(0.01) - 4.0).abs() < 1e-4);
    assert!(at(0.0) <= 4.0 + 1e-4);
    let mut previous = at(1.0);
    for step in 1..=99 {
        let next = at(1.0 - step as f32 / 100.0);
        assert!(next > previous && next - previous < 0.1, "step {step}");
        previous = next;
    }
}

#[test]
fn rage_parameters_shape_the_curve() {
    let late = RageSpec {
        exponent: Some(2.0),
        start_ratio: Some(0.5),
        max_multiplier: Some(3.0),
        full_ratio: None,
    }
    .rule();
    assert_eq!(late.multiplier(0.8), 1.0);
    assert_eq!(late.multiplier(0.5), 1.0);
    assert!(late.multiplier(0.25) < 1.0 + 2.0 * 0.5);
    assert!((late.multiplier(0.01) - 3.0).abs() < 1e-4);
    let clamped = RageSpec {
        max_multiplier: Some(99.0),
        ..Default::default()
    }
    .rule();
    assert_eq!(
        clamped,
        RageRule {
            max_multiplier: 6.0,
            ..clamped
        }
    );
}

#[test]
fn bruno_scales_damage_with_missing_life() {
    let mut m = duel(
        Arc::new(compile_native(&native("bruno"), 1, false)),
        Arc::new(compile_native(&native("sylene"), 1, false)),
    );
    let base = m.fighters[0].attack_mul;
    assert_eq!(m.fighters[0].effective_attack(), base);
    for (ratio, expected) in [(0.5, 2.515), (0.1, 3.727), (0.01, 4.0)] {
        at_life_ratio(&mut m, 0, ratio);
        let got = m.fighters[0].effective_attack() / base;
        assert!((got - expected).abs() < 0.01, "ratio {ratio}: {got}");
    }
    assert_eq!(m.fighters[1].effective_attack(), m.fighters[1].attack_mul);
}

#[test]
fn stacks_grow_per_special_cap_and_decay() {
    let mut m = duel(
        Arc::new(compile_native(&native("sylene"), 1, false)),
        Arc::new(compile_native(&native("bruno"), 1, false)),
    );
    let base = m.fighters[0].attack_mul;
    cast(&mut m, 0, 200);
    assert_eq!(m.fighters[0].passive.stacks, 0, "normals do not stack");
    cast(&mut m, 0, 1001);
    cast(&mut m, 0, 3001);
    cast(&mut m, 0, 21002);
    assert_eq!(m.fighters[0].passive.stacks, 3);
    assert!((m.fighters[0].effective_attack() / base - 1.21).abs() < 1e-4);
    assert!(m.fighters[0].effective_defence() > 1.0 && m.fighters[0].passive_speed() > 1.0);
    for _ in 0..20 {
        cast(&mut m, 0, 1003);
        cast(&mut m, 0, 1004);
    }
    assert_eq!(m.fighters[0].passive.stacks, 10);
    for _ in 0..420 {
        m.fighters[0].tick_passive(true);
    }
    assert_eq!(m.fighters[0].passive.stacks, 9);
}

#[test]
fn stacks_reset_each_round_unless_configured() {
    let keep = StackSpec {
        reset_on_round: Some(false),
        ..Default::default()
    };
    let mut m = duel(
        with_passives(
            "kaito",
            PassiveSpec {
                stacks: Some(keep),
                ..Default::default()
            },
        ),
        with_passives(
            "kaito",
            PassiveSpec {
                stacks: Some(StackSpec::default()),
                ..Default::default()
            },
        ),
    );
    cast(&mut m, 0, 1001);
    cast(&mut m, 1, 1001);
    m.fighters[0].reset_passive_for_round();
    m.fighters[1].reset_passive_for_round();
    assert_eq!(
        (m.fighters[0].passive.stacks, m.fighters[1].passive.stacks),
        (1, 0)
    );
}

#[test]
fn passives_compose_multiplicatively_under_a_global_cap() {
    let both = PassiveSpec {
        rage: Some(RageSpec::default()),
        stacks: Some(StackSpec {
            damage_gain: Some(0.5),
            max_stacks: Some(10),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut m = duel(
        with_passives("kaito", both),
        with_passives("kaito", PassiveSpec::default()),
    );
    let base = m.fighters[0].attack_mul;
    at_life_ratio(&mut m, 0, 0.01);
    cast(&mut m, 0, 1001);
    let single = m.fighters[0].effective_attack() / base;
    assert!((single - 4.0 * 1.5).abs() < 1e-3);
    for _ in 0..10 {
        cast(&mut m, 0, 1002);
        cast(&mut m, 0, 1001);
    }
    assert!(m.fighters[0].effective_attack() <= TOTAL_ATTACK_CAP + 1e-4);
    m.fighters[0].attack_mul = 1.3;
    at_life_ratio(&mut m, 0, 1.0);
    assert!(
        (m.fighters[0].effective_attack() - 1.3 * m.fighters[0].stack_damage_multiplier()).abs()
            < 1e-4
    );
}

#[test]
fn mugen_characters_and_unopted_natives_are_untouched() {
    let mugen = mugen_char("plain", "");
    let mut m = duel(mugen, Arc::new(compile_native(&native("kaito"), 1, false)));
    m.fighters[0].attack_mul = 12.0;
    at_life_ratio(&mut m, 0, 0.01);
    cast(&mut m, 0, 1001);
    for i in 0..2 {
        let f = &m.fighters[i];
        assert_eq!(f.passive.stacks, 0);
        assert_eq!(f.effective_attack(), f.attack_mul);
        assert_eq!(f.effective_defence(), f.defence_mul * f.super_defence_mul);
        assert_eq!(f.passive_speed(), 1.0);
        assert!(f.passive_view().is_none());
    }
}

#[test]
fn bruno_hits_harder_when_nearly_dead() {
    let bruno = Arc::new(compile_native(&native("bruno"), 1, false));
    let dummy = Arc::new(compile_native(&native("kaito"), 1, false));
    let mut healthy = duel(bruno.clone(), dummy.clone());
    let mut dying = duel(bruno, dummy);
    at_life_ratio(&mut dying, 0, 0.01);
    for m in [&mut healthy, &mut dying] {
        m.fighters[0].pos.x = -20.0;
        m.fighters[1].pos.x = 20.0;
        m.fighters[1].life = m.fighters[1].life_max;
    }
    assert!(dying.fighters[0].effective_attack() > 3.9 * healthy.fighters[0].effective_attack());
}

#[test]
fn bruno_and_sylene_declare_the_passives() {
    let bruno = compile_native(&native("bruno"), 1, false);
    let sylene = compile_native(&native("sylene"), 1, false);
    assert!(bruno.passives.rage.is_some() && bruno.passives.stacks.is_none());
    assert!(sylene.passives.stacks.is_some() && sylene.passives.rage.is_none());
    assert_eq!(bruno.passives.rage.unwrap().max_multiplier, 4.0);
}
