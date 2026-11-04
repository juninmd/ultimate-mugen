use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;

mod support;
use support::*;

const STRIKER: &str = "
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
damage = 40
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
";

fn simul_one_versus_two() -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(mugen_char("striker", STRIKER), 0, 0, 1);
    m.add_player(mugen_char("dummy", ""), 1, 0, 2);
    m.add_player(mugen_char("dummy", ""), 1, 0, 3);
    m.team_simul = true;
    m.start_round();
    for _ in 0..600 {
        if m.round_state == 2 { break; }
        m.tick(&[0, 0]);
    }
    m
}

fn strike(m: &mut Match, target: usize) -> f32 {
    for (k, f) in m.fighters.iter_mut().enumerate() {
        let x = if k == 0 { 0.0 } else if k == target { 55.0 } else { 900.0 };
        f.pos.x = x;
    }
    m.fighters[0].facing = 1.0;
    m.fighters[target].facing = -1.0;
    let before = m.fighters[target].life;
    let d = m.fighters[0].def.clone();
    m.change_state(0, 200, d);
    for _ in 0..40 {
        m.tick(&[0, 0]);
        if m.fighters[target].life < before { break; }
    }
    before - m.fighters[target].life
}

#[test]
fn combo_damage_belongs_to_the_defender_being_hit() {
    let mut m = simul_one_versus_two();
    let first = strike(&mut m, 1);
    assert!(first > 0.0, "the first hit landed");
    let second = strike(&mut m, 2);
    assert!(second > 0.0, "the hit on the other defender landed");
    assert_eq!(m.fighters[0].combo_target, Some(m.fighters[2].id));
    assert!((m.fighters[0].combo_damage - second).abs() < 0.001, "combo damage shows only the current defender: {} vs {}", m.fighters[0].combo_damage, second);
    assert!((m.fighters[0].combo_damage - (first + second)).abs() > 0.001, "damage from the previous defender is not carried over");
}

#[test]
fn combo_accumulator_clears_after_the_window() {
    let mut m = simul_one_versus_two();
    let first = strike(&mut m, 1);
    assert!((m.fighters[0].combo_damage - first).abs() < 0.001);
    for _ in 0..(ume_engine::matchs::COMBO_WINDOW + 2) { m.tick(&[0, 0]); }
    assert_eq!(m.fighters[0].combo_count, 0);
    assert_eq!(m.fighters[0].combo_damage, 0.0);
    assert_eq!(m.fighters[0].combo_target, None);
}

#[test]
fn stats_score_and_round_time_follow_the_match() {
    let stats = "
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
damage = 40
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
";
    let mut m = duel(mugen_char("a", stats), mugen_char("b", ""));
    face_off(&mut m, 55.0);
    enter(&mut m, 0, 200);
    for _ in 0..40 { m.tick(&[0, 0]); if m.fighters[1].received_hits > 0 { break; } }
    assert_eq!(m.fighters[1].received_hits, 1);
    assert!(m.fighters[1].received_damage > 0.0);
    assert_eq!(m.first_attack, Some(0));
    let ev = |m: &Match, i: usize, e: &str| m.ev(i, &ume_engine::expr::Expr::new(ume_engine::expr::parse(e).unwrap()));
    assert!(ev(&m, 0, "FirstAttack") == 1.0);
    assert!(ev(&m, 0, "Round") > 0.0);
    assert!(ev(&m, 0, "TimeElapsed") >= 0.0);

    let scorer = "
[Statedef 450]
type = S
physics = S

[State 450, score]
type = ScoreAdd
trigger1 = Time = 0
value = 25

[State 450, timer]
type = RoundTimeSet
trigger1 = Time = 0
value = 30
";
    let mut m = duel(mugen_char("a", scorer), mugen_char("b", ""));
    enter(&mut m, 0, 450);
    m.tick(&[0, 0]);
    assert!((m.fighters[0].score - 25.0).abs() < 1e-6, "score {}", m.fighters[0].score);
    assert_eq!(m.timer, 30 * 60);
    assert!((ev(&m, 0, "Score") - 25.0).abs() < 1e-6);
    assert!((ev(&m, 0, "TimeRemaining") - 30.0).abs() < 1e-6, "remaining {}", ev(&m, 0, "TimeRemaining"));
}

#[test]
fn input_override_controllers_act_deterministically() {
    let asserted = "
[Statedef 500]
type = S
physics = S

[State 500, cmd]
type = AssertCommand
trigger1 = Time = 0
value = hadou
time = 5

[State 500, input]
type = AssertInput
trigger1 = Time = 0
value = 32
time = 3

[State 500, shift]
type = ShiftInput
trigger1 = Time = 0
value = 2
";
    let mut m = duel(mugen_char("a", asserted), mugen_char("b", ""));
    enter(&mut m, 0, 500);
    m.tick(&[0, 0]);
    assert!(m.has_command(0, "hadou"), "AssertCommand makes the command visible");
    let t1 = m.fighters[0].history.tick;
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].history.tick - t1, 3, "ShiftInput pushes two extra history ticks before the current one");
    assert!(m.fighters[0].history.at(m.fighters[0].history.tick) & 32 != 0, "AssertInput injects the button bits");
    for _ in 0..6 { m.tick(&[0, 0]); }
    assert!(!m.has_command(0, "hadou"), "the assertion expires after its time");
}

#[test]
fn hitdef_life_parameters_feed_the_life_systems() {
    let striker = "
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
damage = 40
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
redlife = 50
dizzypoints = 60
guardpoints = 60
score = 10
sparkscale = 2
unhittabletime = 8
";
    let mut m = duel(mugen_char("a", striker), mugen_char("b", ""));
    m.fighters[1].ikemen.dizzy_points = 5.0;
    m.fighters[1].ikemen.guard_points = 5.0;
    face_off(&mut m, 55.0);
    enter(&mut m, 0, 200);
    for _ in 0..40 { m.tick(&[0, 0]); if m.fighters[1].received_hits > 0 { break; } }
    let d = &m.fighters[1];
    assert_eq!(d.received_hits, 1);
    assert!(d.ikemen.red_life > d.life, "red life {} > life {}", d.ikemen.red_life, d.life);
    assert!(d.ikemen.dizzy, "the dizzy meter was drained");
    assert!(d.unhittable > 0, "unhittabletime grants invincibility ticks");
    assert!((m.fighters[0].score - 10.0).abs() < 1e-6, "attacker score {}", m.fighters[0].score);
}

#[test]
fn sprite_sctrls_register_overrides() {
    let cns = "
[Statedef 510]
type = S
physics = S

[State 510, remap]
type = RemapSprite
trigger1 = Time = 0
from = 0, 0
to = 9000, 1
time = 3

[State 510, transform]
type = TransformSprite
trigger1 = Time = 0
scale = 2
angle = 90
time = 2
";
    let mut m = duel(mugen_char("a", cns), mugen_char("b", ""));
    enter(&mut m, 0, 510);
    m.tick(&[0, 0]);
    let remap = m.fighters[0].sprite_remap.first().copied().expect("a remap is registered");
    assert_eq!((remap.0, remap.1), ((0, 0), (9000, 1)));
    assert!(remap.2 > 0);
    let (scale, angle, _, life) = m.fighters[0].sprite_transform.expect("a transform is registered");
    assert!((scale - 2.0).abs() < 1e-6 && (angle - 90.0).abs() < 1e-6 && life > 0);
    for _ in 0..6 { m.tick(&[0, 0]); }
    assert!(m.fighters[0].sprite_remap.is_empty(), "the remap expires");
    assert!(m.fighters[0].sprite_transform.is_none(), "the transform expires");
}

#[test]
fn playsnd_flags_are_exposed_for_stop_on_hit_and_state_change() {
    let cns = "
[Statedef 400]
type = S
physics = S

[State 400, snd]
type = PlaySnd
trigger1 = Time = 0
value = 0, 0
channel = 3
loop = 1
stopongethit = 1
stoponchangestate = 1
";
    let mut m = duel(mugen_char("a", cns), mugen_char("b", ""));
    enter(&mut m, 0, 400);
    m.tick(&[0, 0]);
    let flagged = m.events.iter().any(|e| matches!(e, ume_engine::matchs::Event::Sound { channel: 3, stop_on_get_hit: true, stop_on_change_state: true, .. }));
    assert!(flagged, "PlaySnd carries the stop-on-hit and stop-on-state flags");
    let changed = m.events.iter().any(|e| matches!(e, ume_engine::matchs::Event::StateChange { .. }));
    assert!(changed, "entering a state emits StateChange so flagged channels can stop");
}

#[test]
fn env_shake_dir_restricts_the_axis() {
    for (dir, horizontal, vertical) in [(0u8, true, true), (1, true, false), (2, false, true)] {
        let shaker = format!("
[Statedef 900]
type = S
physics = S

[State 900, shake]
type = EnvShake
trigger1 = Time = 0
time = 20
freq = 60
ampl = -8
dir = {dir}
");
        let mut m = duel(mugen_char("a", &shaker), mugen_char("b", ""));
        enter(&mut m, 0, 900);
        m.tick(&[0, 0]);
        assert!(m.shake_time > 0, "dir {dir}: shaking");
        assert_eq!(m.shake_dir, dir);
        assert_eq!(m.shake_horizontal(), horizontal, "dir {dir} horizontal");
        assert_eq!(m.shake_vertical(), vertical, "dir {dir} vertical");
    }
}

#[test]
fn classic_pushbox_pushes_only_the_defender() {
    let mut classic = duel(mugen_char("a", ""), mugen_char("b", ""));
    classic.fighters[0].pos.x = 0.0;
    classic.fighters[1].pos.x = 20.0;
    classic.fighters[0].state_type = 'A';
    classic.fighters[1].state_type = 'S';
    let (a0, b0) = (classic.fighters[0].pos.x, classic.fighters[1].pos.x);
    classic.tick(&[0, 0]);
    let (da, db) = (classic.fighters[0].pos.x - a0, classic.fighters[1].pos.x - b0);
    assert!(da < -0.5, "the attacker is pushed back in classic mode: {da}");
    assert!(db.abs() < 0.5, "the defender does not move in classic mode: {db}");

    let mut modern = duel(mugen_char("a", ""), mugen_char("b", ""));
    modern.classic_pushbox = false;
    modern.fighters[0].pos.x = 0.0;
    modern.fighters[1].pos.x = 20.0;
    modern.fighters[0].state_type = 'A';
    modern.fighters[1].state_type = 'S';
    let (a0, b0) = (modern.fighters[0].pos.x, modern.fighters[1].pos.x);
    modern.tick(&[0, 0]);
    let (da, db) = (modern.fighters[0].pos.x - a0, modern.fighters[1].pos.x - b0);
    assert!(da < -0.5 && db > 0.5, "modern mode splits the push: {da} / {db}");
}

#[test]
fn projectiles_keep_their_teamside_after_the_owner_turns() {
    let shooter = "
[Statedef 1000]
type = S
movetype = A
physics = S
anim = 0

[State 1000, p]
type = Projectile
trigger1 = Time = 0
projanim = 1000
attr = S, NP
damage = 10
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
velocity = 4
";
    let mut m = duel(mugen_char("a", shooter), mugen_char("b", ""));
    face_off(&mut m, 120.0);
    enter(&mut m, 0, 1000);
    m.tick(&[0, 0]);
    assert_eq!(m.projectiles.len(), 1);
    let before = m.projectiles[0].teamside;
    m.fighters[0].facing = -1.0;
    m.fighters[0].side = 1;
    m.tick(&[0, 0]);
    assert_eq!(m.projectiles[0].teamside, before, "teamside is captured at spawn");
}
