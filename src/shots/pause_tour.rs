use super::{find, screen, stage};
use crate::app::{App, Mode};

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Watch;
    app.sel = (find(app, "lian"), find(app, "shion"), stage(app, "bamboo"));
    app.start_fight().await;
    if let Some(s) = app.session.as_mut() {
        s.paused = true;
    }
    app.pause_preview(0, 0, &[]);
    screen(app, dir, "pause-game", 30).await;
    app.pause_preview(1, 0, &[(0, 0), (1, 2), (3, 4)]);
    screen(app, dir, "pause-moves", 30).await;
    app.pause_preview(1, 7, &[]);
    screen(app, dir, "pause-moves-scrolled", 45).await;
    app.pause_preview(2, 0, &[]);
    screen(app, dir, "pause-system", 30).await;
}
