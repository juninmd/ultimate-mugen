//! Automated screenshot tour (`--shots <dir>`): renders real frames of every screen and fights.
use crate::app::{App, Mode, Screen, Team};
use macroquad::prelude::*;

async fn frames(app: &mut App, n: usize) { for _ in 0..n { app.frame().await; next_frame().await; } }
async fn screen(app: &mut App, dir: &str, name: &str, warm: usize) {
    frames(app, warm).await;
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/{name}.png"));
    println!("shot {name}");
    next_frame().await;
}
async fn snap(app: &mut App, dir: &str, name: &str, draw: impl Fn(&mut App)) {
    // draw twice (first frame warms textures), capture the second
    for i in 0..2 {
        draw(app);
        if i == 1 { get_screen_data().export_png(&format!("{dir}/{name}.png")); println!("shot {name}"); }
        next_frame().await;
    }
}
fn find(app: &App, id: &str) -> usize { app.reg.chars.iter().position(|c| c.id.ends_with(id)).unwrap_or(0) }
fn stage(app: &App, id: &str) -> usize { app.reg.stages.iter().position(|s| s.id.contains(id)).unwrap_or(0) }

async fn fight(app: &mut App, dir: &str, name: &str, p1: &str, p2: &str, st: &str, ticks: usize, want: &str) {
    app.mode = Mode::Watch;
    app.sel = (find(app, p1), find(app, p2), stage(app, st));
    app.start_fight().await;
    let App { session, c, audio, .. } = app;
    let s = session.as_mut().unwrap();
    for _ in 0..ticks { s.step_visual(c, audio); }
    match want {
        "faceoff" => {}
        "transform" | "awake" => {
            for _ in 0..600 { if s.m.round_state == 2 && s.m.fighters[0].ctrl { break; } s.step_visual(c, audio); }
            s.m.fighters[0].power = 3000.0; s.m.fighters[0].life = s.m.fighters[0].life_max * 0.4;
            s.m.try_transform(0);
            for _ in 0..(if want == "awake" { 140 } else { 34 }) { s.step_visual(c, audio); }
        }
        "defeat" | "defeatfinal" => {
            for _ in 0..600 { if s.m.round_state == 2 { break; } s.step_visual(c, audio); }
            if want == "defeatfinal" { s.m.wins = [1, 0]; }
            for f in s.m.fighters.iter_mut() { f.pos.x = if f.side == 0 { -40.0 } else { 40.0 }; }
            s.m.fighters[1].life = 0.0;
            for _ in 0..1500 { s.step_visual(c, audio); if (5190..=5193).contains(&s.m.fighters[1].state_no) && s.m.fighters[1].state_time >= if want == "defeatfinal" { 96 } else { 78 } { break; } }
        }
        "super" | "super2" | "impact" => {
            for f in s.m.fighters.iter_mut() { f.power = 3000.0; }
            for _ in 0..600 { if s.m.fighters[0].ctrl && s.m.fighters[0].state_no == 0 { break; } s.step_visual(c, audio); }
            let d = s.m.fighters[0].def.clone();
            let no = if want == "super2" { 3001 } else { 3000 };
            if d.states.contains_key(&no) { s.m.change_state(0, no, d); }
            let mut ok = false;
            for _ in 0..1500 { s.step_visual(c, audio); if s.m.pause.as_ref().is_some_and(|p| p.is_super && p.time < 24) { ok = true; break; } }
            if ok { for _ in 0..16 { s.step_visual(c, audio); } }
            println!("{name}: super caught = {ok}");
            if want == "impact" { s.vfx.force_impact(); }
        }
        "hit" => { for _ in 0..1200 { s.step_visual(c, audio); if !s.m.explods.is_empty() && s.m.fighters.iter().any(|f| f.move_type == 'H' && f.hit_shake > 2 && !f.ghv.guarded) { break; } } }
        "proj" => { for _ in 0..1500 { s.step_visual(c, audio); if s.m.projectiles.iter().any(|p| !p.dying && p.time > 8) { break; } } }
        "ko" => {
            if let Some(f) = s.m.fighters.get_mut(1) { f.life = 1.0; }
            for _ in 0..3000 { s.step_visual(c, audio); if s.vfx.ko_active() { break; } }
            for _ in 0..14 { s.step_visual(c, audio); }
        }
        "boxes" => { s.r.debug = true; for _ in 0..900 { s.step_visual(c, audio); if s.m.fighters.iter().any(|f| !f.boxes(1).is_empty()) { break; } } }
        _ => {}
    }
    snap(app, dir, name, |a| a.draw_session_only()).await;
}

/// Team battle shot: `simul` = 2v2 on the field together, otherwise sequence (3v3, second bout with carried life).
async fn team_fight(app: &mut App, dir: &str, name: &str, simul: bool, n: usize, stage_id: &str, ticks: usize, human: bool) {
    let (a_ids, b_ids) = (["kaito", "akio", "lian", "daichi", "jin"], ["suzume", "bruno", "sylene", "garan", "marlow"]);
    let a: Vec<usize> = a_ids.iter().take(n).map(|i| find(app, i)).collect();
    let b: Vec<usize> = b_ids.iter().take(n).map(|i| find(app, i)).collect();
    let mut t = Team::new(a.clone(), b.clone(), simul);
    let mut foe = 0;
    if !simul { t.ko[1][0] = true; t.idx[1] = 1; t.life = [0.62, 1.0]; foe = 1; }
    app.mode = if human { if simul { Mode::TeamSimul } else { Mode::TeamSeq } } else { Mode::Watch };
    app.sel = (a[0], b[foe], stage(app, stage_id));
    app.team = Some(t);
    app.start_fight().await;
    {
        let App { session, c, audio, .. } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..ticks { s.step_visual(c, audio); }
        for _ in 0..1500 { s.step_visual(c, audio); if s.m.fighters.iter().any(|f| f.move_type == 'H' && f.hit_shake > 2) { break; } }
    }
    snap(app, dir, name, |a| a.draw_session_only()).await;
    app.team = None;
}

/// Story / cinematic features: face-off, awakening, defeats, boss, cutscenes, zoomed-out 5v5.
async fn story(app: &mut App, dir: &str) {
    fight(app, dir, "50-faceoff-1", "kaito", "shion", "dojo", 82, "faceoff").await;
    fight(app, dir, "51-faceoff-2", "kaito", "shion", "dojo", 195, "faceoff").await;
    fight(app, dir, "52-transform-start", "valdric", "kaito", "spirit", 100, "transform").await;
    fight(app, dir, "53-transformed", "valdric", "kaito", "spirit", 100, "awake").await;
    fight(app, dir, "54-defeat-round", "kaito", "shion", "dojo", 100, "defeat").await;
    fight(app, dir, "55-defeat-final", "kaito", "shion", "bamboo", 100, "defeatfinal").await;
    fight(app, dir, "56-boss-susanoo", "oboro", "kaito", "volcano", 120, "super").await;
    fight(app, dir, "57-boss-kyoka", "oboro", "kaito", "spirit", 120, "super2").await;
    fight(app, dir, "58-boss-awakened", "oboro", "kaito", "volcano", 100, "awake").await;
    fight(app, dir, "59-boss-fight", "kaito", "oboro", "spirit", 400, "hit").await;
    team_fight(app, dir, "60-5v5-zoomout", true, 5, "dojo", 330, true).await;
    team_fight(app, dir, "61-5v5-zoomout-neon", true, 5, "neon", 420, true).await;
    app.sel = (find(app, "kaito"), find(app, "oboro"), 0);
    app.arcade = Some((vec![find(app, "oboro")], 0));
    for (kind, page, name) in [(0u8, 1usize, "62-cutscene-opening"), (1, 3, "63-cutscene-boss"), (2, 1, "64-cutscene-ending")] {
        app.screen = Screen::Cutscene { kind, page, t: 0.0 };
        screen(app, dir, name, 70).await;
    }
}

pub async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    if std::env::var("UME_TOUR").as_deref() == Ok("story") { story(app, dir).await; return; }
    app.screen = Screen::Title;
    screen(app, dir, "01-title", 30).await;
    let cap = |app: &mut App, s: Screen| { app.screen = s; };
    cap(app, Screen::Menu { cur: 1 });
    screen(app, dir, "02-menu", 10).await;
    app.mode = Mode::Cpu;
    cap(app, Screen::Select { cur: [find(app, "kaname"), find(app, "valdric")], picking: 1, picked: [Some(find(app, "kaname")), None] });
    screen(app, dir, "03-select", 40).await;
    app.mode = Mode::TeamSeq; app.team_pick = vec![find(app, "kaito"), find(app, "lian")];
    cap(app, Screen::Select { cur: [find(app, "akio"), find(app, "valdric")], picking: 0, picked: [None, None] });
    screen(app, dir, "32-team-select", 20).await;
    app.mode = Mode::Cpu; app.team_pick.clear();
    cap(app, Screen::Stages { cur: 2 });
    screen(app, dir, "04-stages", 5).await;
    app.sel = (find(app, "kaito"), find(app, "garan"), 0);
    cap(app, Screen::Vs { t: 0.0, step: 0 });
    screen(app, dir, "05a-vs-loading", 3).await;
    screen(app, dir, "05-vs", 70).await;
    app.screen = Screen::Fight;
    fight(app, dir, "06-fight-dojo", "kaito", "shion", "dojo", 240, "hit").await;
    fight(app, dir, "07-fight-neon", "volta", "suzume", "neon", 300, "proj").await;
    fight(app, dir, "08-fight-super", "akio", "bruno", "spirit", 120, "super").await;
    fight(app, dir, "09-fight-bamboo", "lian", "sylene", "bamboo", 400, "proj").await;
    fight(app, dir, "10-fight-volcano", "garan", "marlow", "volcano", 500, "hit").await;
    fight(app, dir, "20-fx-ko", "daichi", "valdric", "dojo", 200, "ko").await;
    fight(app, dir, "23-impact-frame", "akio", "bruno", "spirit", 120, "impact").await;
    fight(app, dir, "24-paint-splash", "kaito", "garan", "plains", 240, "hit").await;
    fight(app, dir, "21-fx-neon-hit", "volta", "kaito", "neon", 200, "hit").await;
    team_fight(app, dir, "30-team-sequence", false, 3, "dojo", 200, false).await;
    team_fight(app, dir, "31-team-parallel", true, 2, "neon", 260, false).await;
    team_fight(app, dir, "33-team-5v5-parallel", true, 5, "plains", 330, true).await;
    team_fight(app, dir, "34-team-5v5-sequence", false, 5, "volcano", 200, true).await;
    // deciding round → "FINAL ROUND" announcer
    {
        app.mode = Mode::Watch;
        app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "bamboo"));
        app.team = None;
        app.start_fight().await;
        let App { session, c, audio, .. } = app;
        let s = session.as_mut().unwrap();
        s.m.wins = [1, 1]; s.m.round_no = 3; s.m.start_round();
        for _ in 0..22 { s.step_visual(c, audio); }
    }
    snap(app, dir, "41-final-round", |a| a.draw_session_only()).await;
    // replay determinism, end to end through the app: fight N ticks, replay the recording, compare the final game state
    {
        fn hash(s: &crate::session::Session) -> u64 { let mut h = 1469598103934665603u64; for f in &s.m.fighters { for v in [f.life.to_bits() as u64, f.power.to_bits() as u64, f.pos.x.to_bits() as u64, f.pos.y.to_bits() as u64, f.state_no as u64, f.state_time as u64] { h ^= v; h = h.wrapping_mul(1099511628211); } } h }
        app.mode = Mode::Watch;
        app.sel = (find(app, "akio"), find(app, "lian"), stage(app, "plains"));
        app.team = None; app.replaying = None;
        app.start_fight().await;
        let h1 = { let App { session, c, audio, .. } = app; let s = session.as_mut().unwrap(); for _ in 0..700 { s.step_visual(c, audio); } hash(s) };
        app.capture_replay();
        app.start_replay().await;
        let (h2, is_replay) = { let App { session, c, audio, .. } = app; let s = session.as_mut().unwrap(); for _ in 0..700 { s.step_visual(c, audio); } (hash(s), s.is_replay) };
        println!("replay-check: is_replay={is_replay} identical={}", h1 == h2);
        for _ in 0..4 { frames(app, 1).await; }
        snap(app, dir, "43-replay", |a| a.draw_session_only()).await;
        app.replaying = None;
    }
    // photosensitivity mode (impact effects = Mínimo): same kind of exchange with every flash / shake / VFX layer off
    app.g.fx_gain = 0.0;
    fight(app, dir, "42-photosensitive-mode", "volta", "kaito", "neon", 200, "hit").await;
    app.g.fx_gain = 1.0;
    // training mode: SF6-style input history + last-combo data (inputs injected, there is no keyboard in the tour)
    {
        use ume_engine::input::k;
        app.mode = Mode::Training;
        app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
        app.team = None;
        app.start_fight().await;
        let App { session, c, audio, .. } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..160 { s.step_visual(c, audio); }
        s.inputs = vec![(0, 20), (k::D, 3), (k::D | k::R, 3), (k::R, 4), (k::R | k::X, 2), (0, 12), (k::X, 2), (0, 8), (k::Y, 2), (k::D, 5), (k::D | k::A, 2), (0, 9), (k::R, 6), (k::R | k::W, 2)];
        for _ in 0..3 { s.step_visual(c, audio); }
        s.hud.inject_last_combo(7, 214.0);
    }
    snap(app, dir, "40-training-hud", |a| a.draw_session_only()).await;
    fight(app, dir, "11-hitboxes", "shion", "kaito", "plains", 200, "boxes").await;
    fight(app, dir, "11b-fight-kaname-valdric", "kaname", "valdric", "spirit", 300, "hit").await;
    fight(app, dir, "11c-fight-daichi-jin", "daichi", "jin", "plains", 300, "proj").await;
    fight(app, dir, "11d-super-haruki", "haruki", "daichi", "neon", 100, "super").await;
    if app.reg.chars.iter().any(|c| c.mugen) {
        let kfm = app.reg.chars.iter().find(|c| c.mugen).map(|c| c.id.clone()).unwrap();
        let ms = app.reg.stages.iter().find(|s| s.mugen && s.id.ends_with("kfm.def")).or(app.reg.stages.iter().find(|s| s.mugen)).map(|s| s.id.clone());
        fight(app, dir, "12-mugen-kfm-vs-native", &kfm, "suzume", "plains", 300, "hit").await;
        if let Some(ms) = ms { fight(app, dir, "13-native-on-mugen-stage", "marlow", &kfm, &ms, 300, "hit").await; }
        if let (Some(a), Some(b)) = (app.reg.chars.iter().find(|c| c.id.contains("true_z")).map(|c| c.id.clone()), app.reg.chars.iter().find(|c| c.id.contains("thunder_horse")).map(|c| c.id.clone())) {
            fight(app, dir, "13b-community-mugen", &a, &b, "rugen_night_festival", 300, "hit").await;
        }
        if let Some(a) = app.reg.chars.iter().find(|c| c.id.contains("restaze")).map(|c| c.id.clone()) {
            fight(app, dir, "13c-community-vs-original", &a, "jin", "rugen_ink_dojo", 300, "hit").await;
        }
    }
    if let Some(a) = app.reg.chars.iter().find(|c| c.id.contains("suave")).map(|c| c.id.clone()) {
        fight(app, dir, "13g-suave-dude", &a, "kaito", "stage1.def", 260, "hit").await;
    }
    for (n, st) in [("13d-stage-ikemen-z", "stagez.def"), ("13f-stage-ikemen-training", "stage1.def")] {
        if app.reg.stages.iter().any(|s| s.id.ends_with(st)) { fight(app, dir, n, "akio", "lian", st, 240, "hit").await; }
    }
    app.session = None;
    cap(app, Screen::Menu { cur: 3 }); frames(app, 30).await; app.trans = 0.62; screen(app, dir, "22-transition", 0).await;
    cap(app, Screen::Video { cur: 3 }); screen(app, dir, "17-video", 5).await;
    app.ui.begin(); app.ui.loading_screen(0.62, "Carregando lutador 2…"); get_screen_data().export_png(&format!("{dir}/18-loading.png")); println!("shot 18-loading"); next_frame().await;
    app.screen = Screen::Controls; screen(app, dir, "14-controls", 5).await;
    // phone landscape with touch controls
    request_new_screen_size(844.0, 390.0);
    frames(app, 10).await;
    app.c.touch.enabled = true;
    app.mode = Mode::Cpu;
    app.sel = (find(app, "suzume"), find(app, "kaito"), stage(app, "dojo"));
    app.start_fight().await;
    app.mode = Mode::Cpu;
    {
        let App { session, c, audio, .. } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..260 { s.step_visual(c, audio); }
    }
    snap(app, dir, "15-mobile-fight", |a| a.draw_session_only()).await;
    app.session = None;
    app.screen = Screen::Menu { cur: 0 }; screen(app, dir, "16-mobile-menu", 5).await;
    request_new_screen_size(1280.0, 720.0);
    frames(app, 10).await;
    story(app, dir).await;
}
