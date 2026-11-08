use crate::app::{App, Mode, Screen};
use crate::gfx::*;
use crate::session::Pilot;
use crate::shots::{find, stage};
use ume_engine::ai::Ai;

async fn frames(app: &mut App, n: usize) {
    for _ in 0..n {
        app.frame().await;
        next_frame().await;
    }
}

async fn cap_screen(app: &mut App, dir: &str, name: &str, warm: usize) {
    frames(app, warm).await;
    app.trans = 0.0;
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/{name}.png"));
    println!("shot {name}");
    next_frame().await;
}

async fn cap_fight(app: &mut App, dir: &str, name: &str) {
    for i in 0..2 {
        app.draw_session_only();
        if i == 1 {
            get_screen_data().export_png(&format!("{dir}/{name}.png"));
            println!("shot {name}");
        }
        next_frame().await;
    }
}

pub async fn run(app: &mut App, dir: &str) {
    let kaito = find(app, "kaito");
    let suzume = find(app, "suzume");
    let akio = find(app, "akio");
    let oboro = find(app, "oboro");

    app.mode = Mode::Arcade;
    app.sel = (kaito, oboro, stage(app, "volcano"));
    app.arcade = Some((vec![oboro, oboro], 0));
    app.ending_shown = false;

    app.screen = Screen::Cutscene {
        kind: 0,
        page: 3,
        t: 5.0,
    };
    cap_screen(app, dir, "story-01-opening-kaito", 35).await;

    app.screen = Screen::Cutscene {
        kind: 1,
        page: 3,
        t: 5.0,
    };
    cap_screen(app, dir, "story-02-preboss-oboro", 35).await;

    app.mode = Mode::Watch;
    app.sel = (kaito, oboro, stage(app, "volcano"));
    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..600 {
            if s.m.round_state == 2 && s.m.fighters[1].ctrl {
                break;
            }
            s.step_visual(c, audio);
        }
        s.m.fighters[1].power = 3000.0;
        s.m.fighters[1].life = s.m.fighters[1].life_max * 0.55;
        s.m.try_transform(1);
        for _ in 0..110 {
            s.step_visual(c, audio);
        }
    }
    cap_fight(app, dir, "story-03-boss-phase1-mangekyo").await;

    app.session = None;
    app.mode = Mode::Arcade;
    app.sel = (kaito, oboro, stage(app, "spirit"));
    app.arcade = Some((vec![oboro, oboro], 0));
    app.ending_shown = false;
    app.screen = Screen::Results { winner: 0, cur: 0 };
    frames(app, 2).await;
    if let Screen::Cutscene { kind: 3, .. } = app.screen {
        app.screen = Screen::Cutscene {
            kind: 3,
            page: 3,
            t: 5.0,
        };
    }
    cap_screen(app, dir, "story-04-boss-transform-cutscene", 35).await;

    app.screen = Screen::Cutscene {
        kind: 3,
        page: 4,
        t: 5.0,
    };
    cap_screen(app, dir, "story-05-hero-challenge-eclipse", 25).await;

    app.screen = Screen::Vs { t: 0.0, step: 0 };
    frames(app, 35).await;
    app.screen = Screen::Vs { t: 1.35, step: 0 };
    cap_screen(app, dir, "story-06-vs-final-challenge", 2).await;

    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        s.pilots[0] = Pilot::Cpu(Ai::new(7, 21));
        for _ in 0..278 {
            s.step_visual(c, audio);
        }
    }
    cap_fight(app, dir, "story-07-boss-phase2-banner").await;

    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..600 {
            if s.m.round_state == 2 && s.m.fighters[1].ctrl {
                break;
            }
            s.step_visual(c, audio);
        }
        s.m.fighters[0].pos.x = -55.0;
        s.m.fighters[1].pos.x = 55.0;
        s.m.fighters[1].power = 3000.0;
        let d = s.m.fighters[1].def.clone();
        let st = if d.states.contains_key(&3002) { 3002 } else { 3000 };
        if d.states.contains_key(&st) {
            s.m.change_state(1, st, d);
        }
        for _ in 0..42 {
            s.step_visual(c, audio);
        }
    }
    cap_fight(app, dir, "story-08-boss-eclipse-super").await;

    app.session = None;
    app.mode = Mode::Arcade;
    app.sel = (kaito, oboro, stage(app, "spirit"));
    app.arcade = Some((vec![oboro, oboro], 1));
    app.ending_shown = false;
    app.screen = Screen::Results { winner: 0, cur: 0 };
    frames(app, 2).await;
    app.screen = Screen::Cutscene {
        kind: 2,
        page: 1,
        t: 5.0,
    };
    cap_screen(app, dir, "story-09-ending-kaito-quote", 30).await;

    app.screen = Screen::Cutscene {
        kind: 2,
        page: 3,
        t: 5.0,
    };
    cap_screen(app, dir, "story-10-ending-kaito-epilogue", 25).await;

    app.sel.0 = suzume;
    app.screen = Screen::Cutscene {
        kind: 2,
        page: 1,
        t: 5.0,
    };
    cap_screen(app, dir, "story-11-ending-suzume", 30).await;

    app.sel.0 = akio;
    app.screen = Screen::Cutscene {
        kind: 2,
        page: 1,
        t: 5.0,
    };
    cap_screen(app, dir, "story-12-ending-akio", 30).await;

    app.cutscene_done(2);
    cap_screen(app, dir, "story-13-returned-to-main-menu", 30).await;
}
