use super::{find, screen, stage};
use crate::app::{App, Mode};
use crate::trials::{Sample, Tracker};

fn demo(app: &mut App, cleared: usize, hits: i32, damage: f32) {
    let Some((_, t)) = app.trials.as_mut() else { return };
    t.cleared = cleared;
    t.demo = Some(Sample { hits, damage, life_max: 1000.0, power: 0.0 });
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Training;
    app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
    app.team = None;
    app.start_fight().await;
    screen(app, dir, "trials-0-first", 240).await;
    let name = app.session.as_ref().map(|s| s.m.fighters[0].def.display_name.clone()).unwrap_or_default();
    app.trials = Some((name, Tracker::default()));
    demo(app, 2, 5, 180.0);
    screen(app, dir, "trials-1-progress", 3).await;
    demo(app, 2, 9, 300.0);
    screen(app, dir, "trials-2-almost", 3).await;
    demo(app, 2, 0, 0.0);
    screen(app, dir, "trials-3-clear", 3).await;
    demo(app, 4, 7, 220.0);
    screen(app, dir, "trials-4-super", 3).await;
}
