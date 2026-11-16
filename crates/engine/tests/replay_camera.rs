use ume_engine::replay_camera::{ReplayCamera, Subject, ZOOM_MAX, ZOOM_MIN};

fn s(x: f32) -> Subject {
    Subject { x, attacking: false }
}

fn settle(cam: &mut ReplayCamera, subjects: &[Subject]) {
    for _ in 0..600 {
        cam.update(subjects);
    }
}

#[test]
fn close_fighters_get_a_tighter_shot_than_far_apart_ones() {
    let mut near = ReplayCamera::new(0.0);
    settle(&mut near, &[s(-40.0), s(40.0)]);
    let mut far = ReplayCamera::new(0.0);
    settle(&mut far, &[s(-300.0), s(300.0)]);
    assert!(near.shot().zoom > far.shot().zoom, "close fight must zoom in more");
    assert!((far.shot().zoom - ZOOM_MIN).abs() < 0.05, "wide spread stays at the base zoom");
    assert!(near.shot().zoom <= ZOOM_MAX + 1e-4, "zoom never exceeds the cinematic cap");
}

#[test]
fn camera_leans_toward_the_attacker() {
    let mut cam = ReplayCamera::new(0.0);
    let subjects = [Subject { x: -100.0, attacking: false }, Subject { x: 100.0, attacking: true }];
    settle(&mut cam, &subjects);
    assert!(cam.shot().x > 0.0, "focus must lean toward the attacker at x=100");
    assert!(cam.shot().x < 100.0, "focus must not snap onto the attacker");
}

#[test]
fn pan_eases_without_overshooting_in_one_step() {
    let mut cam = ReplayCamera::new(0.0);
    let before = cam.shot().x;
    let after = cam.update(&[s(200.0), s(200.0)]).x;
    assert!(after > before && after < 200.0, "one frame moves part of the way: {after}");
}

#[test]
fn empty_or_non_finite_subjects_keep_the_last_shot() {
    let mut cam = ReplayCamera::new(50.0);
    settle(&mut cam, &[s(60.0)]);
    let kept = cam.shot();
    assert_eq!(cam.update(&[]), kept);
    assert_eq!(cam.update(&[s(f32::NAN)]), kept);
}

#[test]
fn update_is_deterministic() {
    let run = || {
        let mut cam = ReplayCamera::new(10.0);
        let mut last = cam.shot();
        for i in 0..120 {
            last = cam.update(&[s(i as f32 * 0.5), s(-50.0)]);
        }
        last
    };
    assert_eq!(run(), run());
}
