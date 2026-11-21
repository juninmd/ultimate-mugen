use super::{find, screen, stage};
use crate::app::{App, Mode, Screen};

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Cpu;
    let chosen = stage(app, "dojo") + 1;
    app.sel = (find(app, "kaito"), find(app, "akio"), chosen - 1);
    app.set_stage_filter(2, "zzz");
    app.screen = Screen::Stages { cur: chosen };
    screen(app, dir, "stage-filter-sticky", 8).await;
    app.open_stages(chosen);
    screen(app, dir, "stage-filter-entry", 8).await;
    let linked = stage(app, "neon");
    app.sel.2 = linked;
    app.screen = Screen::Stages { cur: app.sel.2 };
    screen(app, dir, "stage-link-before", 6).await;
    app.open_stages(app.sel.2 + 1);
    screen(app, dir, "stage-link-after", 6).await;
}
