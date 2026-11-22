use crate::app::{App, Mode, Screen, Team};
use crate::shots::{find, stage};
use crate::smoke::ctx::Ctx;

const VS_FRAMES: usize = 400;

async fn through_versus(ctx: &mut Ctx, app: &mut App, name: &str) {
    app.screen = Screen::Vs { t: 0.0, step: 0 };
    ctx.shot(app, &format!("{name}-versus"), 20).await;
    let mut reached = false;
    for _ in 0..VS_FRAMES {
        ctx.frames(app, 1).await;
        if matches!(app.screen, Screen::Fight) {
            reached = true;
            break;
        }
    }
    ctx.check(&format!("{name} leaves the versus screen"), reached, format!("within {VS_FRAMES} frames"));
    if reached {
        ctx.shot(app, &format!("{name}-fight-start"), 1).await;
        ctx.shot(app, &format!("{name}-fight-live"), 120).await;
    }
    app.session = None;
}

pub async fn versus(ctx: &mut Ctx, app: &mut App) {
    app.mode = Mode::Cpu;
    app.team = None;
    app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "dojo"));
    through_versus(ctx, app, "1v1").await;
}

pub async fn arena(ctx: &mut Ctx, app: &mut App) {
    app.mode = Mode::PowerArena;
    app.team = None;
    app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "dojo"));
    through_versus(ctx, app, "arena").await;
}

pub async fn team7(ctx: &mut Ctx, app: &mut App) {
    let ids = ["kaito", "lian", "kaname", "akio", "shion", "suzume", "bruno"];
    let side_a: Vec<usize> = ids.iter().map(|id| find(app, id)).collect();
    let side_b: Vec<usize> = ids.iter().rev().map(|id| find(app, id)).collect();
    app.mode = Mode::TeamSimul;
    app.team_n = 7;
    app.sel = (side_a[0], side_b[0], stage(app, "dojo"));
    app.team = Some(Team::new(side_a, side_b, true));
    app.team_pal = vec![app.side_pal(0)];
    through_versus(ctx, app, "team7").await;
    app.team = None;
}
