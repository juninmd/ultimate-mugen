use super::{find, screen, stage};
use crate::app::{App, Mode};

pub(crate) async fn suzume_abilities_tour(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);

    // 1. Pause Menu / Movelist with Power Badges
    app.mode = Mode::Watch;
    app.sel = (
        find(app, "suzume"),
        find(app, "shion"),
        stage(app, "bamboo"),
    );
    app.start_fight().await;
    if let Some(s) = app.session.as_mut() {
        s.paused = true;
    }
    screen(app, dir, "suzume-pause-movelist", 2).await;

    // 2. Flight (Voo Parametrizavel)
    if let Some(s) = app.session.as_mut() {
        s.paused = false;
        s.m.round_state = 2;
        s.m.intro_ticks = 0;
        s.m.fighters[0].pos.x = -60.0;
        s.m.fighters[0].pos.y = -65.0;
        s.m.fighters[0].vel.x = 2.0;
        s.m.fighters[0].vel.y = 0.0;
        s.m.fighters[0].state_no = 3900;
        s.m.fighters[0].state_type = 'A';
        s.m.fighters[0].ctrl = true;
        s.m.fighters[0].power = 2800.0;
        let def = s.m.fighters[0].def.clone();
        s.m.fighters[0].change_anim(41, &def, 1);
        s.m.fighters[1].pos.x = 80.0;
        s.m.fighters[1].pos.y = 0.0;
    }
    for _ in 0..12 {
        if let Some(s) = app.session.as_mut() {
            s.step_visual(&mut app.c, &mut app.audio);
        }
    }
    screen(app, dir, "suzume-flight", 2).await;

    // 3. Teleport Behind Opponent (Passo das Sombras)
    if let Some(s) = app.session.as_mut() {
        s.m.fighters[0].pos.x = 115.0;
        s.m.fighters[0].pos.y = 0.0;
        s.m.fighters[0].facing = -1.0;
        s.m.fighters[0].state_no = 3910;
        s.m.fighters[0].state_time = 5;
        let def = s.m.fighters[0].def.clone();
        s.m.fighters[0].change_anim(def.anims.get(&3910).map(|_| 3910).unwrap_or(0), &def, 1);
    }
    for _ in 0..4 {
        if let Some(s) = app.session.as_mut() {
            s.step_visual(&mut app.c, &mut app.audio);
        }
    }
    screen(app, dir, "suzume-teleport", 2).await;

    // 4. Super Speed (Shunshin / AfterImage)
    if let Some(s) = app.session.as_mut() {
        s.m.fighters[0].pos.x = -80.0;
        s.m.fighters[0].facing = 1.0;
        s.m.fighters[0].vars[58] = 1;
        s.m.fighters[0].vel.x = 10.0;
        let def = s.m.fighters[0].def.clone();
        s.m.fighters[0].change_anim(100, &def, 1);
    }
    for _ in 0..8 {
        if let Some(s) = app.session.as_mut() {
            s.step_visual(&mut app.c, &mut app.audio);
        }
    }
    screen(app, dir, "suzume-superspeed", 2).await;

    // 5. Transformation: Form 1 - Lotus da Noite (Partial Transformation)
    if let Some(s) = app.session.as_mut() {
        s.m.fighters[0].form = 0;
        s.m.fighters[0].vars[58] = 0;
        s.m.fighters[0].pos.x = -60.0;
        s.m.fighters[0].pos.y = 0.0;
        s.m.fighters[0].facing = 1.0;
        s.m.fighters[0].power = 3000.0;
        s.m.fighters[0].ctrl = true;
        s.m.try_transform(0);
    }
    for _ in 0..36 {
        if let Some(s) = app.session.as_mut() {
            s.step_visual(&mut app.c, &mut app.audio);
        }
    }
    screen(app, dir, "suzume-form-lotus-noite", 2).await;

    // 6. Transformation: Form 2 - Sombra Carmesim (Full Body Transformation)
    if let Some(s) = app.session.as_mut() {
        s.m.fighters[0].power = 3000.0;
        s.m.fighters[0].ctrl = true;
        s.m.fighters[0].state_type = 'S';
        s.m.try_transform(0);
    }
    for _ in 0..36 {
        if let Some(s) = app.session.as_mut() {
            s.step_visual(&mut app.c, &mut app.audio);
        }
    }
    screen(app, dir, "suzume-form-sombra-carmesim", 2).await;

    // 7. Transformation: Form 3 - Metamorfose em Mikazuki Oboro (Character Swap)
    if let Some(s) = app.session.as_mut() {
        s.m.fighters[0].power = 3000.0;
        s.m.fighters[0].ctrl = true;
        s.m.fighters[0].state_type = 'S';
        s.m.try_transform(0);
    }
    for _ in 0..36 {
        if let Some(s) = app.session.as_mut() {
            s.step_visual(&mut app.c, &mut app.audio);
        }
    }
    screen(app, dir, "suzume-form-oboro", 2).await;
}
