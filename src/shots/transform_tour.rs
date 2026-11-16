use super::{find, screen, stage};
use crate::app::{App, Mode};
use ume_engine::potential::PotentialLevel;

async fn open_duel(app: &mut App, first: &str, second: &str) {
    app.mode = Mode::Watch;
    app.sel = (find(app, first), find(app, second), stage(app, "dojo"));
    app.start_fight().await;
    if let Some(s) = app.session.as_mut() {
        s.m.round_state = 2;
        s.m.intro_ticks = 0;
        s.m.fighters[0].pos.x = -50.0;
        s.m.fighters[1].pos.x = 50.0;
    }
}

async fn settle(app: &mut App) {
    for _ in 0..6 {
        if let Some(s) = app.session.as_mut() {
            s.step_visual(&mut app.c, &mut app.audio);
        }
    }
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    open_duel(app, "bruno", "kaito").await;
    let steps: [(&str, u8, PotentialLevel, f32); 4] = [
        ("native-form-1", 1, PotentialLevel::Idle, 0.0),
        ("potential", 0, PotentialLevel::Potential, 0.6),
        ("fury", 0, PotentialLevel::Fury, 0.3),
        ("potential-and-form", 1, PotentialLevel::Potential, 0.9),
    ];
    for (tag, form, level, left) in steps {
        if let Some(s) = app.session.as_mut() {
            s.m.fighters[1].ctrl = false;
            let f = &mut s.m.fighters[0];
            f.form = form;
            f.ctrl = false;
            if level != PotentialLevel::Idle {
                f.set_potential_level(level);
                f.potential.time = (level.duration() as f32 * left) as i32;
            }
        }
        settle(app).await;
        screen(app, dir, &format!("transform-{tag}"), 2).await;
    }
}
