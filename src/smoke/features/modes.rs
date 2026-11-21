use crate::app::{App, Mode, Screen, Team};
use crate::shots::{find, stage};
use crate::smoke::ctx::Ctx;

pub async fn select(ctx: &mut Ctx, app: &mut App) {
    app.mode = Mode::Cpu;
    let (kaito, shion) = (find(app, "kaito"), find(app, "shion"));
    app.screen = Screen::Select { cur: [kaito, shion], picking: 0, picked: [None, None] };
    ctx.shot(app, "select-1p", 30).await;
    app.screen = Screen::Select { cur: [kaito, shion], picking: 1, picked: [Some(kaito), None] };
    ctx.shot(app, "select-2p", 30).await;
    app.screen = Screen::Stages { cur: stage(app, "dojo") };
    ctx.shot(app, "stage-select", 30).await;
}

pub async fn training(ctx: &mut Ctx, app: &mut App) {
    app.mode = Mode::Training;
    app.team = None;
    app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
    app.start_fight().await;
    let loaded = app.session.is_some() && matches!(app.screen, Screen::Fight);
    ctx.check("training loads", loaded, app.msg.clone());
    if loaded {
        ctx.shot(app, "training", 90).await;
        let ticking = app.session.as_ref().is_some_and(|s| s.m.game_time > 0);
        ctx.check("simulation advances", ticking, "game_time > 0");
    }
    app.session = None;
}

pub async fn team(ctx: &mut Ctx, app: &mut App) {
    let ids = ["kaito", "lian", "kaname", "akio", "shion"];
    let side_a: Vec<usize> = ids.iter().map(|id| find(app, id)).collect();
    let side_b: Vec<usize> = ids.iter().rev().map(|id| find(app, id)).collect();
    app.mode = Mode::TeamSimul;
    app.team_n = 5;
    app.sel = (side_a[0], side_b[0], stage(app, "dojo"));
    app.team = Some(Team::new(side_a, side_b, true));
    app.team_pal = vec![app.side_pal(0)];
    app.start_fight().await;
    let fighters = app.session.as_ref().map(|s| s.m.fighters.iter().filter(|f| !f.is_helper).count()).unwrap_or(0);
    ctx.check("5v5 loads ten fighters", fighters == 10, format!("{fighters} fighters"));
    if fighters > 0 {
        ctx.shot(app, "team-5v5", 90).await;
    }
    app.team = None;
    app.session = None;
}

pub async fn story(ctx: &mut Ctx, app: &mut App) {
    let hero = find(app, "kaito");
    app.mode = Mode::Story;
    app.sel.0 = hero;
    app.story_begin(hero);
    app.story_show_intro();
    ctx.shot(app, "story-intro", 40).await;
    app.story_resume();
    app.start_fight().await;
    let loaded = app.session.is_some() && matches!(app.screen, Screen::Fight);
    ctx.check("story wave loads", loaded, app.msg.clone());
    if loaded {
        ctx.shot(app, "story-wave", 90).await;
    }
    app.session = None;
}
