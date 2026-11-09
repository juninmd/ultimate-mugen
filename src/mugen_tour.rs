//! Visual probe for imported M.U.G.E.N. fighters: `UME_TOUR=mugen UME_CHARS=a,b UME_STAGE=dojo ultimate-mugen --shots <dir>`.
//! Captures the select grid, then CPU-vs-CPU frames whenever a projectile / effect / airborne hit is on screen.
use crate::app::{App, Mode, Screen};
use crate::gfx::*;
use crate::shots::{find, stage};

async fn grab(app: &mut App, dir: &str, name: &str) {
    for i in 0..2 {
        app.draw_session_only();
        if i == 1 {
            get_screen_data().export_png(&format!("{dir}/{name}.png"));
            println!("shot {name}");
        }
        next_frame().await;
    }
}

async fn catalog_tour(app: &mut App, dir: &str) {
    for (name, view, page) in [("catalog-gallery", 0u8, 0usize), ("catalog-quarantine", 1, 0)] {
        app.screen = Screen::Mugen { view, page };
        for _ in 0..600 {
            app.frame().await;
            next_frame().await;
        }
        app.trans = 0.0;
        app.frame().await;
        get_screen_data().export_png(&format!("{dir}/{name}.png"));
        println!("shot {name}");
        next_frame().await;
    }
}

pub async fn run(app: &mut App, dir: &str) {
    if std::env::var_os("UME_MUGEN_CATALOG").is_some() {
        return catalog_tour(app, dir).await;
    }
    match std::env::var("UME_TOUR").as_deref() {
        Ok("moves") => return crate::moves_tour::run(app, dir).await,
        Ok("duel") => return crate::moves_tour::duel::run(app, dir).await,
        Ok("clips") => return crate::moves_tour::clips::run(app, dir).await,
        _ => {}
    }
    let chars: Vec<String> = std::env::var("UME_CHARS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if chars.len() < 2 {
        println!("set UME_CHARS=a,b (suffix of the registry id)");
        return;
    }
    if let Some(missing) = chars
        .iter()
        .take(2)
        .find(|c| !app.reg.chars.iter().any(|r| r.id.ends_with(c.as_str())))
    {
        println!("no registry id ends with '{missing}' (ids end in .def, e.g. ichiji/ichiji.def)");
        return;
    }
    let (a, b) = (find(app, &chars[0]), find(app, &chars[1]));
    // select grid: let the thumbnails stream in
    let n = app.reg.chars.len();
    app.mode = Mode::Cpu;
    app.screen = Screen::Select {
        cur: [a, b],
        picking: 0,
        picked: [None, None],
    };
    for _ in 0..(n * 4).min(900) {
        app.frame().await;
        next_frame().await;
    }
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/m-select.png"));
    println!("shot m-select");
    next_frame().await;
    // fight
    app.mode = Mode::Watch;
    app.sel = (
        a,
        b,
        stage(
            app,
            &std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into()),
        ),
    );
    app.start_fight().await;
    let (mut shots, mut last) = (0, -999i64);
    for t in 0..(60 * 60) {
        {
            let App {
                session, c, audio, ..
            } = app;
            let s = session.as_mut().unwrap();
            s.step_visual(c, audio);
            let air_hit =
                s.m.fighters
                    .iter()
                    .any(|f| f.move_type == 'H' && f.pos.y < -20.0 && f.hit_shake == 0);
            let fx = !s.m.projectiles.is_empty() || s.m.explods.len() > 2;
            if (air_hit || fx) && (t as i64 - last) > 90 && shots < 14 {
                last = t as i64;
                shots += 1;
            } else {
                continue;
            }
        }
        grab(app, dir, &format!("m-fight-{shots:02}")).await;
    }
    // pause screen (move list) and the keyboard rebinding screen
    app.session.as_mut().unwrap().paused = true;
    for _ in 0..4 {
        app.frame().await;
        next_frame().await;
    }
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/m-pause.png"));
    println!("shot m-pause");
    next_frame().await;
    app.session.as_mut().unwrap().paused = false;
    app.screen = Screen::Controls;
    app.key_edit = Some(crate::app::KeyEdit {
        slot: 0,
        cur: 4,
        waiting: false,
        msg: String::new(),
    });
    for _ in 0..45 {
        app.frame().await;
        next_frame().await;
    }
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/m-keys.png"));
    println!("shot m-keys");
}
