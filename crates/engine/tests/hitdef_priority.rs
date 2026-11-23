use crate::support;
use support::*;
use ume_engine::matchs::Match;

fn state(priority: &str) -> String {
    format!(
        r#"
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
damage = 30
hitflag = MAFD
guardflag = MA
pausetime = 0, 0
ground.velocity = -4
ground.hittime = 12
priority = {priority}

[State 200, end]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
"#
    )
}

fn clash(p1: &str, p2: &str) -> Match {
    let mut m = duel(mugen_char("p1", &state(p1)), mugen_char("p2", &state(p2)));
    face_off(&mut m, 24.0);
    enter(&mut m, 0, 200);
    enter(&mut m, 1, 200);
    ticks(&mut m, 4);
    m
}

#[test]
fn a_hit_beats_a_miss() {
    let m = clash("7, Miss", "4, Hit");
    assert!(m.fighters[0].life < m.fighters[0].life_max, "the Hit hits the Miss user");
    assert_eq!(m.fighters[1].life, m.fighters[1].life_max, "the Miss does not hit");
}

#[test]
fn a_dodge_makes_both_attacks_miss() {
    let m = clash("7, Dodge", "4, Hit");
    assert_eq!(m.fighters[0].life, m.fighters[0].life_max);
    assert_eq!(m.fighters[1].life, m.fighters[1].life_max);
}

#[test]
fn the_higher_priority_hit_wins_the_trade() {
    let m = clash("7, Hit", "4, Hit");
    assert_eq!(m.fighters[0].life, m.fighters[0].life_max, "the low-priority hit is cancelled");
    assert!(m.fighters[1].life < m.fighters[1].life_max);
}

#[test]
fn equal_priority_hits_trade() {
    let m = clash("4, Hit", "4, Hit");
    assert!(m.fighters[0].life < m.fighters[0].life_max);
    assert!(m.fighters[1].life < m.fighters[1].life_max);
}
