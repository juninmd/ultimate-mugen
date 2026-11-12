use crate::app::{App, Mode, Screen, Team};
use crate::gfx::*;
use crate::session::Pilot;
use crate::shots::{find, stage};
use crate::story::STAGES;
use ume_engine::ai::Ai;

const FIGHT_WARMUP_TICKS: usize = 150;
const FIGHT_LIMIT_TICKS: usize = 7200;
const PLAYER_LIFE_FACTOR: f32 = 3.0;
const PLAYER_ATTACK_FACTOR: f32 = 3.0;

async fn frames(app: &mut App, n: usize) {
    for _ in 0..n {
        app.frame().await;
        next_frame().await;
    }
}

async fn cap_screen(app: &mut App, dir: &str, name: &str) {
    frames(app, 30).await;
    app.trans = 0.0;
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/{name}.png"));
    println!("shot {name}");
    next_frame().await;
}

async fn cap_fight(app: &mut App, dir: &str, name: &str) {
    for i in 0..2 {
        app.draw_session_only();
        app.story_hud();
        if i == 1 {
            get_screen_data().export_png(&format!("{dir}/{name}.png"));
            println!("shot {name}");
        }
        next_frame().await;
    }
}

fn automate_player(app: &mut App, strong: bool) {
    let Some(s) = app.session.as_mut() else { return };
    s.pilots[0] = Pilot::Cpu(Ai::new(7, 5));
    if !strong {
        return;
    }
    for f in s.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 0) {
        f.life_max *= PLAYER_LIFE_FACTOR;
        f.life *= PLAYER_LIFE_FACTOR;
        f.attack_mul *= PLAYER_ATTACK_FACTOR;
    }
}

fn step(app: &mut App, ticks: usize, until_over: bool) -> Option<usize> {
    let App { session, c, audio, .. } = app;
    let s = session.as_mut()?;
    for _ in 0..ticks {
        s.step_visual(c, audio);
        if until_over && s.over.is_some() {
            break;
        }
    }
    s.over
}

fn foes_alive(app: &App) -> usize {
    app.session
        .as_ref()
        .map(|s| s.m.fighters.iter().filter(|f| !f.is_helper && f.side == 1 && f.alive).count())
        .unwrap_or(0)
}

fn place_hero(app: &mut App, lane: f32) {
    if let Some(s) = app.session.as_mut() {
        if let Some(hero) = s.m.fighters.iter_mut().find(|f| !f.is_helper && f.side == 0) {
            hero.lane = lane;
        }
    }
}

async fn cap_lane_extremes(app: &mut App, dir: &str, name: &str, far: f32, near: f32) {
    place_hero(app, far);
    cap_fight(app, dir, &format!("{name}-lane-far")).await;
    place_hero(app, near);
    cap_fight(app, dir, &format!("{name}-lane-near")).await;
    place_hero(app, 0.0);
}

async fn cap_guard(app: &mut App, dir: &str, name: &str) {
    if let Some(s) = app.session.as_mut() {
        s.pilots[0] = Pilot::Replay(vec![ume_engine::input::k::GUARD; FIGHT_LIMIT_TICKS], 0);
    }
    for _ in 0..FIGHT_LIMIT_TICKS {
        step(app, 1, false);
        let guarding = app.session.as_ref().is_some_and(|s| {
            s.m.fighters.iter().any(|f| !f.is_helper && f.side == 0 && (120..=155).contains(&f.state_no))
        });
        if guarding {
            break;
        }
    }
    cap_fight(app, dir, &format!("{name}-guard")).await;
}

fn wait_ready(app: &mut App) {
    for _ in 0..400 {
        let ready = app
            .session
            .as_ref()
            .is_some_and(|s| s.m.round_state == 2 && !s.m.is_lane_scrolling());
        if ready {
            break;
        }
        step(app, 1, false);
    }
}

async fn play_fight(app: &mut App, dir: &str, name: &str, strong: bool, band: Option<(f32, f32)>) -> bool {
    if app.session.is_none() {
        app.start_fight().await;
        automate_player(app, strong);
    } else if let Some(s) = app.session.as_mut() {
        s.pilots[0] = Pilot::Cpu(Ai::new(7, 5));
    }
    if let Some((far, near)) = band {
        wait_ready(app);
        cap_lane_extremes(app, dir, name, far, near).await;
        cap_guard(app, dir, name).await;
        app.session = None;
        app.start_fight().await;
        automate_player(app, strong);
    }
    wait_ready(app);
    let warmup = if name.ends_with("-boss") { FIGHT_WARMUP_TICKS } else { 15 };
    step(app, warmup, true);
    println!("{name}: {} foe(s) alive at capture", foes_alive(app));
    cap_fight(app, dir, name).await;
    let over = step(app, FIGHT_LIMIT_TICKS, true);
    if over == Some(0) {
        step(app, 10, false);
        cap_fight(app, dir, &format!("{name}-ko")).await;
    }
    over == Some(0)
}

async fn npc_lineup(app: &mut App, dir: &str) {
    let hero = find(app, "kaito");
    let ids = ["npc_brute", "npc_razor", "npc_spitfire", "npc_colossus", "npc_phantom"];
    let foes: Vec<usize> = ids.iter().map(|id| find(app, id)).collect();
    app.mode = Mode::Watch;
    app.sel = (hero, foes[0], stage(app, "brazil"));
    app.team = Some(Team::new(vec![hero], foes, true));
    app.team_pal = vec![app.side_pal(0)];
    app.start_fight().await;
    step(app, 400, false);
    step(app, 20, false);
    cap_fight(app, dir, "story-npc-lineup").await;
    app.team = None;
}

pub async fn run(app: &mut App, dir: &str) {
    let hero = find(app, "kaito");
    app.screen = Screen::Menu { cur: 0 };
    cap_screen(app, dir, "story-00-menu").await;
    npc_lineup(app, dir).await;

    app.mode = Mode::Story;
    app.sel.0 = hero;
    app.story_begin(hero);
    let mut cleared_all = true;
    for (index, def) in STAGES.iter().enumerate() {
        app.story_show_intro();
        cap_screen(app, dir, &format!("story-s{}-intro", index + 1)).await;
        app.story_resume();
        let waves = def.waves.len();
        for wave in 0..=waves {
            let label = if wave == waves { "boss".to_string() } else { format!("wave{}", wave + 1) };
            let name = format!("story-s{}-{label}", index + 1);
            let band = (wave == 0).then_some((def.lane_far, def.lane_near));
            let won = play_fight(app, dir, &name, true, band).await;
            if !won {
                cleared_all = false;
                println!("{name}: player lost");
            }
            app.story_win();
            if wave < waves {
                frames(app, 2).await;
            }
        }
        if index + 1 < STAGES.len() {
            cap_screen(app, dir, &format!("story-s{}-cleared", index + 1)).await;
        }
    }
    cap_screen(app, dir, "story-99-victory").await;
    println!("story run finished, every fight won by the player: {cleared_all}");

    app.story_begin(hero);
    app.story_resume();
    app.start_fight().await;
    automate_player(app, false);
    if let Some(s) = app.session.as_mut() {
        for f in s.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 0) {
            f.life = 1.0;
        }
    }
    step(app, FIGHT_LIMIT_TICKS, true);
    app.story_loss();
    cap_screen(app, dir, "story-98-game-over").await;
}
