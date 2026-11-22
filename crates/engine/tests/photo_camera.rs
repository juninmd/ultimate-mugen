mod support;
use support::*;
use ume_engine::photo_camera::{frame, unframe, PhotoView, CLOSE_MAX, FAR_MAX, PAN_Y_LIMIT};

fn arena() -> ume_engine::matchs::Match {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.stage.bound_left = -200.0;
    m.stage.bound_right = 200.0;
    m
}

#[test]
fn framing_is_display_only_and_restores_every_field() {
    let mut m = arena();
    m.camera.x = 30.0;
    let before = (m.camera.x, m.camera.y, m.zoom, m.view_w, m.view_h, m.stage.zoffset);
    let mut view = PhotoView::default();
    view.pan(80.0, 40.0);
    view.zoom_by(1.8);
    let saved = frame(&mut m, &view).expect("valid view");
    assert_eq!(m.view_w, before.3, "close-ups are drawn by the renderer, the field never shrinks");
    unframe(&mut m, saved);
    assert_eq!(before, (m.camera.x, m.camera.y, m.zoom, m.view_w, m.view_h, m.stage.zoffset));
}

#[test]
fn panning_never_leaves_the_stage() {
    let mut m = arena();
    let mut view = PhotoView::default();
    view.pan(5000.0, 0.0);
    frame(&mut m, &view).unwrap();
    assert_eq!(m.camera.x, 200.0);
    view.pan(-99999.0, 0.0);
    frame(&mut m, &view).unwrap();
    assert_eq!(m.camera.x, -200.0);
}

#[test]
fn zoom_and_vertical_pan_are_clamped() {
    let mut view = PhotoView::default();
    for _ in 0..50 {
        view.zoom_by(1.2);
    }
    assert_eq!(view.zoom, CLOSE_MAX);
    for _ in 0..80 {
        view.zoom_by(0.8);
    }
    assert_eq!(view.zoom, FAR_MAX);
    view.pan(0.0, 9999.0);
    assert_eq!(view.dy, PAN_Y_LIMIT);
}

#[test]
fn non_finite_input_is_ignored() {
    let mut view = PhotoView::default();
    view.pan(f32::NAN, 1.0);
    view.zoom_by(f32::INFINITY);
    view.zoom_by(0.0);
    assert!(view.is_centered());
}

#[test]
fn centered_view_keeps_the_match_framing() {
    let mut m = arena();
    let (w, h) = (m.view_w, m.view_h);
    frame(&mut m, &PhotoView::default()).unwrap();
    assert!((m.view_w - w).abs() < 0.01 && (m.view_h - h).abs() < 0.01);
}

#[test]
fn zooming_out_widens_the_field_and_closeup_reports_the_rest() {
    let mut m = arena();
    let w = m.view_w;
    let mut view = PhotoView::default();
    view.zoom_by(0.7);
    frame(&mut m, &view).unwrap();
    assert!(m.view_w > w);
    assert_eq!(ume_engine::photo_camera::closeup(&view), 1.0);
    view.zoom_by(3.0);
    assert!(ume_engine::photo_camera::closeup(&view) > 1.0);
}
