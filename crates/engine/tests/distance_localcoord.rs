mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

fn ev(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &Expr::new(parse(src).unwrap()))
}

fn hd_char() -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char("hd", "")).ok().unwrap();
    d.local_w = 1280.0;
    d.local_h = 720.0;
    d.constants.insert("size.ground.front".into(), 64.0);
    d.constants.insert("size.ground.back".into(), 60.0);
    Arc::new(d)
}

fn mixed_duel() -> Match {
    let mut m = duel(mugen_char("sd", ""), hd_char());
    m.fighters[0].facing = 1.0;
    m.fighters[1].facing = -1.0;
    m.fighters[0].pos.x = 0.0;
    m
}

#[test]
fn body_dist_is_zero_when_hd_and_sd_boxes_touch() {
    let mut m = mixed_duel();
    m.fighters[1].pos.x = m.fighters[0].front_width() + m.fighters[1].front_width();
    assert!((ev(&m, 0, "P2BodyDist X")).abs() < 1e-4);
    assert!((ev(&m, 1, "P2BodyDist X")).abs() < 1e-4);
}

#[test]
fn body_dist_reports_the_gap_in_each_fighters_own_localcoord() {
    let mut m = mixed_duel();
    m.fighters[1].pos.x = m.fighters[0].front_width() + m.fighters[1].front_width() + 8.0;
    assert!((ev(&m, 0, "P2BodyDist X") - 8.0).abs() < 1e-4);
    assert!((ev(&m, 1, "P2BodyDist X") - 32.0).abs() < 1e-4);
}

#[test]
fn p2dist_converts_each_side_to_its_own_localcoord() {
    let mut m = mixed_duel();
    m.fighters[1].pos.x = 40.0;
    assert!((ev(&m, 0, "P2Dist X") - 40.0).abs() < 1e-4);
    assert!((ev(&m, 1, "P2Dist X") - 160.0).abs() < 1e-4);
    assert!((ev(&m, 1, "P2Dist Y")).abs() < 1e-4);
}

#[test]
fn p2dist_ignores_the_camera() {
    let mut m = mixed_duel();
    m.fighters[1].pos.x = 40.0;
    let before = (
        ev(&m, 0, "P2Dist X"),
        ev(&m, 0, "P2BodyDist X"),
        ev(&m, 1, "P2Dist X"),
    );
    m.camera.x += 90.0;
    m.camera.y -= 30.0;
    assert_eq!(
        (
            ev(&m, 0, "P2Dist X"),
            ev(&m, 0, "P2BodyDist X"),
            ev(&m, 1, "P2Dist X")
        ),
        before
    );
}

#[test]
fn p2dist_inside_a_throw_state_uses_only_the_two_positions() {
    let mut m = duel(
        mugen_char("thrower", "[Statedef 800]\ntype = S\n"),
        hd_char(),
    );
    m.fighters[0].pos.x = 0.0;
    m.fighters[1].pos.x = 24.0;
    enter(&mut m, 0, 800);
    m.camera.x += 120.0;
    assert!((ev(&m, 0, "P2Dist X") - 24.0).abs() < 1e-4);
}

#[test]
fn screen_position_is_in_the_fighters_localcoord() {
    let mut m = mixed_duel();
    m.fighters[0].pos.x = m.camera.x + 20.0;
    m.fighters[1].pos.x = m.camera.x + 20.0;
    let sd = ev(&m, 0, "ScreenPos X");
    let hd = ev(&m, 1, "ScreenPos X");
    assert!((sd - (20.0 + m.view_w as f64 / 2.0)).abs() < 1e-3);
    assert!((hd - sd * 4.0).abs() < 1e-3);
}

#[test]
fn edges_follow_a_widescreen_view_in_the_fighters_localcoord() {
    let mut m = mixed_duel();
    m.view_w = 560.0;
    let half = 280.0;
    assert!((ev(&m, 0, "LeftEdge") - (m.camera.x as f64 - half)).abs() < 1e-3);
    assert!((ev(&m, 0, "RightEdge") - (m.camera.x as f64 + half)).abs() < 1e-3);
    assert!((ev(&m, 1, "RightEdge") - (m.camera.x as f64 + half) * 4.0).abs() < 1e-3);
    m.fighters[0].pos.x = m.camera.x + 100.0;
    m.fighters[0].facing = 1.0;
    assert!((ev(&m, 0, "FrontEdgeDist") - (half - 100.0)).abs() < 1e-3);
    assert!((ev(&m, 0, "BackEdgeDist") - (half + 100.0)).abs() < 1e-3);
    m.fighters[0].facing = -1.0;
    assert!((ev(&m, 0, "FrontEdgeDist") - (half + 100.0)).abs() < 1e-3);
}
