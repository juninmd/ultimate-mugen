mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::fighter::{asf, HitDefData};
use ume_engine::matchs::Match;

fn with_constants(name: &str, constants: &[(&str, f64)]) -> Arc<CharDef> {
    let mut def = Arc::try_unwrap(mugen_char(name, "")).ok().unwrap();
    for (key, value) in constants {
        def.constants.insert(key.to_string(), *value);
    }
    Arc::new(def)
}

fn hit(damage: f32) -> HitDefData {
    HitDefData {
        attr_state: "S".into(), hitflag: "MAFD".into(), guardflag: "MA".into(), damage, guard_damage: 0.0,
        red_life: [damage * 0.75, 0.0], dizzy_points: damage * 1.8, guard_points: damage * 1.5,
        ground_hit_time: 12, guard_hit_time: 10, airjuggle: 100, kill: true, guard_kill: false, p1stateno: -1, p2stateno: -1, ..Default::default()
    }
}

fn pair(constants: &[(&str, f64)]) -> Match {
    duel(mugen_char("attacker", ""), with_constants("defender", constants))
}

#[test]
fn a_hit_raises_red_life_above_life() {
    let mut m = pair(&[]);
    m.apply_hit(0, 1, &hit(100.0), (0.0, 0.0), None);
    let defender = &m.fighters[1];
    assert_eq!(defender.life, 900.0);
    assert_eq!(defender.ikemen.red_life, 925.0);
    assert!(defender.ikemen.red_life > defender.life);
}

#[test]
fn red_life_never_falls_below_life_or_above_the_maximum() {
    let mut m = pair(&[]);
    let mut hard = hit(100.0);
    hard.red_life = [500.0, 0.0];
    m.apply_hit(0, 1, &hard, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].ikemen.red_life, 900.0);
    m.fighters[1].life = 1000.0;
    ticks(&mut m, 1);
    assert_eq!(m.fighters[1].ikemen.red_life, 1000.0);
    m.fighters[1].life = 0.0;
    ticks(&mut m, 1);
    assert_eq!(m.fighters[1].ikemen.red_life, 0.0);
}

#[test]
fn life_add_and_life_set_pull_red_life_along() {
    let cns = "
[Statedef 700]
type = S
physics = S

[State 700, heal]
type = LifeAdd
trigger1 = Time = 0
value = 50
absolute = 1
";
    let mut m = duel(mugen_char("attacker", ""), mugen_char("defender", cns));
    m.apply_hit(0, 1, &hit(100.0), (0.0, 0.0), None);
    enter(&mut m, 1, 700);
    ticks(&mut m, 1);
    assert_eq!(m.fighters[1].life, 950.0);
    assert_eq!(m.fighters[1].ikemen.red_life, 950.0);
}

#[test]
fn five_hits_make_the_defender_dizzy() {
    let mut m = pair(&[("data.dizzypoints", 100.0)]);
    let mut hd = hit(10.0);
    hd.dizzy_points = 20.0;
    for _ in 0..4 {
        m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
        assert!(!m.fighters[1].ikemen.dizzy);
    }
    assert_eq!(m.fighters[1].ikemen.dizzy_points, 20.0);
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    let defender = &m.fighters[1];
    assert_eq!(defender.ikemen.dizzy_points, defender.ikemen.dizzy_max);
    assert!(defender.ikemen.dizzy);
    assert_eq!(defender.state_no, 5300);
    assert!(!defender.ctrl);
    assert_eq!(m.ev(1, &ume_engine::chardef::safe_expr("Dizzy && !Ctrl")), 1.0);
}

#[test]
fn dizzy_recovers_after_its_state_and_restores_the_points() {
    let mut m = pair(&[("data.dizzypoints", 40.0)]);
    let mut hd = hit(10.0);
    hd.dizzy_points = 40.0;
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].state_no, 5300);
    ticks(&mut m, 170);
    let defender = &m.fighters[1];
    assert!(!defender.ikemen.dizzy);
    assert_eq!(defender.ikemen.dizzy_points, 40.0);
    assert_ne!(defender.state_no, 5300);
}

#[test]
fn five_guarded_hits_break_the_guard() {
    let mut m = pair(&[("data.guardpoints", 200.0)]);
    enter(&mut m, 1, 120);
    ticks(&mut m, 1);
    enter(&mut m, 1, 120);
    let mut hd = hit(5.0);
    hd.guard_damage = 1.0;
    hd.guard_points = 40.0;
    for _ in 0..4 {
        m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
        assert!(m.fighters[1].ghv.guarded);
        assert!(!m.fighters[1].ikemen.guard_break);
    }
    assert_eq!(m.fighters[1].ikemen.guard_points, 40.0);
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    let defender = &m.fighters[1];
    assert_eq!(defender.ikemen.guard_points, defender.ikemen.guard_max);
    assert!(defender.ikemen.guard_break);
    assert_eq!(defender.state_no, 5400);
    assert_eq!(m.ev(1, &ume_engine::chardef::safe_expr("GuardBreak")), 1.0);
}

#[test]
fn the_assert_flags_protect_each_system() {
    let mut m = pair(&[("data.dizzypoints", 100.0), ("data.guardpoints", 100.0)]);
    m.fighters[1].asserts |= asf::NODIZZYPOINTSDAMAGE | asf::NOREDLIFEDAMAGE;
    let mut hd = hit(10.0);
    hd.dizzy_points = 30.0;
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].ikemen.dizzy_points, 100.0);
    assert_eq!(m.fighters[1].ikemen.red_life, 1000.0);

    let mut m = pair(&[("data.guardpoints", 100.0)]);
    enter(&mut m, 1, 120);
    ticks(&mut m, 1);
    enter(&mut m, 1, 120);
    m.fighters[1].asserts |= asf::NOGUARDPOINTSDAMAGE;
    let mut guard = hit(5.0);
    guard.guard_points = 60.0;
    m.apply_hit(0, 1, &guard, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].ikemen.guard_points, 100.0);
}

#[test]
fn defence_scales_the_point_damage() {
    let mut m = pair(&[("data.dizzypoints", 100.0)]);
    m.fighters[1].defence_mul = 2.0;
    let mut hd = hit(10.0);
    hd.dizzy_points = 40.0;
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].ikemen.dizzy_points, 80.0);
}

#[test]
fn the_points_return_to_full_when_a_new_round_starts() {
    let mut m = pair(&[("data.dizzypoints", 100.0), ("data.guardpoints", 100.0)]);
    let mut hd = hit(10.0);
    hd.dizzy_points = 30.0;
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    m.fighters[1].ikemen.guard_points = 10.0;
    m.start_round();
    let defender = &m.fighters[1];
    assert_eq!(defender.ikemen.dizzy_points, 100.0);
    assert_eq!(defender.ikemen.guard_points, 100.0);
    assert_eq!(defender.ikemen.red_life, defender.life_max);
    assert!(!defender.ikemen.dizzy && !defender.ikemen.guard_break);
}

const PARAM_CNS: &str = "
[Statedef 200]
type = S
movetype = A
physics = S
anim = 200
ctrl = 0

[State 200, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NA
damage = 100, 20
hitflag = MAF
guardflag = MA
pausetime = 0, 0
EXTRA
";

fn built_hitdef(extra: &str, hyper: bool, constants: &[(&str, f64)]) -> HitDefData {
    let cns = PARAM_CNS.replace("EXTRA", extra).replace("S, NA", if hyper { "S, HA" } else { "S, NA" });
    let mut def = Arc::try_unwrap(mugen_char("p1", &cns)).ok().unwrap();
    for (key, value) in constants {
        def.constants.insert(key.to_string(), *value);
    }
    let mut m = duel(Arc::new(def), mugen_char("p2", ""));
    enter(&mut m, 0, 200);
    ticks(&mut m, 1);
    m.fighters[0].hitdef.clone().expect("HitDef active")
}

#[test]
fn hitdef_defaults_come_from_the_life_multipliers() {
    let hd = built_hitdef("", false, &[]);
    assert_eq!(hd.red_life, [75.0, 15.0]);
    assert_eq!(hd.dizzy_points, 180.0);
    assert_eq!(hd.guard_points, 150.0);
    let hyper = built_hitdef("", true, &[]);
    assert_eq!(hyper.dizzy_points, 0.0);
    assert_eq!(hyper.guard_points, -33.0);
}

#[test]
fn hitdef_parameters_override_the_multipliers() {
    let hd = built_hitdef("redlife = 40, 8\ndizzypoints = 12\nguardpoints = 7\nscore = 3, -1", false, &[]);
    assert_eq!(hd.red_life, [40.0, 8.0]);
    assert_eq!(hd.dizzy_points, 12.0);
    assert_eq!(hd.guard_points, 7.0);
    assert_eq!(hd.score, [3.0, -1.0]);
}

#[test]
fn constants_section_sets_the_multipliers() {
    let mut constants = std::collections::HashMap::new();
    let mut provided = std::collections::HashSet::new();
    ume_engine::cns::parse_constants("[Constants]\ndefault.lifetoredlifemul = 0.5\n", &mut constants, &mut provided);
    assert_eq!(constants.get("default.lifetoredlifemul"), Some(&0.5));
    let hd = built_hitdef("", false, &[("default.lifetoredlifemul", 0.5)]);
    assert_eq!(hd.red_life, [50.0, 10.0]);
}

#[test]
fn guard_points_regenerate_passively_during_neutral() {
    let mut m = pair(&[("data.guardpoints", 200.0)]);
    enter(&mut m, 1, 120); ticks(&mut m, 1); enter(&mut m, 1, 120);
    let mut hd = hit(5.0); hd.guard_points = 50.0;
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].ikemen.guard_points, 150.0);
    enter(&mut m, 1, 0); ticks(&mut m, 20);
    assert_eq!(m.fighters[1].ikemen.guard_points, 160.0);
}

