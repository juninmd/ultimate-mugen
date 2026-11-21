use super::{find, screen, stage};
use crate::app::{App, Mode};
use crate::session::dummy::DummyPlan;
use crate::session::dummy_tape;
use ume_engine::input::k;

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Training;
    app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
    app.team = None;
    app.start_fight().await;
    dummy_tape::start_recording();
    for i in 0..150 {
        let bits = match i % 50 {
            0..=19 => k::R,
            20..=24 => k::X,
            _ => 0,
        };
        dummy_tape::record(bits, 1.0);
    }
    app.dummy_plan = DummyPlan::Playback;
    if let Some(s) = app.session.as_mut() {
        s.set_dummy_plan(DummyPlan::Playback);
        s.hud.inject_last_combo(7, 214.0);
    }
    dummy_tape::stop_recording();
    screen(app, dir, "training-best-before", 120).await;
    if let Some(s) = app.session.as_mut() {
        s.hud = crate::hud::Hud::new();
    }
    screen(app, dir, "training-best-after", 30).await;
}
