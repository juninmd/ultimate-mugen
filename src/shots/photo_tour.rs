use super::{find, screen, stage};
use crate::app::{App, Mode};
use crate::photo::{Filter, Photo};

fn look(app: &mut App, filter: Filter, hud: bool, zoom: f32, pan: (f32, f32)) {
    let Some(photo) = app.photo.as_mut() else { return };
    photo.filter = filter;
    photo.hud = hud;
    photo.view.zoom = zoom;
    photo.view.dx = pan.0;
    photo.view.dy = pan.1;
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Watch;
    app.sel = (find(app, "lian"), find(app, "shion"), stage(app, "bamboo"));
    app.start_fight().await;
    {
        let App { session, c, audio, .. } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..420 {
            s.step_visual(c, audio);
        }
        s.paused = true;
    }
    app.pause_preview(0, 0, &[]);
    screen(app, dir, "photo-0-pause", 30).await;
    app.photo = Some(Photo::new());
    look(app, Filter::Off, false, 1.0, (0.0, 0.0));
    screen(app, dir, "photo-1-clean", 6).await;
    look(app, Filter::Off, true, 1.0, (0.0, 0.0));
    screen(app, dir, "photo-2-hud", 6).await;
    for (filter, name) in [(Filter::Cinema, "cinema"), (Filter::Noir, "noir"), (Filter::Vivid, "vivid"), (Filter::Retro, "retro")] {
        look(app, filter, false, 1.7, (0.0, -10.0));
        screen(app, dir, &format!("photo-3-{name}"), 6).await;
    }
    look(app, Filter::Cinema, false, 0.7, (0.0, 0.0));
    screen(app, dir, "photo-4-wide", 6).await;
}
