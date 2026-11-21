use crate::app::{App, Mode, Screen};
use crate::shots::{find, stage};
use crate::smoke::ctx::Ctx;

const INTRO_TICKS: usize = 900;
const COMBAT_TICKS: usize = 5400;
const KO_TICKS: usize = 1500;
const RESULTS_FRAMES: usize = 1500;

pub async fn run(ctx: &mut Ctx, app: &mut App) {
    app.mode = Mode::Watch;
    app.team = None;
    app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "dojo"));
    app.start_fight().await;
    let loaded = app.session.is_some() && matches!(app.screen, Screen::Fight);
    ctx.check("fight loads", loaded, app.msg.clone());
    if !loaded {
        return;
    }
    let (ready, damaged) = {
        let App { session, c, audio, .. } = app;
        let Some(s) = session.as_mut() else { return };
        let ready = (0..INTRO_TICKS).any(|_| {
            s.step_visual(c, audio);
            s.m.round_state == 2
        });
        let damaged = ready
            && (0..COMBAT_TICKS).any(|_| {
                s.step_visual(c, audio);
                s.m.fighters.iter().any(|f| !f.is_helper && f.life < f.life_max)
            });
        (ready, damaged)
    };
    ctx.check("round intro reaches fighting state", ready, format!("within {INTRO_TICKS} ticks"));
    ctx.check("combat deals damage", damaged, format!("within {COMBAT_TICKS} ticks"));
    ctx.shot(app, "fight", 2).await;
    let over = knock_out(app);
    ctx.check("match ends on KO", over, format!("within {KO_TICKS} ticks"));
    let mut reached = false;
    for _ in 0..RESULTS_FRAMES {
        ctx.frames(app, 1).await;
        if matches!(app.screen, Screen::Results { .. }) {
            reached = true;
            break;
        }
    }
    ctx.check("results screen follows the KO", reached, format!("within {RESULTS_FRAMES} frames"));
    if reached {
        ctx.shot(app, "results", 30).await;
    }
    app.session = None;
}

fn knock_out(app: &mut App) -> bool {
    let App { session, c, audio, .. } = app;
    let Some(s) = session.as_mut() else { return false };
    s.m.wins = [1, 0];
    if let Some(foe) = s.m.fighters.iter_mut().find(|f| !f.is_helper && f.side == 1) {
        foe.life = 0.0;
    }
    (0..KO_TICKS).any(|_| {
        s.step_visual(c, audio);
        s.over.is_some()
    })
}
