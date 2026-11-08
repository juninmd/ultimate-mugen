mod support;
use support::*;
use ume_engine::expr::{parse, Expr};
use ume_engine::fighter::{Bind, BindKind};
use ume_engine::matchs::Match;

const ATTACKER: &str = "
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
damage = 30, 5
priority = 5, Hit
hitflag = MAF
guardflag = MA
pausetime = 0, 0
ground.velocity = -4
ground.hittime = 12
fall = 1
fall.recover = 1
fall.recovertime = 10
getpower = 40, 20
givepower = 15, 7
dizzypoints = 25
guardpoints = 12
redlife = 18, 6
score = 3, 1
down.recover = 0
p1stateno = 210
p2stateno = 5100

[Statedef 210]
type = S
movetype = I
physics = S
anim = 0
ctrl = 1

[Statedef 5100]
type = S
movetype = H
physics = S
anim = 5000
ctrl = 0
";

fn ghv(m: &Match, i: usize, k: &str) -> f64 {
    m.ev(i, &Expr::new(parse(&format!("GetHitVar({k})")).unwrap()))
}

fn after_hit() -> Match {
    let mut m = duel(mugen_char("p1", ATTACKER), mugen_char("p2", ""));
    face_off(&mut m, 40.0);
    enter(&mut m, 0, 200);
    ticks(&mut m, 4);
    assert_eq!(m.fighters[1].state_no, 5100);
    m
}

#[test]
fn every_parameter_reports_the_hitdef_after_a_hit() {
    let m = after_hit();
    let attacker_id = m.fighters[0].id as f64;
    let table: [(&str, f64); 30] = [
        ("damage", 30.0),
        ("hitdamage", 30.0),
        ("guarddamage", 5.0),
        ("priority", 5.0),
        ("facing", 1.0),
        ("power", 15.0),
        ("hitpower", 40.0),
        ("guardpower", 20.0),
        ("attr", 65.0),
        ("guardflag", 7.0),
        ("dizzypoints", 25.0),
        ("guardpoints", 12.0),
        ("redlife", 18.0),
        ("score", 3.0),
        ("teamside", 1.0),
        ("projid", -1.0),
        ("playerid", attacker_id),
        ("id", attacker_id),
        ("p1stateno", 210.0),
        ("p2stateno", 5100.0),
        ("down.recover", 0.0),
        ("fall", 1.0),
        ("fall.recover", 1.0),
        ("fall.recovertime", 10.0),
        ("guarded", 0.0),
        ("hitcount", 1.0),
        ("guardcount", 0.0),
        ("hittime", 12.0),
        ("isbound", 0.0),
        ("playerno", m.fighters[0].player_no as f64),
    ];
    for (name, want) in table {
        assert_eq!(ghv(&m, 1, name), want, "GetHitVar({name})");
    }
}

#[test]
fn recover_time_counts_down_from_the_hitdef_value() {
    let mut m = after_hit();
    let elapsed = m.fighters[1].fall_time as f64;
    assert_eq!(ghv(&m, 1, "recovertime"), 10.0 - elapsed);
    ticks(&mut m, 20);
    assert_eq!(ghv(&m, 1, "recovertime"), 0.0);
}

#[test]
fn recover_time_is_zero_without_a_fall() {
    let mut m = duel(
        mugen_char("p1", &ATTACKER.replace("fall = 1", "fall = 0")),
        mugen_char("p2", ""),
    );
    face_off(&mut m, 40.0);
    enter(&mut m, 0, 200);
    ticks(&mut m, 4);
    assert_eq!(ghv(&m, 1, "recovertime"), 0.0);
}

#[test]
fn isbound_follows_the_live_bind() {
    let mut m = after_hit();
    m.fighters[1].bind = Some(Bind {
        to: m.fighters[0].id,
        time: 5,
        x: 0.0,
        y: 0.0,
        kind: BindKind::Target,
    });
    assert_eq!(ghv(&m, 1, "isbound"), 1.0);
}

#[test]
fn unknown_parameters_evaluate_to_zero() {
    let m = after_hit();
    assert_eq!(ghv(&m, 1, "not.a.real.param"), 0.0);
    assert_eq!(ghv(&m, 1, "not.a.real.param"), 0.0);
}

#[test]
fn defaults_before_any_hit_are_neutral() {
    let m = duel(mugen_char("a", ""), mugen_char("b", ""));
    for name in [
        "hitdamage",
        "priority",
        "attr",
        "score",
        "teamside",
        "power",
    ] {
        assert_eq!(ghv(&m, 0, name), 0.0, "GetHitVar({name})");
    }
    assert_eq!(ghv(&m, 0, "down.recover"), 1.0);
    assert_eq!(ghv(&m, 0, "p1stateno"), -1.0);
}
