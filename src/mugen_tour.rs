//! Visual probe for imported M.U.G.E.N. fighters: `UME_TOUR=mugen UME_CHARS=a,b UME_STAGE=dojo ultimate-mugen --shots <dir>`.
//! Captures the select grid, then CPU-vs-CPU frames whenever a projectile / effect / airborne hit is on screen.
use crate::app::{App, Mode, Screen};
use crate::shots::{find, stage};
use macroquad::prelude::*;

async fn grab(app: &mut App, dir: &str, name: &str) {
    for i in 0..2 {
        app.draw_session_only();
        if i == 1 { get_screen_data().export_png(&format!("{dir}/{name}.png")); println!("shot {name}"); }
        next_frame().await;
    }
}

pub async fn run(app: &mut App, dir: &str) {
    let chars: Vec<String> = std::env::var("UME_CHARS").unwrap_or_default().split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    if chars.len() < 2 { println!("set UME_CHARS=a,b (suffix of the registry id)"); return; }
    let (a, b) = (find(app, &chars[0]), find(app, &chars[1]));
    // select grid: let the thumbnails stream in
    let n = app.reg.chars.len();
    app.mode = Mode::Cpu;
    app.screen = Screen::Select { cur: [a, b], picking: 0, picked: [None, None] };
    for _ in 0..(n * 4).min(900) { app.frame().await; next_frame().await; }
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/m-select.png"));
    println!("shot m-select");
    next_frame().await;
    // fight
    app.mode = Mode::Watch;
    app.sel = (a, b, stage(app, &std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into())));
    app.start_fight().await;
    let (mut shots, mut last) = (0, -999i64);
    for t in 0..(60 * 60) {
        {
            let App { session, c, audio, .. } = app;
            let s = session.as_mut().unwrap();
            s.step_visual(c, audio);
            let air_hit = s.m.fighters.iter().any(|f| f.move_type == 'H' && f.pos.y < -20.0 && f.hit_shake == 0);
            let fx = !s.m.projectiles.is_empty() || s.m.explods.len() > 2;
            if (air_hit || fx) && (t as i64 - last) > 90 && shots < 14 { last = t as i64; shots += 1; } else { continue; }
        }
        grab(app, dir, &format!("m-fight-{shots:02}")).await;
    }
}
