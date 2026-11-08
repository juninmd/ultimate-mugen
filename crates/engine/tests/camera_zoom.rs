mod support;
use support::*;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

fn ev(m: &Match, src: &str) -> f64 {
    m.ev(0, &Expr::new(parse(src).unwrap()))
}

fn zoom_state(params: &str) -> String {
    format!("[Statedef 800]\ntype = S\nphysics = S\n\n[State 800, zoom]\ntype = Zoom\ntrigger1 = var(0) = 0\n{params}\n\n[State 800, once]\ntype = VarSet\ntrigger1 = 1\nv = 0\nvalue = 1\n")
}

fn zoomer(params: &str) -> Match {
    let mut m = duel(
        mugen_char("zoomer", &zoom_state(params)),
        mugen_char("dummy", ""),
    );
    m.stage.zoom_in = 1.5;
    m.stage.zoom_out = 0.8;
    m.stage.bound_right = 10.0;
    m.stage.bound_left = -10.0;
    enter(&mut m, 0, 800);
    m
}

fn step(m: &mut Match, n: usize) {
    for _ in 0..n {
        m.tick(&[0, 0]);
    }
}

#[test]
fn camera_zoom_defaults_to_one() {
    let m = duel(mugen_char("a", ""), mugen_char("b", ""));
    assert_eq!(ev(&m, "CameraZoom"), 1.0);
    assert_eq!(ev(&m, "ZoomVar(scale)"), 1.0);
    assert_eq!(ev(&m, "ZoomVar(active)"), 0.0);
}

#[test]
fn zoom_sctrl_is_read_back_by_camera_zoom() {
    let mut m = zoomer("scale = 1.2\ntime = 3");
    step(&mut m, 1);
    assert!((ev(&m, "CameraZoom") - 1.2).abs() < 1e-4);
    assert_eq!(ev(&m, "ZoomVar(active)"), 1.0);
    assert!((ev(&m, "ZoomVar(target)") - 1.2).abs() < 1e-4);
    assert_eq!(ev(&m, "CameraZoom = ZoomVar(scale)"), 1.0);
    assert!(
        m.view_w < ume_engine::matchs::BASE_W,
        "zooming in shrinks the visible field"
    );
}

#[test]
fn zoom_in_is_bounded_by_the_stage_unless_bound_is_off() {
    let mut m = zoomer("scale = 3\ntime = 2");
    step(&mut m, 1);
    assert!(
        (ev(&m, "CameraZoom") - 1.5).abs() < 1e-4,
        "stage zoomin = 1.5"
    );
    let mut free = zoomer("scale = 3\ntime = 2\nbound = 0");
    step(&mut free, 1);
    assert!((ev(&free, "CameraZoom") - 3.0).abs() < 1e-4);
}

#[test]
fn zoom_out_is_bounded_by_the_stage() {
    let mut m = zoomer("scale = 0.5\ntime = 2");
    step(&mut m, 1);
    assert!(
        (ev(&m, "CameraZoom") - 0.8).abs() < 1e-4,
        "stage zoomout = 0.8"
    );
    assert!((ev(&m, "ZoomVar(min)") - 0.8).abs() < 1e-4);
    assert!((ev(&m, "ZoomVar(max)") - 1.5).abs() < 1e-4);
}

#[test]
fn lag_smooths_the_zoom_change() {
    let mut m = zoomer("scale = 1.2\ntime = 6\nlag = 0.5");
    step(&mut m, 1);
    let first = ev(&m, "CameraZoom");
    assert!(first > 1.0 && first < 1.2, "{first}");
    step(&mut m, 5);
    assert!((ev(&m, "CameraZoom") - 1.2).abs() < 0.01);
}

#[test]
fn zoom_in_delay_postpones_the_zoom_in() {
    let mut m = zoomer("scale = 1.2\ntime = 6\nzoomindelay = 2");
    step(&mut m, 2);
    assert_eq!(ev(&m, "CameraZoom"), 1.0);
    step(&mut m, 1);
    assert!((ev(&m, "CameraZoom") - 1.2).abs() < 1e-4);
}

#[test]
fn delta_keeps_changing_the_requested_scale() {
    let mut m = zoomer("scale = 1.0\ndelta = 0.1\ntime = 4");
    step(&mut m, 3);
    assert!((ev(&m, "CameraZoom") - 1.2).abs() < 1e-3);
}

#[test]
fn the_zoom_returns_to_normal_after_the_request_ends() {
    let mut m = zoomer("scale = 1.3\ntime = 2");
    step(&mut m, 2);
    assert!(ev(&m, "CameraZoom") > 1.2);
    step(&mut m, 200);
    assert!((ev(&m, "CameraZoom") - 1.0).abs() < 0.01);
    assert_eq!(ev(&m, "ZoomVar(active)"), 0.0);
}

#[test]
fn pos_moves_the_camera_and_bound_keeps_it_inside_the_stage() {
    let mut bounded = zoomer("scale = 1\npos = 80, 0\ntime = 20");
    step(&mut bounded, 12);
    assert!(bounded.camera.x <= 10.0 + 1e-3, "{}", bounded.camera.x);
    let mut free = zoomer("scale = 1\npos = 80, 0\ntime = 20\nbound = 0");
    step(&mut free, 12);
    assert!(free.camera.x > 60.0, "{}", free.camera.x);
    assert_eq!(ev(&free, "ZoomVar(posx)"), 80.0);
}

#[test]
fn the_zoom_is_deterministic() {
    let mut a = zoomer("scale = 1.25\ntime = 10\nlag = 0.3");
    let mut b = a.clone();
    for _ in 0..30 {
        a.tick(&[0, 0]);
        b.tick(&[0, 0]);
        assert_eq!(a.zoom.to_bits(), b.zoom.to_bits());
        assert_eq!(a.camera.x.to_bits(), b.camera.x.to_bits());
    }
}
