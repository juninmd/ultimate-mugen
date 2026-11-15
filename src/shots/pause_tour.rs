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
    training_dummy(app, dir).await;
}

async fn training_dummy(app: &mut App, dir: &str) {
    use crate::session::dummy::DummyPlan;
    app.mode = Mode::Training;
    app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
    app.team = None;
    app.start_fight().await;
    for (plan, name, frames) in [(DummyPlan::Crouch, "training-dummy-crouch", 260), (DummyPlan::Jump, "training-dummy-jump", 50)] {
        app.dummy_plan = plan;
        if let Some(s) = app.session.as_mut() {
            s.set_dummy_plan(plan);
        }
        screen(app, dir, name, frames).await;
    }
    app.dummy_plan = DummyPlan::Guard;
    if let Some(s) = app.session.as_mut() {
        s.set_dummy_plan(DummyPlan::Guard);
        s.paused = true;
    }
    app.pause_preview(0, 3, &[]);
    screen(app, dir, "pause-training-dummy", 30).await;
}
