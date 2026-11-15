use crate::app::{App, Mode};
use crate::gfx::*;
use crate::session::Pilot;
use crate::shots::find;
use ume_engine::input::k;

const STEP_LIMIT: usize = 3600;
const PARKED_FOE_OFFSET: f32 = 260.0;

async fn cap(app: &mut App, dir: &str, name: &str) {
    for pass in 0..2 {
        app.draw_session_only();
        app.story_hud();
        if pass == 1 {
            get_screen_data().export_png(&format!("{dir}/{name}.png"));
            println!("shot {name}");
        }
        next_frame().await;
    }
}

async fn burst(app: &mut App, dir: &str, name: &str, frames: usize, ticks_per_frame: usize) {
    if std::env::var("UME_TOUR_BURST").is_err() {
        return;
    }
    for frame in 0..frames {
        step(app, ticks_per_frame);
        app.draw_session_only();
        app.story_hud();
        get_screen_data().export_png(&format!("{dir}/{name}-{frame:03}.png"));
        next_frame().await;
    }
}

fn step_until(
    app: &mut App,
    limit: usize,
    mut done: impl FnMut(&crate::session::Session) -> bool,
) -> bool {
    let App {
        session, c, audio, ..
    } = app;
    let Some(s) = session.as_mut() else {
        return false;
    };
    for _ in 0..limit {
        if done(s) {
            return true;
        }
        s.step_visual(c, audio);
    }
    done(s)
}

fn step(app: &mut App, ticks: usize) {
    let mut left = ticks;
    step_until(app, ticks, |_| {
        let stop = left == 0;
        left = left.saturating_sub(1);
        stop
    });
}

fn script(app: &mut App, pattern: &[u32], hold: usize, gap: usize) {
    let mut bits = Vec::with_capacity(STEP_LIMIT);
    while bits.len() < STEP_LIMIT {
        for &button in pattern {
            bits.extend(std::iter::repeat_n(button, hold));
            bits.extend(std::iter::repeat_n(0, gap));
        }
    }
    if let Some(s) = app.session.as_mut() {
        s.pilots[0] = Pilot::Replay(bits, 0);
    }
}

fn park_foes(app: &mut App) {
    let Some(s) = app.session.as_mut() else {
        return;
    };
    let x = s.m.stage.start_x + PARKED_FOE_OFFSET;
    for pilot in s.pilots.iter_mut().skip(1) {
        *pilot = Pilot::Replay(vec![0; STEP_LIMIT * 4], 0);
    }
    for f in
        s.m.fighters
            .iter_mut()
            .filter(|f| !f.is_helper && f.side == 1)
    {
        f.pos.x = x;
    }
}

fn stand_by_first_drop(app: &mut App) {
    let Some(s) = app.session.as_mut() else {
        return;
    };
    let Some((x, lane, reach)) =
        s.m.objects
            .breakables
            .iter()
            .find(|b| b.drop.is_some())
            .map(|b| (b.x, b.lane, b.half_width))
    else {
        return;
    };
    if let Some(hero) = s.m.fighters.first_mut() {
        hero.pos.x = x - reach - 26.0;
        hero.lane = lane + 5.0;
        hero.facing = 1.0;
        hero.life = hero.life_max * 0.45;
    }
}

fn car_damage(s: &crate::session::Session) -> f32 {
    s.m.objects
        .breakables
        .first()
        .map_or(1.0, |b| b.damage_ratio())
}

pub async fn run(app: &mut App, dir: &str) {
    let hero = find(app, "kaito");
    app.mode = Mode::Story;
    app.sel.0 = hero;
    app.story_begin(hero);
    app.story_resume();
    app.start_fight().await;
    step_until(app, 600, |s| s.m.round_state == 2);
    step(app, 20);
    cap(app, dir, "objects-01-wave-with-crate-and-barrel").await;

    park_foes(app);
    stand_by_first_drop(app);
    script(app, &[k::X, k::Y], 3, 16);
    burst(app, dir, "burst-food", 44, 3).await;
    step_until(app, 400, |s| {
        s.m.objects.breakables.iter().any(|b| b.shake > 5)
    });
    cap(app, dir, "objects-02-crate-struck").await;
    step_until(app, 900, |s| !s.m.objects.pickups.is_empty());
    step(app, 5);
    cap(app, dir, "objects-03-crate-broken-food-pops-out").await;
    step(app, 45);
    cap(app, dir, "objects-04-food-on-the-floor-low-life").await;
    script(app, &[k::R], 6, 0);
    burst(app, dir, "burst-food-walk", 24, 3).await;
    step_until(app, 600, |s| s.m.objects.pickups.is_empty());
    step(app, 6);
    cap(app, dir, "objects-05-food-collected-life-restored").await;

    app.session = None;
    app.story_jump_to_boss();
    app.start_fight().await;
    step_until(app, 600, |s| s.m.round_state == 2);
    if let Some(s) = app.session.as_mut() {
        for f in
            s.m.fighters
                .iter_mut()
                .filter(|f| !f.is_helper && f.side == 1)
        {
            f.life = 0.0;
        }
    }
    step_until(app, STEP_LIMIT, |s| s.over.is_some());
    app.story_win();
    println!("bonus round active: {}", app.story_bonus_active());
    step(app, 30);
    cap(app, dir, "objects-06-bonus-stage-start").await;
    script(app, &[k::Y, k::B, k::X, k::Y], 3, 13);
    burst(app, dir, "burst-car", 110, 4).await;
    for (name, damage) in [
        ("objects-07-car-dented", 0.36),
        ("objects-08-car-smoking", 0.7),
    ] {
        step_until(app, STEP_LIMIT, |s| {
            car_damage(s) >= damage && s.m.objects.breakables.iter().any(|b| b.shake > 5)
        });
        cap(app, dir, name).await;
    }
    step_until(app, STEP_LIMIT, |s| s.m.objects.intact_count() == 0);
    step(app, 4);
    cap(app, dir, "objects-09-car-wrecked").await;
    step(app, 40);
    cap(app, dir, "objects-10-bonus-perfect").await;
    step_until(app, STEP_LIMIT, |s| s.over.is_some());
    app.story_win();
    println!("story after bonus: {:?}", app.story_snapshot());
    for _ in 0..30 {
        app.frame().await;
        next_frame().await;
    }
    app.trans = 0.0;
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/objects-11-stage-cleared.png"));
    next_frame().await;
}
