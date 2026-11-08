use super::{find, screen, stage};
use crate::app::{App, Mode};

const BRUNO_LIFE_SHARES: [(f32, &str); 4] = [(1.0, "100"), (0.5, "50"), (0.1, "10"), (0.01, "01")];
const GARAN_FIGHT_SECONDS: [(i32, &str); 4] = [(0, "000"), (30, "030"), (67, "067"), (120, "120")];
const SYLENE_STACKS: [i32; 3] = [0, 4, 10];

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
    for (share, tag) in BRUNO_LIFE_SHARES {
        if let Some(s) = app.session.as_mut() {
            let f = &mut s.m.fighters[0];
            f.life = f.life_max * share;
            s.m.fighters[1].life = s.m.fighters[1].life_max;
            s.m.fighters[0].ctrl = false;
            s.m.fighters[1].ctrl = false;
        }
        settle(app).await;
        screen(app, dir, &format!("bruno-rage-{tag}"), 2).await;
    }
    open_duel(app, "sylene", "kaito").await;
    for stacks in SYLENE_STACKS {
        if let Some(s) = app.session.as_mut() {
            s.m.fighters[0].passive.stacks = stacks;
            s.m.fighters[0].ctrl = false;
            s.m.fighters[1].ctrl = false;
        }
        settle(app).await;
        screen(app, dir, &format!("sylene-stacks-{stacks:02}"), 2).await;
    }
    open_duel(app, "garan", "kaito").await;
    for (seconds, tag) in GARAN_FIGHT_SECONDS {
        if let Some(s) = app.session.as_mut() {
            s.m.fighters[0].passive.fight_ticks = seconds * 60;
            s.m.fighters[0].ctrl = false;
            s.m.fighters[1].ctrl = false;
        }
        settle(app).await;
        screen(app, dir, &format!("garan-enrage-{tag}"), 2).await;
    }
}
