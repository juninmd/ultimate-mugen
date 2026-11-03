mod support;
use support::*;
use ume_engine::fighter::HitDefData;

fn jab(extra: &str) -> String {
    format!("
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
{extra}

[State 200, end]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
")
}

fn land(extra: &str, gap: f32) -> ume_engine::matchs::Match {
    let mut m = duel(mugen_char("p1", &jab(extra)), mugen_char("p2", ""));
    face_off(&mut m, gap);
    enter(&mut m, 0, 200);
    ticks(&mut m, 4);
    m
}

#[test]
fn chainid_and_nochainid_gate_follow_ups() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    enter(&mut m, 1, 5000);
    m.fighters[1].ghv.hit_id = 7;
    let base = HitDefData { hitflag: "MAF".into(), attr_state: "S".into(), ..Default::default() };
    assert!(m.can_be_hit(1, &base, 0));
    assert!(m.can_be_hit(1, &HitDefData { chain_id: Some(7), ..base.clone() }, 0));
    assert!(!m.can_be_hit(1, &HitDefData { chain_id: Some(3), ..base.clone() }, 0));
    assert!(!m.can_be_hit(1, &HitDefData { no_chain_id: [Some(1), Some(7)], ..base.clone() }, 0));
    assert!(!m.can_be_hit(1, &base, 1), "teammates are immune by default");
    assert!(m.can_be_hit(1, &HitDefData { affect_team: 'F', ..base.clone() }, 1));
    assert!(m.can_be_hit(1, &HitDefData { affect_team: 'B', ..base.clone() }, 1));
    assert!(!m.can_be_hit(1, &HitDefData { affect_team: 'F', ..base }, 0));
}

#[test]
fn hit_records_hit_id_and_deactivates_mugen_hitdef() {
    let m = land("id = 11\nchainid = -1", 40.0);
    assert_eq!(m.fighters[1].ghv.hit_id, 11);
    assert_eq!(m.fighters[1].move_type, 'H');
    assert!(!m.fighters[0].hitdef_active, "a MUGEN HitDef stops after it connects");
}

#[test]
fn snap_and_distance_limits_place_the_target() {
    let m = land("snap = 25, -10", 40.0);
    let (a, d) = (&m.fighters[0], &m.fighters[1]);
    assert!(((d.pos.x - a.pos.x) - 25.0).abs() < 6.0, "snap x: {}", d.pos.x - a.pos.x);
    let m = land("mindist = 55", 30.0);
    assert!(m.fighters[1].pos.x - m.fighters[0].pos.x >= 54.0);
    let m = land("maxdist = 20", 55.0);
    assert!(m.fighters[1].pos.x - m.fighters[0].pos.x <= 28.0);
}

#[test]
fn target_palfx_and_env_shake() {
    let m = land("palfx.time = 20\npalfx.add = 100, 0, 0\nenvshake.time = 12\nenvshake.ampl = -6\nfall.envshake.time = 9", 40.0);
    let p = m.fighters[1].palfx.as_ref().expect("hit PalFX applied to p2");
    assert_eq!(p.add[0], 100.0);
    assert!(m.shake_time > 0);
    assert_eq!(m.fighters[1].ghv.fall_env_shake.time, 9);
}

#[test]
fn fallenvshake_controller_consumes_the_stored_shake() {
    let cns = "
[Statedef 600]
type = L
movetype = H
physics = N

[State 600, s]
type = FallEnvShake
trigger1 = Time = 0
";
    let mut m = duel(mugen_char("a", ""), mugen_char("b", cns));
    m.fighters[1].ghv.fall_env_shake = ume_engine::fighter::Shake { time: 15, freq: 60.0, ampl: -4.0, phase: 0.0 };
    enter(&mut m, 1, 600);
    m.tick(&[0, 0]);
    assert!(m.shake_time > 0);
    assert_eq!(m.fighters[1].ghv.fall_env_shake.time, 0);
}

#[test]
fn mugen_corner_push_uses_cornerpush_veloff() {
    let mut m = duel(mugen_char("p1", &jab("ground.cornerpush.veloff = -7")), mugen_char("p2", ""));
    m.stage.bound_left = 0.0;
    m.stage.bound_right = 0.0;
    m.camera.x = 0.0;
    m.fighters[1].pos.x = m.view_w / 2.0 - 16.0;
    m.fighters[0].pos.x = m.fighters[1].pos.x - 40.0;
    let x0 = m.fighters[0].pos.x;
    enter(&mut m, 0, 200);
    ticks(&mut m, 8);
    assert!(m.fighters[1].move_type == 'H');
    assert!(m.fighters[0].pos.x < x0 - 3.0, "attacker pushed off the cornered target: {} -> {}", x0, m.fighters[0].pos.x);
}

#[test]
fn down_velocity_applies_to_lying_targets() {
    let mut m = duel(mugen_char("p1", &jab("down.velocity = -2, -5\ndown.hittime = 33")), mugen_char("p2", ""));
    face_off(&mut m, 40.0);
    m.fighters[1].state_type = 'L';
    m.fighters[1].move_type = 'H';
    enter(&mut m, 0, 200);
    ticks(&mut m, 4);
    assert_eq!(m.fighters[1].ghv.hittime, 33);
    assert!((m.fighters[1].ghv.yvel + 5.0).abs() < 1e-3);
}

const REVERSAL: &str = "
[Statedef 400]
type = S
movetype = I
physics = S
anim = 300
ctrl = 0

[State 400, r]
type = ReversalDef
trigger1 = Time = 0
reversal.attr = SCA, NA
pausetime = 5, 7
p1stateno = 410
p2stateno = 420

[Statedef 410]
type = S
movetype = A
physics = S

[Statedef 420]
type = S
movetype = H
physics = S
";

#[test]
fn reversaldef_catches_matching_attacks() {
    let mut m = duel(mugen_char("rev", REVERSAL), mugen_char("atk", &jab("")));
    face_off(&mut m, 40.0);
    m.fighters[0].facing = 1.0;
    enter(&mut m, 0, 400);
    enter(&mut m, 1, 200);
    let life = m.fighters[0].life;
    ticks(&mut m, 6);
    assert_eq!(m.fighters[0].state_no, 410);
    assert_eq!(m.fighters[1].state_no, 420);
    assert_eq!(m.fighters[1].move_reversed > 0, true);
    assert_eq!(m.fighters[0].life, life, "the reversed attack deals no damage");
}

#[test]
fn hitoverride_redirects_the_hit_without_damage() {
    let cns = "
[Statedef 0]
type = S
movetype = I
physics = S
anim = 0

[State 0, o]
type = HitOverride
trigger1 = 1
attr = SCA, AA
stateno = 700
time = 1

[Statedef 700]
type = S
movetype = H
physics = S
";
    let mut m = duel(mugen_char("p1", &jab("")), mugen_char("parry", cns));
    face_off(&mut m, 40.0);
    let life = m.fighters[1].life;
    enter(&mut m, 0, 200);
    ticks(&mut m, 5);
    assert_eq!(m.fighters[1].state_no, 700);
    assert_eq!(m.fighters[1].life, life);
    assert!(m.fighters[0].move_hit > 0);
}
