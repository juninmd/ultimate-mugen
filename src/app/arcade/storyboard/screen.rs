use super::draw::draw;
use crate::app::{App, Screen};
use crate::controls::nav;
use crate::gfx::get_frame_time;

pub(super) fn tick(app: &mut App) {
    let Screen::Cutscene {
        kind: 6,
        mut page,
        mut t,
    } = app.screen
    else {
        return;
    };
    let Some(playback) = app
        .arcade
        .as_mut()
        .and_then(|run| run.storyboard_player.take())
    else {
        app.route_storyboard_done();
        return;
    };
    let Some(scene) = playback.storyboard.scenes.get(page) else {
        app.route_storyboard_done();
        return;
    };
    draw(app, &playback, page, t);
    let dt = get_frame_time();
    let max_time = scene.end_time.max(1) as f32 / 60.0;
    let skip = app.skip_chip() || app.c.pressed(nav::BACK | nav::START);
    if skip {
        app.route_storyboard_done();
        return;
    }
    if app.c.pressed(nav::OK) || app.tap_any() || t + dt >= max_time {
        page += 1;
        t = 0.0;
    } else {
        t += dt;
    }
    if page >= playback.storyboard.scenes.len() {
        app.route_storyboard_done();
        return;
    }
    if let Some(run) = app.arcade.as_mut() {
        run.storyboard_player = Some(playback);
    }
    app.screen = Screen::Cutscene { kind: 6, page, t };
}
