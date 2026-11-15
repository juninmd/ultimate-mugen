use super::{find, screen, stage};
use crate::app::{App, Mode};
use crate::session::Pilot;
use ume_engine::input::k;

const FIGHTER: &str = "mika";
const INPUT_TICKS: usize = 600;
const READY_LIMIT: usize = 600;

fn step(app: &mut App, ticks: usize) {
    let App {
        session, c, audio, ..
    } = app;
    if let Some(s) = session.as_mut() {
        for _ in 0..ticks {
            s.step_visual(c, audio);
        }
    }
}

fn script(app: &mut App, motion: &[u32], gap: usize) {
    let mut bits = Vec::with_capacity(INPUT_TICKS);
    while bits.len() < INPUT_TICKS {
        for &held in motion {
            bits.extend(std::iter::repeat_n(held, 2));
        }
        bits.extend(std::iter::repeat_n(0, gap));
    }
    if let Some(s) = app.session.as_mut() {
        s.pilots[0] = Pilot::Replay(bits, 0);
        s.pilots[1] = Pilot::Replay(vec![0; INPUT_TICKS], 0);
    }
}

fn wait_for_projectile(app: &mut App, limit: usize) {
    for _ in 0..limit {
        if app
            .session
            .as_ref()
            .is_some_and(|s| s.m.fighters.iter().any(|f| f.is_helper))
        {
            break;
        }
        step(app, 1);
    }
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let fighter = find(app, FIGHTER);
    if !app.reg.chars[fighter].id.ends_with(FIGHTER) {
        eprintln!("umfkit tour: run with UME_UMF_DIR=docs/examples/umf");
        return;
    }
    app.mode = Mode::Watch;
    app.sel = (fighter, find(app, "kaito"), stage(app, "dojo"));
    app.pal[0] = 2;
    app.start_fight().await;
    if let Some(s) = app.session.as_mut() {
        s.paused = true;
    }
    app.pause_preview(1, 0, &[]);
    screen(app, dir, "kit-01-inherited-movelist", 30).await;
    if let Some(s) = app.session.as_mut() {
        s.paused = false;
    }
    for _ in 0..READY_LIMIT {
        if app.session.as_ref().is_none_or(|s| s.m.round_state == 2) {
            break;
        }
        step(app, 1);
    }
    script(app, &[k::D, k::D | k::R, k::R, k::R | k::X], 50);
    wait_for_projectile(app, 120);
    step(app, 8);
    screen(app, dir, "kit-02-renamed-special", 0).await;
    step(app, 90);
    script(app, &[k::D, k::D | k::R, k::R, k::R | k::X | k::A], 50);
    if let Some(hero) = app.session.as_mut().and_then(|s| s.m.fighters.first_mut()) {
        hero.power = hero.power.max(2000.0);
    }
    wait_for_projectile(app, 120);
    step(app, 8);
    screen(app, dir, "kit-03-cloned-ex-move", 0).await;
}
