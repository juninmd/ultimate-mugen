use super::{find, screen, stage};
use crate::app::{arena_setup, App, Mode, Screen, Team};
use crate::gfx::*;
use std::time::Instant;

fn env_n(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::PowerArena;
    app.arena.squad = 3;
    app.arena.total = 100;
    app.screen = Screen::ArenaSetup { row: 3 };
    screen(app, dir, "arena-setup", 2).await;

    let squad: Vec<usize> = ["kaito", "lian", "suzume"].iter().map(|id| find(app, id)).collect();
    let slots = env_n("UME_ARENA_SLOTS", arena_setup::active_cap());
    let pool_n = env_n("UME_ARENA_POOL", arena_setup::pool_cap());
    let pool: Vec<usize> = (0..app.reg.chars.len())
        .filter(|i| !app.reg.chars[*i].mugen && !app.reg.chars[*i].boss && !squad.contains(i))
        .take(pool_n)
        .collect();
    let foes: Vec<usize> = (0..slots).map(|k| pool[k % pool.len()]).collect();
    app.team_n = squad.len();
    app.arena.total = env_n("UME_ARENA_TOTAL", 100);
    app.team = Some(Team::new(squad.clone(), foes.clone(), true));
    app.sel = (squad[0], foes[0], stage(app, "dojo"));
    let load = Instant::now();
    app.start_fight().await;
    println!("arena: load {:.0} ms · squad {} · slots {} · pool {}", load.elapsed().as_secs_f64() * 1000.0, squad.len(), slots, pool_n);
    {
        let App { session, c, audio, .. } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..900 {
            if s.m.round_state == 2 && s.m.fighters[0].ctrl {
                break;
            }
            s.step_visual(c, audio);
        }
    }
    screen(app, dir, "arena-start", 1).await;

    let (mut step, mut draw, mut worst, mut upload_kb) = (0.0f32, 0.0f32, 0.0f32, 0.0f64);
    let frames = 600;
    for i in 0..frames {
        let t = Instant::now();
        {
            let App { session, c, audio, .. } = app;
            let s = session.as_mut().unwrap();
            if i % 45 == 0 {
                for f in s.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 1 && f.alive).take(3) {
                    f.life = 0.0;
                }
            }
            s.step_visual(c, audio);
        }
        let st = t.elapsed().as_secs_f32() * 1000.0;
        let t = Instant::now();
        app.draw_session_only();
        let dr = t.elapsed().as_secs_f32() * 1000.0;
        next_frame().await;
        upload_kb += stats().upload_kb as f64;
        step += st;
        draw += dr;
        worst = worst.max(st + dr);
    }
    let stats = stats();
    let s = app.session.as_ref().unwrap();
    let swarm = s.m.swarm.as_ref().unwrap();
    println!(
        "arena: {} frames · step avg {:.2} ms · draw avg {:.2} ms · worst {:.1} ms · atlas rgba {:.0}% index {:.0}% · resets {} · up {:.0} KB/frame · kills {} spawned {} · map {:.0} px · fighters {}",
        frames,
        step / frames as f32,
        draw / frames as f32,
        worst,
        stats.rgba_fill * 100.0,
        stats.index_fill * 100.0,
        stats.rgba_resets,
        upload_kb / frames as f64,
        swarm.kills,
        swarm.spawned,
        s.m.arena_half * 2.0,
        s.m.fighters.len()
    );
    screen(app, dir, "arena-fight", 1).await;
    app.c.touch.enabled = true;
    screen(app, dir, "arena-fight-touch", 1).await;
    app.c.touch.enabled = false;
    {
        let App { session, .. } = app;
        let s = session.as_mut().unwrap();
        let far = s.m.arena_half * 0.8;
        for f in s.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 0) {
            f.pos.x = far;
        }
    }
    screen(app, dir, "arena-far-edge", 40).await;
    {
        let App { session, .. } = app;
        let s = session.as_mut().unwrap();
        let e = s.m.elixir.as_ref().unwrap().clone();
        let f = &mut s.m.fighters[0];
        f.pos.x = e.x - 90.0;
        f.pos.y = e.y;
        f.unhittable = 600;
    }
    screen(app, dir, "arena-elixir", 60).await;
    {
        let App { session, .. } = app;
        let s = session.as_mut().unwrap();
        let (x, y) = { let e = s.m.elixir.as_ref().unwrap(); (e.x, e.y) };
        let f = &mut s.m.fighters[0];
        f.pos.x = x;
        f.pos.y = y;
        println!("arena: life_max before {}", f.life_max);
    }
    screen(app, dir, "arena-elixir-taken", 6).await;
    println!("arena: life_max after {}", app.session.as_ref().unwrap().m.fighters[0].life_max);
}
