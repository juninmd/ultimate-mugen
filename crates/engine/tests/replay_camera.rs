use ume_engine::replay_camera::{direct, Subject, TIGHTEN_MAX};

fn s(x: f32) -> Subject {
    Subject { x, attacking: false }
}

#[test]
fn close_fighters_are_framed_tighter_than_far_apart_ones() {
    let near = direct(0.0, 1.0, &[s(-40.0), s(40.0)]);
    let far = direct(0.0, 1.0, &[s(-300.0), s(300.0)]);
    assert!(near.zoom > far.zoom, "close fight must zoom in more");
    assert!((far.zoom - 1.0).abs() < 0.05, "wide spread keeps the base zoom");
    assert!(near.zoom <= TIGHTEN_MAX + 1e-4, "zoom never exceeds the cap");
}

#[test]
fn zoom_scales_the_match_base_zoom() {
    let base = direct(0.0, 1.3, &[s(-40.0), s(40.0)]).zoom;
    let plain = direct(0.0, 1.0, &[s(-40.0), s(40.0)]).zoom;
    assert!((base / plain - 1.3).abs() < 1e-4, "a multi-fighter base zoom must not be lost");
}

#[test]
fn focus_leans_toward_the_attacker_without_snapping() {
    let subjects = [Subject { x: -100.0, attacking: false }, Subject { x: 100.0, attacking: true }];
    let shot = direct(0.0, 1.0, &subjects);
    assert!(shot.x > 0.0 && shot.x < 100.0, "pan moves toward the attacker: {}", shot.x);
}

#[test]
fn pan_eases_part_of_the_way_per_frame() {
    let shot = direct(0.0, 1.0, &[s(200.0), s(200.0)]);
    assert!(shot.x > 0.0 && shot.x < 200.0, "one frame moves part of the way: {}", shot.x);
}

#[test]
fn no_valid_subjects_keeps_the_previous_shot() {
    assert_eq!(direct(42.0, 1.0, &[]).x, 42.0);
    assert_eq!(direct(42.0, 1.0, &[s(f32::NAN)]).x, 42.0);
}

#[test]
fn direction_is_deterministic() {
    let subjects = [s(10.0), s(-30.0)];
    assert_eq!(direct(5.0, 1.0, &subjects), direct(5.0, 1.0, &subjects));
}
