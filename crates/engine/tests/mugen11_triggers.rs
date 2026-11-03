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
