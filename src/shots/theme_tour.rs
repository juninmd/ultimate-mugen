use super::{find, screen, stage};
use crate::app::{App, Mode, Screen};

const FILE: &str = "tweaks.toml";
const READY_LIMIT: usize = 600;
const FIGHT_TICKS: usize = 420;
const CUSTOM: &str = "[theme]\npreset = \"mono\"\naccent = \"#7a3cff\"\ntitle = \"#b8ff5c\"\n\n[hud]\nlife = \"#5cc8ff\"\nlife_low = \"#ff5ca8\"\ndamage = \"#7a3cff\"\n\n[menu]\narcade = \"Fliperama\"\nstory = \"Aventura\"\n";

fn use_theme(text: &str) {
    crate::storage::write(FILE, text);
    crate::tweaks::reload_now();
}

fn step(app: &mut App, ticks: usize, until_ready: bool) {
    let App {
        session, c, audio, ..
    } = app;
    if let Some(s) = session.as_mut() {
        for _ in 0..ticks {
            if until_ready && s.m.round_state == 2 {
                break;
            }
            s.step_visual(c, audio);
        }
    }
}

async fn shoot(app: &mut App, dir: &str, name: &str) {
    app.session = None;
    app.screen = Screen::Menu { cur: 0 };
    screen(app, dir, &format!("theme-{name}-menu"), 40).await;
    app.mode = Mode::Watch;
    app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
    app.start_fight().await;
    step(app, READY_LIMIT, true);
    step(app, FIGHT_TICKS, false);
    if let Some(s) = app.session.as_mut() {
        s.paused = true;
    }
    app.pause_preview(0, 0, &[]);
    screen(app, dir, &format!("theme-{name}-pause"), 30).await;
    if let Some(s) = app.session.as_mut() {
        s.paused = false;
    }
    screen(app, dir, &format!("theme-{name}-fight"), 0).await;
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let saved = crate::storage::read(FILE);
    for name in ["default", "emerald", "sunset", "mono"] {
        use_theme(&format!("[theme]\npreset = \"{name}\"\n"));
        shoot(app, dir, name).await;
    }
    use_theme(CUSTOM);
    shoot(app, dir, "custom").await;
    use_theme(saved.as_deref().unwrap_or(crate::tweaks::DEFAULT_TEXT));
}
