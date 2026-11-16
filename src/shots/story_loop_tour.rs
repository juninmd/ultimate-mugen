use crate::app::{App, Mode};
use crate::gfx::*;
use crate::session::Pilot;
use crate::shots::find;
use crate::story::STAGES;

const READY_LIMIT: usize = 600;
const IDLE_TICKS: usize = 4000;
const SETTLE_TICKS: usize = 150;
const LOOP_FRAMES: usize = 32;
const LOOP_TICKS_PER_FRAME: usize = 2;
const SHOWN_WAVES: [usize; 3] = [0, 2, 4];

async fn cap(app: &mut App, path: &str) {
    for pass in 0..2 {
        app.draw_session_only();
        app.story_hud();
        if pass == 1 {
            get_screen_data().export_png(path);
            println!("shot {path}");
        }
        next_frame().await;
    }
}

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

fn settle(app: &mut App) {
    for _ in 0..READY_LIMIT {
        if app.session.as_ref().is_none_or(|s| s.m.round_state == 2) {
            break;
        }
        step(app, 1);
    }
    if let Some(s) = app.session.as_mut() {
        for pilot in s.pilots.iter_mut() {
            *pilot = Pilot::Replay(vec![0; IDLE_TICKS], 0);
        }
    }
    step(app, SETTLE_TICKS);
}

pub async fn run(app: &mut App, dir: &str) {
    let hero = find(app, "kaito");
    app.mode = Mode::Story;
    app.sel.0 = hero;
    app.story_begin(hero);
    app.story_resume();
    let burst = std::env::var("UME_TOUR_BURST").is_ok();
    for (index, stage) in STAGES.iter().enumerate() {
        for wave in SHOWN_WAVES {
            app.session = None;
            app.story_jump_to_stage(index);
            app.story_jump_to_wave(wave);
            app.start_fight().await;
            settle(app);
            let name = format!("{}-{}-fight-{}", index + 1, stage.arena, wave + 1);
            cap(app, &format!("{dir}/loop-{name}.png")).await;
            if burst && wave == 2 {
                for frame in 0..LOOP_FRAMES {
                    step(app, LOOP_TICKS_PER_FRAME);
                    cap(app, &format!("{dir}/burst-{name}-{frame:03}.png")).await;
                }
            }
        }
    }
}
