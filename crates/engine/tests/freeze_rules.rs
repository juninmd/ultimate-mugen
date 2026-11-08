mod support;
use support::*;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

const CNS: &str = "
[Statedef 200]
type = S
movetype = A
physics = N
anim = 200
ctrl = 0

[State 200, tick]
type = VarAdd
trigger1 = 1
v = 0
value = 1

[State 200, tick even in pause]
type = VarAdd
trigger1 = 1
ignorehitpause = 1
v = 1
value = 1

[Statedef 210]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 210, explods]
type = Explod
trigger1 = Time = 0
id = 1
anim = 1000
removetime = -1
ignorehitpause = 0

[State 210, explods free]
type = Explod
trigger1 = Time = 0
id = 2
anim = 1000
removetime = -1
ignorehitpause = 1

[State 210, explods timed]
type = Explod
trigger1 = Time = 0
id = 3
anim = 1000
removetime = -1
pausemovetime = 4
supermovetime = 4

[Statedef 220]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 220, helper]
type = Helper
trigger1 = Time = 0
id = 7
stateno = 230
pausemovetime = 5
preserve = 1

[State 220, plain helper]
type = Helper
trigger1 = Time = 0
id = 8
stateno = 230

[Statedef 230]
type = S
movetype = I
physics = N
anim = 200
ctrl = 0

[State 230, drift]
type = PosAdd
trigger1 = 1
x = 1

[Statedef 240]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 240, pause]
type = Pause
trigger1 = Time = 0
time = 20
movetime = 3

[Statedef 250]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 250, super]
type = SuperPause
trigger1 = Time = 0
time = 20
movetime = 3
anim = -1
";

const JAB: &str = "
[Statedef 300]
type = S
movetype = A
physics = N
anim = 200
ctrl = 0

[State 300, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NA
damage = 30
hitflag = MAFD
guardflag = MA
pausetime = 8, 8
ground.velocity = -4
ground.hittime = 20
";

fn ev(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &Expr::new(parse(src).unwrap()))
}

fn setup() -> Match {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", CNS));
    face_off(&mut m, 200.0);
    m
}

fn anim_signature(m: &Match, i: usize) -> (i32, usize, i32, i32) {
    let f = &m.fighters[i];
    (f.anim_elapsed, f.elem, f.elem_time, f.state_time)
}

#[test]
fn hitpause_freezes_animation_state_time_and_anim_triggers() {
    let mut m = setup();
    enter(&mut m, 0, 200);
    ticks(&mut m, 2);
    m.fighters[0].hit_pause = 4;
    let frozen = anim_signature(&m, 0);
    let anim_time = ev(&m, 0, "AnimTime");
    let elem_time = ev(&m, 0, "AnimElemTime(1)");
    for _ in 0..3 {
        m.tick(&[0, 0]);
        assert_eq!(anim_signature(&m, 0), frozen);
        assert_eq!(ev(&m, 0, "AnimTime"), anim_time);
        assert_eq!(ev(&m, 0, "AnimElemTime(1)"), elem_time);
    }
    assert_eq!(m.fighters[0].hit_pause, 1);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].hit_pause, 0);
    m.tick(&[0, 0]);
    assert_eq!(
        m.fighters[0].anim_elapsed,
        frozen.0 + 1,
        "animation resumes after the pause"
    );
    assert_eq!(m.fighters[0].state_time, frozen.3 + 1);
}

#[test]
fn hitshake_freezes_the_defender_the_same_way() {
    let mut m = setup();
    enter(&mut m, 0, 200);
    ticks(&mut m, 2);
    m.fighters[0].hit_shake = 3;
    let frozen = anim_signature(&m, 0);
    ticks(&mut m, 3);
    assert_eq!(anim_signature(&m, 0), frozen);
    assert_eq!(m.fighters[0].hit_shake, 0);
    ticks(&mut m, 1);
    assert_eq!(m.fighters[0].anim_elapsed, frozen.0 + 1);
}

#[test]
fn only_ignorehitpause_controllers_run_during_hitpause() {
    let mut m = setup();
    enter(&mut m, 0, 200);
    ticks(&mut m, 1);
    let (plain, always) = (m.fighters[0].vars[0], m.fighters[0].vars[1]);
    m.fighters[0].hit_pause = 3;
    ticks(&mut m, 3);
    assert_eq!(
        m.fighters[0].vars[0], plain,
        "regular controllers wait for the pause to end"
    );
    assert_eq!(
        m.fighters[0].vars[1],
        always + 3,
        "ignorehitpause controllers keep running"
    );
}

#[test]
fn hitpause_time_counts_down_for_the_attacker_and_hitshake_for_the_defender() {
    let mut m = duel(mugen_char("a", JAB), mugen_char("b", ""));
    face_off(&mut m, 40.0);
    enter(&mut m, 0, 300);
    for _ in 0..12 {
        m.tick(&[0, 0]);
        if m.fighters[1].move_type == 'H' {
            break;
        }
    }
    assert_eq!(m.fighters[1].move_type, 'H');
    assert_eq!(ev(&m, 0, "HitPauseTime"), 8.0);
    assert_eq!(
        ev(&m, 1, "HitPauseTime"),
        0.0,
        "the defender is shaken, not paused"
    );
    assert_eq!(ev(&m, 1, "GetHitVar(hitshaketime)"), 8.0);
    assert_eq!(ev(&m, 1, "HitShakeOver"), 0.0);
    m.tick(&[0, 0]);
    assert_eq!(ev(&m, 0, "HitPauseTime"), 7.0);
    assert_eq!(ev(&m, 1, "GetHitVar(hitshaketime)"), 7.0);
}

#[test]
fn gethitvars_are_stable_while_the_defender_is_in_hitshake() {
    let mut m = duel(mugen_char("a", JAB), mugen_char("b", ""));
    face_off(&mut m, 40.0);
    enter(&mut m, 0, 300);
    for _ in 0..12 {
        m.tick(&[0, 0]);
        if m.fighters[1].move_type == 'H' {
            break;
        }
    }
    let names = [
        "damage",
        "hittime",
        "slidetime",
        "ctrltime",
        "xvel",
        "yvel",
        "yaccel",
        "type",
        "animtype",
        "groundtype",
        "fall",
        "hitcount",
        "guarded",
        "id",
        "fall.time",
    ];
    let sample = |m: &Match| {
        names
            .iter()
            .map(|n| ev(m, 1, &format!("GetHitVar({n})")))
            .collect::<Vec<_>>()
    };
    let first = sample(&m);
    assert_eq!(first[0], 30.0);
    assert_eq!(first[1], 20.0);
    let shake = m.fighters[1].hit_shake;
    assert!(shake >= 4);
    for _ in 0..shake - 1 {
        m.tick(&[0, 0]);
        assert_eq!(
            sample(&m),
            first,
            "no hit variable moves while the shake runs"
        );
    }
    assert_eq!(
        m.fighters[1].hit_over_time, 20,
        "the recovery clock does not run during the shake"
    );
}

fn explod(m: &Match, owner: usize, id: i32) -> &ume_engine::matchs::Explod {
    m.explods
        .iter()
        .find(|e| e.owner == m.fighters[owner].id && e.id == id)
        .expect("explod")
}

#[test]
fn explod_ignorehitpause_zero_waits_for_owner_hitpause_and_hitshake() {
    let mut m = setup();
    enter(&mut m, 0, 210);
    ticks(&mut m, 1);
    for field in ["hit_pause", "hit_shake"] {
        let (waits, free) = (explod(&m, 0, 1).time, explod(&m, 0, 2).time);
        if field == "hit_pause" {
            m.fighters[0].hit_pause = 3;
        } else {
            m.fighters[0].hit_shake = 3;
        }
        ticks(&mut m, 2);
        assert_eq!(
            explod(&m, 0, 1).time,
            waits,
            "{field}: ignorehitpause = 0 freezes"
        );
        assert_eq!(
            explod(&m, 0, 2).time,
            free + 2,
            "{field}: ignorehitpause = 1 keeps going"
        );
        ticks(&mut m, 1);
        let resumed = explod(&m, 0, 1).time;
        ticks(&mut m, 1);
        assert_eq!(
            explod(&m, 0, 1).time,
            resumed + 1,
            "{field}: resumes once the owner is free"
        );
    }
}

#[test]
fn frozen_explod_stays_bound_to_its_owner() {
    let mut m = setup();
    enter(&mut m, 0, 210);
    ticks(&mut m, 1);
    m.fighters[0].hit_pause = 6;
    let before = explod(&m, 0, 1).pos;
    ticks(&mut m, 2);
    assert_eq!(explod(&m, 0, 1).pos.x, before.x);
    assert_eq!(explod(&m, 0, 1).pos.y, before.y);
}

#[test]
fn superpause_and_pause_freeze_explods_unless_they_carry_a_move_budget() {
    for state in [250, 240] {
        let mut m = setup();
        enter(&mut m, 0, 210);
        ticks(&mut m, 1);
        enter(&mut m, 1, state);
        m.tick(&[0, 0]);
        assert!(m.pause.is_some());
        let (idle, budget) = (explod(&m, 0, 1).time, explod(&m, 0, 3).time);
        ticks(&mut m, 8);
        assert_eq!(
            explod(&m, 0, 1).time,
            idle,
            "state {state}: no budget means frozen"
        );
        assert_eq!(
            explod(&m, 0, 3).time,
            budget + 4,
            "state {state}: the budget allows exactly four moving ticks"
        );
    }
}

#[test]
fn pausemovetime_helper_moves_through_pause_but_not_through_superpause() {
    let mut m = setup();
    enter(&mut m, 0, 220);
    ticks(&mut m, 2);
    let helpers: Vec<usize> = (0..m.fighters.len())
        .filter(|&i| m.fighters[i].is_helper)
        .collect();
    assert_eq!(helpers.len(), 2);
    let moving = *helpers
        .iter()
        .find(|&&i| m.fighters[i].helper_id == 7)
        .unwrap();
    let plain = *helpers
        .iter()
        .find(|&&i| m.fighters[i].helper_id == 8)
        .unwrap();
    enter(&mut m, 1, 240);
    m.tick(&[0, 0]);
    let (mx, px) = (m.fighters[moving].pos.x, m.fighters[plain].pos.x);
    ticks(&mut m, 8);
    assert_eq!(
        m.fighters[moving].pos.x,
        mx + 5.0,
        "pausemovetime = 5 allows exactly five moving ticks"
    );
    assert_eq!(
        m.fighters[plain].pos.x, px,
        "a helper without a budget freezes"
    );
}

#[test]
fn pauser_moves_for_movetime_and_everyone_else_waits() {
    let mut m = setup();
    enter(&mut m, 1, 240);
    m.tick(&[0, 0]);
    let (a, b) = (m.fighters[0].state_time, m.fighters[1].state_time);
    ticks(&mut m, 6);
    assert_eq!(
        m.fighters[0].state_time, a,
        "the opponent is frozen for the whole pause"
    );
    assert_eq!(
        m.fighters[1].state_time,
        b + 3,
        "the pauser moves for movetime ticks"
    );
}

#[test]
fn pausemovetime_does_not_apply_to_superpause() {
    let mut m = setup();
    enter(&mut m, 0, 220);
    ticks(&mut m, 2);
    let moving = (0..m.fighters.len())
        .find(|&i| m.fighters[i].is_helper && m.fighters[i].helper_id == 7)
        .unwrap();
    enter(&mut m, 1, 250);
    m.tick(&[0, 0]);
    let x = m.fighters[moving].pos.x;
    ticks(&mut m, 8);
    assert_eq!(
        m.fighters[moving].pos.x, x,
        "only supermovetime keeps a helper moving in a super pause"
    );
}
