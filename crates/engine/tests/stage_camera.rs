mod support;
use std::sync::Arc;
use support::*;
use ume_engine::matchs::Match;
use ume_formats::stage::load_stage;
use ume_formats::vfs::Vfs;

fn field() -> Match {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.stage.vertical_follow = 1.0;
    m.stage.bound_high = -400.0;
    m.camera.y = 0.0;
    m.camera_target_y = 0.0;
    m
}

fn lift(m: &mut Match, y: f32) {
    m.fighters[0].pos.y = y;
    m.fighters[1].pos.y = 0.0;
}

fn load(def: &str) -> ume_formats::stage::MugenStage {
    let mut v = Vfs::new();
    v.insert("stages/c.def".into(), def.as_bytes().to_vec());
    load_stage(&v, "stages/c.def").unwrap()
}

#[test]
fn missing_camera_parameters_keep_the_previous_behaviour() {
    let st = load("[Camera]\nboundhigh = -40\n\n[BGdef]\n");
    assert_eq!(
        (
            st.vertical_follow_delta,
            st.y_scroll_speed,
            st.zoom_in_speed,
            st.zoom_out_speed
        ),
        (0.0, 0.3, 0.04, 0.04)
    );
    assert!(!st.lowest_cap && !st.dynamic_bound_high && st.auto_turn);
    assert_eq!(st.bound_high_delta, 0.0);
    let mut m = field();
    lift(&mut m, -30.0);
    m.update_camera();
    assert!(
        (m.camera.y + 9.0).abs() < 1e-4,
        "0.3 of the way to the target, no jump"
    );
}

#[test]
fn camera_parameters_are_parsed_and_clamped() {
    let st = load("[Camera]\nverticalfollowdelta = 12\nyscrollspeed = 2\nzoominspeed = -1\nzoomoutspeed = 0.5\nlowestcap = 1\ndynamicboundhigh = 1\nboundhighdelta = 30\n\n[BGdef]\n");
    assert_eq!(st.vertical_follow_delta, 12.0);
    assert_eq!(st.y_scroll_speed, 1.0);
    assert_eq!(st.zoom_in_speed, 0.0);
    assert_eq!(st.zoom_out_speed, 0.5);
    assert!(st.lowest_cap && st.dynamic_bound_high);
    assert_eq!(st.bound_high_delta, 30.0);
}

#[test]
fn vertical_follow_delta_ignores_small_height_changes() {
    let mut m = field();
    m.stage.camera.y_scroll_speed = 1.0;
    m.stage.camera.vertical_follow_delta = 10.0;
    lift(&mut m, -6.0);
    m.update_camera();
    assert_eq!(m.camera.y, 0.0, "below the delta the camera stays put");
    lift(&mut m, -25.0);
    m.update_camera();
    assert_eq!(m.camera.y, -25.0, "past the delta it follows");
    lift(&mut m, -21.0);
    m.update_camera();
    assert_eq!(m.camera.y, -25.0, "a 4px change does not move it back");
}

#[test]
fn y_scroll_speed_sets_how_fast_the_camera_catches_up() {
    let mut m = field();
    m.stage.camera.y_scroll_speed = 0.5;
    lift(&mut m, -40.0);
    m.update_camera();
    assert!((m.camera.y + 20.0).abs() < 1e-4);
    m.update_camera();
    assert!((m.camera.y + 30.0).abs() < 1e-4);
}

#[test]
fn lowest_cap_keeps_the_camera_from_dropping_under_the_start_height() {
    let mut m = field();
    m.stage.start_y = -15.0;
    m.stage.camera.y_scroll_speed = 1.0;
    m.stage.camera.lowest_cap = true;
    lift(&mut m, -5.0);
    m.update_camera();
    assert_eq!(m.camera.y, -15.0);
    m.stage.camera.lowest_cap = false;
    m.update_camera();
    assert_eq!(m.camera.y, -5.0);
}

#[test]
fn dynamic_bound_high_grows_with_the_zoom_out() {
    let mut m = field();
    m.set_zoom_for_teams(5, 0.5);
    m.stage.bound_high = -60.0;
    m.stage.camera.zoom_in_speed = 0.0;
    m.stage.camera.zoom_out_speed = 0.0;
    m.stage.camera.y_scroll_speed = 1.0;
    lift(&mut m, -900.0);
    m.update_camera();
    assert_eq!(m.camera.y, -60.0, "fixed bound by default");
    m.stage.camera.dynamic_bound_high = true;
    m.stage.camera.bound_high_delta = 100.0;
    m.update_camera();
    let expected = -60.0 - 100.0 * (m.zoom - 1.0);
    assert!((m.camera.y - expected).abs() < 1e-3);
}

#[test]
fn zoom_speeds_are_clamped_and_directional() {
    let mut m = field();
    m.set_zoom_for_teams(5, 0.5);
    m.fighters[0].pos.x = -250.0;
    m.fighters[1].pos.x = 250.0;
    let start = m.zoom;
    m.stage.camera.zoom_out_speed = 0.0;
    m.update_camera();
    assert_eq!(m.zoom, start, "speed 0 holds the zoom");
    m.stage.camera.zoom_out_speed = 7.0;
    m.update_camera();
    let target = (500.0 + 150.0) / ume_engine::matchs::BASE_W;
    assert!(
        (m.zoom - target).abs() < 1e-4,
        "speeds above 1 clamp to a one-tick snap"
    );
    m.fighters[0].pos.x = -20.0;
    m.fighters[1].pos.x = 20.0;
    m.stage.camera.zoom_in_speed = 0.0;
    let held = m.zoom;
    m.update_camera();
    assert_eq!(
        m.zoom, held,
        "zoom-in speed is separate from zoom-out speed"
    );
    m.stage.camera.zoom_in_speed = 1.0;
    m.update_camera();
    assert!((m.zoom - m.zoom_min).abs() < 1e-4);
}

#[test]
fn auto_turn_off_keeps_the_facing_when_the_enemy_crosses_over() {
    for (auto_turn, expected) in [(true, -1.0f32), (false, 1.0)] {
        let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
        m.stage.auto_turn = auto_turn;
        m.fighters[0].pos.x = 0.0;
        m.fighters[0].facing = 1.0;
        m.fighters[0].ctrl = true;
        m.fighters[1].pos.x = -60.0;
        m.tick(&[0, 0]);
        assert_eq!(m.fighters[0].facing, expected, "autoturn {auto_turn}");
    }
}

#[test]
fn a_cornered_fighter_holds_still_while_the_opponent_closes_in() {
    let mut m = field();
    let half = m.view_w / 2.0;
    m.camera.x = 0.0;
    m.fighters[0].pos.x = -half + m.stage.tension + 10.0;
    m.fighters[1].pos.x = half - m.stage.tension - 10.0;
    let screen_x = m.fighters[0].pos.x - m.camera.x;
    for _ in 0..140 {
        m.fighters[1].pos.x -= 1.0;
        m.update_camera();
        assert!(
            (m.fighters[0].pos.x - m.camera.x - screen_x).abs() < 1e-3,
            "the cornered fighter drifted to {}",
            m.fighters[0].pos.x - m.camera.x
        );
    }
}

#[test]
fn the_camera_pans_until_a_fighter_leaves_the_tension_band() {
    let mut m = field();
    let half = m.view_w / 2.0;
    m.camera.x = 0.0;
    m.fighters[0].pos.x = -half + 20.0;
    m.fighters[1].pos.x = 70.0;
    for _ in 0..60 {
        m.update_camera();
    }
    let screen_x = m.fighters[0].pos.x - m.camera.x + half;
    assert!(
        (screen_x - m.stage.tension).abs() < 1e-2,
        "expected tension {0} px from the edge, got {screen_x}",
        m.stage.tension
    );
}

#[test]
fn stagevar_reports_the_camera_parameters() {
    let st = Arc::new(load(
        "[Camera]\nverticalfollowdelta = 4\nzoominspeed = 0.1\nlowestcap = 1\n\n[BGdef]\n",
    ));
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.bg = Some(ume_engine::stage_bg::BgState::new(st));
    let ev = |src: &str| {
        m.ev(
            0,
            &ume_engine::expr::Expr::new(ume_engine::expr::parse(src).unwrap()),
        )
    };
    assert_eq!(ev("StageVar(camera.verticalfollowdelta)"), 4.0);
    assert!((ev("StageVar(camera.zoominspeed)") - 0.1).abs() < 1e-6);
    assert_eq!(ev("StageVar(camera.lowestcap)"), 1.0);
    assert!((ev("StageVar(camera.yscrollspeed)") - 0.3).abs() < 1e-6);
}
