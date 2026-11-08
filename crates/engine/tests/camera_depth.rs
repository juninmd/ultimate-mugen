mod support;
use support::*;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

fn ev(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &Expr::new(parse(src).unwrap()))
}

fn step(m: &mut Match, n: usize) {
    for _ in 0..n {
        m.tick(&[0, 0]);
    }
}

fn once(kind: &str, params: &str) -> String {
    format!("[Statedef 800]\ntype = S\nphysics = S\n\n[State 800, run]\ntype = {kind}\ntrigger1 = var(0) = 0\n{params}\n\n[State 800, once]\ntype = VarSet\ntrigger1 = 1\nv = 0\nvalue = 1\n\n[State 800, shot]\ntype = Projectile\ntrigger1 = Time = 3\nprojanim = 1000\nattr = S, NP\ndamage = 10\nhitflag = MAF\npausetime = 0, 0\nvelocity = 0\noffset = 0, -30\n")
}

fn scripted(kind: &str, params: &str) -> Match {
    let mut m = duel(
        mugen_char("scripted", &once(kind, params)),
        mugen_char("dummy", ""),
    );
    enter(&mut m, 0, 800);
    m
}

#[test]
fn ground_level_offset_raises_the_floor_under_the_character() {
    let mut m = scripted("GroundLevelOffset", "value = 20");
    step(&mut m, 2);
    assert_eq!(m.fighters[0].pos.y, -20.0);
    assert_eq!(
        ev(&m, 0, "Pos Y"),
        0.0,
        "Pos Y is measured from the character's own ground"
    );
    assert!(
        (ev(&m, 1, "ScreenPos Y") - ev(&m, 0, "ScreenPos Y") - 20.0).abs() < 1e-3,
        "the raised floor shows on screen"
    );
    assert_eq!(ev(&m, 1, "Pos Y"), 0.0);
}

#[test]
fn ground_level_offset_moves_the_projectile_spawn_height() {
    let mut m = scripted("GroundLevelOffset", "value = 20");
    step(&mut m, 5);
    assert_eq!(m.projectiles.len(), 1);
    assert!(
        (m.projectiles[0].pos.y - (-20.0 - 30.0)).abs() < 1.5,
        "{}",
        m.projectiles[0].pos.y
    );
}

#[test]
fn a_negative_offset_digs_the_floor_and_zero_restores_it() {
    let mut m = scripted("GroundLevelOffset", "value = -15");
    step(&mut m, 40);
    assert_eq!(m.fighters[0].pos.y, 15.0);
    assert_eq!(ev(&m, 0, "Pos Y"), 0.0);
    m.fighters[0].ground_level = 0.0;
    step(&mut m, 40);
    assert_eq!(m.fighters[0].pos.y, 0.0);
}

#[test]
fn a_raised_floor_does_not_count_as_airborne() {
    let mut m = scripted("GroundLevelOffset", "value = 20");
    step(&mut m, 2);
    assert!(m.fighters[0].air_height().abs() < 1e-4);
}

#[test]
fn camera_controller_moves_and_zooms_the_camera() {
    let mut m = scripted("Camera", "pos = 5, 0\nzoom = 1.2\ntime = 3");
    m.stage.zoom_in = 1.5;
    m.stage.bound_right = 100.0;
    step(&mut m, 1);
    assert!((ev(&m, 0, "CameraZoom") - 1.2).abs() < 1e-4);
    step(&mut m, 2);
    assert!(m.camera.x > 4.0, "{}", m.camera.x);
}

#[test]
fn camera_controller_without_zoom_keeps_the_current_scale() {
    let mut m = scripted("Camera", "pos = 5, 0\ntime = 2");
    step(&mut m, 1);
    assert_eq!(ev(&m, 0, "CameraZoom"), 1.0);
}

#[test]
fn depth_controllers_are_ignored_with_one_warning() {
    let mut m = scripted("Depth", "value = 12");
    let (x, y) = (m.fighters[0].pos.x, m.fighters[0].pos.y);
    step(&mut m, 3);
    assert_eq!((m.fighters[0].pos.x, m.fighters[0].pos.y), (x, y));
    assert_eq!(m.warnings.len(), 1);
    assert!(m.warnings[0].contains("Z axis"));
    assert_eq!(ev(&m, 0, "Pos Z"), 0.0);
    let mut bound = scripted("DepthBound", "value = 4");
    step(&mut bound, 3);
    assert_eq!(bound.warnings.len(), 1);
    assert!(bound.warnings[0].starts_with("depthbound"));
}
