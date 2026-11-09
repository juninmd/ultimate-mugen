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
}
