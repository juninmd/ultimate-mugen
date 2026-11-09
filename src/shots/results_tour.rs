use super::{find, screen, stage};
use crate::app::{App, Mode, Screen};

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Watch;
    app.team = None;
    app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "dojo"));
    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..1500 {
            s.step_visual(c, audio);
        }
        s.m.fighters[1].life *= 0.3;
    }
    app.screen = Screen::Results { winner: 0, cur: 0 };
    screen(app, dir, "r1-results-stats", 30).await;
    grid_presets(app, dir).await;
}

pub(crate) async fn grid_presets(app: &mut App, dir: &str) {
    app.mode = Mode::Cpu;
    let ids = ["kaname", "valdric", "akio", "kaito"];
    for preset in 0..crate::settings::GRID_PRESETS.len() {
        app.settings.grid = preset;
        app.screen = Screen::Select {
            cur: [find(app, ids[0]), find(app, ids[1])],
            picking: 0,
            picked: [None, None],
        };
        screen(app, dir, &format!("grid-{preset}"), 60).await;
    }
    app.settings.grid = 1;
}
