use super::{ElementState, Input, Pacer, KeyCode};

/// Holding a key and alt-tabbing: the window that loses focus never sees the key-up, so the shell has to
/// forget it. Otherwise the fighter keeps walking (or keeps an attack button pressed, which makes every
/// command stop firing because commands need a fresh press edge).
#[test]
fn a_lost_key_up_is_cleared_when_the_window_loses_focus() {
    let mut i = Input::default();
    i.key(ElementState::Pressed, Some(KeyCode::ArrowRight), false, "");
    i.key(ElementState::Pressed, Some(KeyCode::KeyX), false, "x");
    assert!(i.is_down(KeyCode::ArrowRight) && i.is_down(KeyCode::KeyX));
    i.release_all();
    assert!(!i.is_down(KeyCode::ArrowRight) && !i.is_down(KeyCode::KeyX), "nothing stays held");
    assert!(!i.is_pressed(KeyCode::KeyX) && i.pressed_order.is_empty(), "no stale press survives the focus change");
}

#[test]
fn focus_loss_clears_the_mouse_and_every_finger() {
    let mut i = Input::default();
    i.mouse_down = true; i.mouse_pressed = true;
    i.touch(3, super::Phase::Started, [40.0, 50.0]);
    i.touch(4, super::Phase::Started, [60.0, 70.0]);
    i.release_all();
    assert!(!i.mouse_down && !i.mouse_pressed);
    assert!(i.touches.is_empty(), "no finger stays pressed on the touch pad");
}

#[test]
fn releasing_a_key_normally_is_unchanged() {
    let mut i = Input::default();
    i.key(ElementState::Pressed, Some(KeyCode::KeyQ), false, "q");
    i.key(ElementState::Released, Some(KeyCode::KeyQ), false, "");
    assert!(!i.is_down(KeyCode::KeyQ) && i.is_pressed(KeyCode::KeyQ), "the press is still readable this frame");
    assert_eq!(i.typed, "q");
}

#[test]
fn a_repeated_key_does_not_queue_extra_presses() {
    let mut i = Input::default();
    i.key(ElementState::Pressed, Some(KeyCode::ArrowUp), false, "");
    i.key(ElementState::Pressed, Some(KeyCode::ArrowUp), true, "");
    assert_eq!(i.pressed_order, vec![KeyCode::ArrowUp]);
    assert!(i.is_down(KeyCode::ArrowUp));
}

#[test]
fn stretch_forces_sixteen_nine_logical_space_on_any_window() {
    assert_eq!(super::logical_size([1000.0, 500.0], None), [1000.0, 500.0]);
    let [w, h] = super::logical_size([1000.0, 900.0], Some(super::STRETCH_ASPECT));
    assert_eq!(h, 900.0);
    assert!((w / h - 16.0 / 9.0).abs() < 1e-5);
}

#[test]
fn jittery_vsync_frames_become_exact_periods() {
    let mut p = Pacer::default();
    for raw in [0.0163, 0.0171, 0.0165, 0.0168, 0.0166] { assert_eq!(p.pace(raw), 1.0 / 60.0); }
}

#[test]
fn long_run_keeps_real_time() {
    let mut p = Pacer::default();
    let raw = 1.0 / 59.5;
    let total: f32 = (0..6000).map(|_| p.pace(raw)).sum();
    assert!((total - raw * 6000.0).abs() < 1.0 / 60.0 + 1e-3);
}

#[test]
fn hitches_pass_through() {
    let mut p = Pacer::default();
    assert_eq!(p.pace(0.05), 0.05);
    assert_eq!(p.pace(1.0), 0.25);
}
