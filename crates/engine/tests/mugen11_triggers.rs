mod support;
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
