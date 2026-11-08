mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::parse_mugen_version;
use ume_engine::expr::{parse, truthy};

fn ev(m: &ume_engine::matchs::Match, i: usize, src: &str) -> f64 { m.ev(i, &ume_engine::expr::Expr::new(parse(src).unwrap())) }

#[test]
fn win_and_lose_kinds() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.fighters[1].life = 0.0;
    ticks(&mut m, 2);
    assert_eq!(m.winner_side, 0);
    assert_eq!(ev(&m, 0, "WinKO"), 1.0);
    assert_eq!(ev(&m, 0, "WinPerfect"), 1.0);
    assert_eq!(ev(&m, 0, "WinTime"), 0.0);
    assert_eq!(ev(&m, 1, "LoseKO"), 1.0);
    assert_eq!(ev(&m, 1, "LoseTime"), 0.0);

    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.fighters[0].life -= 10.0;
    m.fighters[1].life -= 300.0;
    m.timer = 1;
    ticks(&mut m, 3);
    assert_eq!(m.winner_side, 0);
    assert_eq!(ev(&m, 0, "WinTime"), 1.0);
    assert_eq!(ev(&m, 0, "WinPerfect"), 0.0);
    assert_eq!(ev(&m, 1, "LoseTime"), 1.0);
    assert_eq!(ev(&m, 1, "LoseKO"), 0.0);
}

#[test]
fn mugenversion_from_def() {
    assert_eq!(parse_mugen_version(Some("1.1")), 1.1);
    assert_eq!(parse_mugen_version(Some("1.0")), 1.0);
    assert_eq!(parse_mugen_version(Some("04,14,2001")), 0.0);
    assert_eq!(parse_mugen_version(None), 1.0);
    let m = duel(mugen_char("a", ""), mugen_char("b", ""));
    assert_eq!(ev(&m, 0, "MugenVersion"), 1.0);
}

#[test]
fn undefined_redirects_are_bottom() {
    let m = duel(mugen_char("a", ""), mugen_char("b", ""));
    assert!(!truthy(ev(&m, 0, "helper(5), time >= 0")));
    assert!(!truthy(ev(&m, 0, "helper(5), life != [0, 10]")));
    assert!(!truthy(ev(&m, 0, "!(parent, time)")));
    assert!(truthy(ev(&m, 0, "(parent, time) || 1")));
}

#[test]
fn hitdefattr_follows_movetype_and_wildcards() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.fighters[0].hitdef = Some(ume_engine::fighter::HitDefData { attr_state: "S".into(), attr_kind: vec!["SP".into()], ..Default::default() });
    m.fighters[0].move_type = 'A';
    assert_eq!(ev(&m, 0, "HitDefAttr = SC, AP"), 1.0);
    assert_eq!(ev(&m, 0, "HitDefAttr = S, NA, SP"), 1.0);
    assert_eq!(ev(&m, 0, "HitDefAttr = S, NA"), 0.0);
    assert_eq!(ev(&m, 0, "HitDefAttr = A, SP"), 0.0);
    assert_eq!(ev(&m, 0, "HitDefAttr = Q"), 0.0, "invalid attr strings are false, not errors");
    m.fighters[0].move_type = 'I';
    assert_eq!(ev(&m, 0, "HitDefAttr = SCA"), 0.0);
}

#[test]
fn command_through_redirect_reads_that_players_buffer() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.fighters[1].cmd.active.insert("x".into());
    assert_eq!(ev(&m, 0, "p2, command = \"x\""), 1.0);
    assert_eq!(ev(&m, 0, "command = \"x\""), 0.0);
}

#[test]
fn projcanceltime_tracks_cancelled_projectiles() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    assert_eq!(ev(&m, 0, "ProjCancelTime(0)"), -1.0);
    let gt = m.game_time;
    m.fighters[0].proj_contacts.push(('x', 3, gt));
    ticks(&mut m, 2);
    assert_eq!(ev(&m, 0, "ProjCancelTime(3)"), 2.0);
    assert_eq!(ev(&m, 0, "ProjContactTime(3)"), -1.0, "a cancel is not a contact");
}

const SPAWN_HELPER: &str = "
[Statedef 800]
type = S
physics = S

[State 800, h]
type = Helper
trigger1 = Time = 0
id = 5
stateno = 810

[Statedef 810]
type = S
physics = N
anim = 0
";

fn team_of_four() -> ume_engine::matchs::Match {
    use ume_engine::common::{common_def, flat_stage};
    let mut m = ume_engine::matchs::Match::new(common_def(), None, flat_stage());
    for (name, side) in [("a0", 0), ("b0", 1), ("a1", 0), ("b1", 1)] {
        m.add_player(mugen_char(name, SPAWN_HELPER), side, 0, 1);
    }
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0, 0, 0]); }
    m
}

#[test]
fn partner_resolves_teammate_in_two_versus_two() {
    let mut m = team_of_four();
    m.fighters[2].life = 321.0;
    m.fighters[2].vars[3] = 7;
    enter(&mut m, 2, 200);
    let anim = m.fighters[2].anim_no as f64;
    assert_eq!(ev(&m, 0, "partner, life"), 321.0);
    assert_eq!(ev(&m, 0, "partner, stateno"), 200.0);
    assert_eq!(ev(&m, 0, "partner, var(3)"), 7.0);
    assert_eq!(ev(&m, 0, "partner, anim"), anim);
    assert_eq!(ev(&m, 2, "partner, stateno"), 0.0, "the teammate sees the first player as its partner");
    assert_eq!(ev(&m, 1, "partner, life"), m.fighters[3].life as f64, "the other side has its own partner");
    assert!(ev(&m, 0, "partner(1), life").is_nan(), "a two-man team has no second partner");
}

#[test]
fn partner_skips_helpers_and_missing_teammates() {
    let mut m = duel(mugen_char("a", SPAWN_HELPER), mugen_char("b", ""));
    assert!(ev(&m, 0, "partner, life").is_nan(), "no teammate in a duel");
    enter(&mut m, 0, 800);
    ticks(&mut m, 3);
    let h = m.fighters.iter().position(|f| f.is_helper).expect("helper spawned");
    assert!(m.fighters[h].side == 0);
    assert!(ev(&m, 0, "partner, life").is_nan(), "a helper is never a partner");
    assert_eq!(ev(&m, 0, "NumPartner"), 0.0);

    let mut m = team_of_four();
    enter(&mut m, 0, 800);
    for _ in 0..3 { m.tick(&[0, 0, 0, 0]); }
    assert!(m.fighters.iter().any(|f| f.is_helper));
    m.fighters[2].life = 150.0;
    assert_eq!(ev(&m, 0, "partner, life"), 150.0, "the helper does not shadow the real teammate");
    let h = m.fighters.iter().position(|f| f.is_helper).unwrap();
    assert_eq!(ev(&m, h, "partner, life"), 150.0, "a helper redirects through its root");
}

#[test]
fn command_in_custom_state_uses_state_owner_commands_on_the_victim_inputs() {
    let mut owner = Arc::try_unwrap(mugen_char("a", "[Statedef 7000]\ntype = S\n")).ok().unwrap();
    owner.commands = vec![ume_engine::commands::parse_command("hadou", "N, x", 15, 1)];
    let owner = Arc::new(owner);
    let mut m = duel(owner.clone(), mugen_char("b", ""));
    m.change_state(1, 7000, owner.clone());
    m.tick(&[0, 0]);
    m.tick(&[0, ume_engine::input::k::X]);
    assert_eq!(ev(&m, 1, "command = \"hadou\""), 1.0);
    assert_eq!(ev(&m, 0, "command = \"hadou\""), 0.0);
    for _ in 0..20 { m.tick(&[0, ume_engine::input::k::R]); }
    m.tick(&[0, ume_engine::input::k::R | ume_engine::input::k::X]);
    assert_eq!(ev(&m, 1, "command = \"hadou\""), 0.0);
}

#[test]
fn command_outside_custom_state_ignores_foreign_commands() {
    let mut owner = Arc::try_unwrap(mugen_char("a", "")).ok().unwrap();
    owner.commands = vec![ume_engine::commands::parse_command("hadou", "x", 15, 1)];
    let mut m = duel(Arc::new(owner), mugen_char("b", ""));
    m.tick(&[0, ume_engine::input::k::X]);
    assert_eq!(ev(&m, 1, "command = \"hadou\""), 0.0);
}

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
attr = S, SP
damage = 30
priority = 5
pausetime = 4, 6
sparkxy = 7, -9
ground.velocity = -4, -2
ground.hittime = 12
p2stateno = 5100

[Statedef 5100]
type = S
movetype = H
physics = S
anim = 5000
ctrl = 0

[Statedef 5200]
type = C
movetype = I
physics = C
anim = 0
ctrl = 1
";

fn land_strike(defender: &str) -> ume_engine::matchs::Match {
    let mut m = duel(mugen_char("striker", STRIKER), mugen_char("dummy", defender));
    face_off(&mut m, 40.0);
    enter(&mut m, 0, 200);
    while m.fighters[0].move_hit_var.hit_time == 0 && m.game_time < 2000 { m.tick(&[0, 0]); }
    m
}

#[test]
fn hit_description_triggers_report_the_landed_hit() {
    let m = land_strike("");
    assert_eq!(m.fighters[1].state_no, 5100);
    let table: [(usize, &str, f64); 17] = [
        (0, "HitDefVar(p2stateno)", 5100.0), (0, "HitDefVar(p1stateno)", -1.0), (0, "HitDefVar(priority)", 5.0),
        (0, "HitDefVar(sparkx)", 7.0), (0, "HitDefVar(sparky)", -9.0), (0, "HitDefVar(pausetime.p1)", 4.0),
        (0, "HitDefVar(pausetime.p2)", 6.0), (0, "HitDefVar(ground.velocity.x)", -4.0), (0, "HitDefVar(ground.velocity.y)", -2.0),
        (0, "HitDefVar(attr)", 1.0 + 2048.0), (0, "HitDefVar(id)", 0.0),
        (0, "MoveHitVar(id)", m.fighters[1].id as f64), (0, "MoveHitVar(playerno)", m.fighters[1].player_no as f64),
        (0, "MoveHitVar(overridden)", 0.0), (0, "MoveHitVar(frame)", 1.0),
        (1, "HitVel X", m.fighters[1].ghv.xvel as f64), (1, "HitVel Y", -2.0),
    ];
    for (who, src, want) in table {
        assert_eq!(ev(&m, who, src), want, "{src}");
    }
}

#[test]
fn hit_by_attr_matches_the_attribute_of_the_last_hit() {
    let m = land_strike("");
    assert_eq!(ev(&m, 1, "HitByAttr = SCA, SP"), 1.0);
    assert_eq!(ev(&m, 1, "HitByAttr = S, AP"), 1.0);
    assert_eq!(ev(&m, 1, "HitByAttr = SCA, NA, HT"), 0.0);
    assert_eq!(ev(&m, 1, "HitByAttr = C, SP"), 0.0);
    assert_eq!(ev(&m, 1, "HitByAttr != SCA, NA"), 1.0);
    assert_eq!(ev(&m, 0, "HitByAttr = SCA, AA, AT, AP"), 0.0, "an attacker that was never hit matches nothing");
}

#[test]
fn move_frame_and_counter_flags_expire() {
    let mut m = land_strike("");
    assert_eq!(ev(&m, 0, "MoveHitVar(frame)"), 1.0);
    ticks(&mut m, 2);
    assert_eq!(ev(&m, 0, "MoveHitVar(frame)"), 0.0);
    assert_eq!(ev(&m, 0, "MoveCountered"), 0.0);

    let mut m = duel(mugen_char("striker", STRIKER), mugen_char("dummy", ""));
    face_off(&mut m, 40.0);
    m.fighters[1].move_type = 'A';
    enter(&mut m, 0, 200);
    while m.fighters[0].move_hit_var.hit_time == 0 && m.game_time < 2000 { m.tick(&[0, 0]); }
    assert!(ev(&m, 0, "MoveCountered") >= 1.0);
}

#[test]
fn hit_override_sets_hitoverridden_and_overridden_flags() {
    let mut m = duel(mugen_char("striker", STRIKER), mugen_char("dummy", ""));
    face_off(&mut m, 40.0);
    m.fighters[1].hit_overrides[0] = Some(ume_engine::fighter::HitOverride { attr: "SCA, AA, AT, AP".into(), state_no: 5000, time: 30, force_air: false, keep: false, hits: 1, threshold: 0.0, chip: 0.0 });
    enter(&mut m, 0, 200);
    while m.fighters[0].move_hit_var.hit_time == 0 && m.game_time < 2000 { m.tick(&[0, 0]); }
    assert_eq!(ev(&m, 1, "HitOverridden"), 1.0);
    assert_eq!(ev(&m, 0, "MoveHitVar(overridden)"), 1.0);
    assert_eq!(m.fighters[1].life, m.fighters[1].life_max);
    ticks(&mut m, 3);
    assert_eq!(ev(&m, 1, "HitOverridden"), 0.0);
}

#[test]
fn previous_state_triggers_remember_the_last_state() {
    let mut m = duel(mugen_char("a", STRIKER), mugen_char("b", ""));
    enter(&mut m, 0, 200);
    assert_eq!(ev(&m, 0, "PrevStateType = S"), 1.0);
    assert_eq!(ev(&m, 0, "PrevMoveType = I"), 1.0);
    enter(&mut m, 0, 5200);
    assert_eq!(ev(&m, 0, "PrevStateType = S"), 1.0);
    assert_eq!(ev(&m, 0, "PrevMoveType = A"), 1.0);
    assert_eq!(ev(&m, 0, "PrevMoveType != I"), 1.0);
    enter(&mut m, 0, 0);
    assert_eq!(ev(&m, 0, "PrevStateType = C"), 1.0);
    assert_eq!(ev(&m, 0, "PrevStateNo"), 5200.0);
}

#[test]
fn prev_anim_tracks_the_animation_before_the_last_change() {
    let mut m = duel(mugen_char("a", STRIKER), mugen_char("b", ""));
    enter(&mut m, 0, 200);
    assert_eq!(ev(&m, 0, "Anim"), 200.0);
    assert_eq!(ev(&m, 0, "PrevAnim"), 0.0);
    enter(&mut m, 0, 5100);
    assert_eq!(ev(&m, 0, "PrevAnim"), 200.0);
}

#[test]
fn state_ownership_and_existence_triggers() {
    let mut m = land_strike("");
    assert_eq!(ev(&m, 0, "SelfStateNoExist(200)"), 1.0);
    assert_eq!(ev(&m, 0, "SelfStateNoExist(777)"), 0.0);
    assert_eq!(ev(&m, 1, "SelfStateNoExist(5100)"), 0.0, "the dummy does not own state 5100");
    assert_eq!(ev(&m, 1, "InCustomState"), 1.0, "the target runs a state given by the attacker");
    assert_eq!(ev(&m, 0, "InCustomState"), 0.0);
    assert_eq!(ev(&m, 0, "InCustomAnim"), 0.0);
    m.fighters[0].state_no = 100;
    assert_eq!(ev(&m, 0, "InRunState"), 1.0);
    m.fighters[0].state_no = 0;
    assert_eq!(ev(&m, 0, "InRunState"), 0.0);
}

#[test]
fn is_asserted_shares_the_assert_special_table() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    assert_eq!(ev(&m, 0, "IsAsserted(NoWalk)"), 0.0);
    m.fighters[0].asserts |= ume_engine::fighter::asf::NOWALK;
    assert_eq!(ev(&m, 0, "IsAsserted(NoWalk)"), 1.0);
    assert_eq!(ev(&m, 0, "IsAsserted(\"nowalk\")"), 1.0);
    assert_eq!(ev(&m, 0, "IsAsserted(NoJump)"), 0.0);
    assert_eq!(ev(&m, 1, "IsAsserted(NoWalk)"), 0.0);
    assert_eq!(ev(&m, 0, "IsAsserted(Intro)"), 0.0);
    m.global_asserts |= ume_engine::flags::gsf::INTRO;
    assert_eq!(ev(&m, 0, "IsAsserted(Intro)"), 1.0);
    assert_eq!(ev(&m, 0, "IsAsserted(NotAFlag)"), 0.0);
}

#[test]
fn juggle_points_parent_and_air_jump_triggers() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    assert_eq!(ev(&m, 0, "JugglePoints"), 15.0);
    m.fighters[0].juggle_used = 6;
    assert_eq!(ev(&m, 0, "JugglePoints"), 9.0);
    m.fighters[0].juggle_used = 40;
    assert_eq!(ev(&m, 0, "JugglePoints"), 0.0);

    assert_eq!(ev(&m, 1, "ParentExist"), 0.0);
    m.fighters[1].parent = Some(m.fighters[0].id);
    assert_eq!(ev(&m, 1, "ParentExist"), 1.0);
    m.fighters[0].destroyed = true;
    assert_eq!(ev(&m, 1, "ParentExist"), 0.0);

    assert_eq!(ev(&m, 1, "AirJumpCount"), 0.0);
    m.fighters[1].air_jumps = 2;
    assert_eq!(ev(&m, 1, "AirJumpCount"), 2.0);
}
