//! Automated screenshot tour (`--shots <dir>`): renders real frames of every screen and fights.
use crate::app::{App, Mode, Screen, Team};
use crate::gfx::*;
use crate::shots::replay::replay_determinism;
mod replay;
mod story_tour;
mod suzume_tour;

async fn frames(app: &mut App, n: usize) {
    for _ in 0..n {
        app.frame().await;
        next_frame().await;
    }
}
async fn screen(app: &mut App, dir: &str, name: &str, warm: usize) {
    frames(app, warm).await;
    app.trans = 0.0;
    app.frame().await;
    get_screen_data().export_png(&format!("{dir}/{name}.png"));
    println!("shot {name}");
    next_frame().await;
}
async fn snap(app: &mut App, dir: &str, name: &str, draw: impl Fn(&mut App)) {
    // draw twice (first frame warms textures), capture the second
    for i in 0..2 {
        draw(app);
        if i == 1 {
            get_screen_data().export_png(&format!("{dir}/{name}.png"));
            println!("shot {name}");
        }
        next_frame().await;
    }
}
pub(crate) fn find(app: &App, id: &str) -> usize {
    app.reg
        .chars
        .iter()
        .position(|c| c.id.ends_with(id))
        .unwrap_or(0)
}
pub(crate) fn stage(app: &App, id: &str) -> usize {
    app.reg
        .stages
        .iter()
        .position(|s| s.id.contains(id))
        .unwrap_or(0)
}

pub(crate) async fn fight(
    app: &mut App,
    dir: &str,
    name: &str,
    p1: &str,
    p2: &str,
    st: &str,
    ticks: usize,
    want: &str,
) {
    app.mode = Mode::Watch;
    app.sel = (find(app, p1), find(app, p2), stage(app, st));
    app.start_fight().await;
    let App {
        session, c, audio, ..
    } = app;
    let s = session.as_mut().unwrap();
    for _ in 0..ticks {
        s.step_visual(c, audio);
    }
    match want {
        "faceoff" => {}
        "transform" | "awake" => {
            for _ in 0..600 {
                if s.m.round_state == 2 && s.m.fighters[0].ctrl {
                    break;
                }
                s.step_visual(c, audio);
            }
            s.m.fighters[0].power = 3000.0;
            s.m.fighters[0].life = s.m.fighters[0].life_max * 0.4;
            s.m.try_transform(0);
            for _ in 0..(if want == "awake" { 140 } else { 34 }) {
                s.step_visual(c, audio);
            }
        }
        "defeat" | "defeatfinal" => {
            for _ in 0..600 {
                if s.m.round_state == 2 {
                    break;
                }
                s.step_visual(c, audio);
            }
            if want == "defeatfinal" {
                s.m.wins = [1, 0];
            }
            for f in s.m.fighters.iter_mut() {
                f.pos.x = if f.side == 0 { -40.0 } else { 40.0 };
            }
            s.m.fighters[1].life = 0.0;
            for _ in 0..1500 {
                s.step_visual(c, audio);
                if (5190..=5193).contains(&s.m.fighters[1].state_no)
                    && s.m.fighters[1].state_time >= if want == "defeatfinal" { 96 } else { 78 }
                {
                    break;
                }
            }
        }
        "dragon" | "rush" => crate::pixel_tour::probe(s, c, audio, want, name),
        "super" | "super2" | "impact" => {
            for f in s.m.fighters.iter_mut() {
                f.power = 3000.0;
            }
            for _ in 0..600 {
                if s.m.fighters[0].ctrl && s.m.fighters[0].state_no == 0 {
                    break;
                }
                s.step_visual(c, audio);
            }
            let d = s.m.fighters[0].def.clone();
            let no = if want == "super2" { 3001 } else { 3000 };
            if d.states.contains_key(&no) {
                s.m.change_state(0, no, d);
            }
            let mut ok = false;
            for _ in 0..1500 {
                s.step_visual(c, audio);
                if s.m
                    .pause
                    .as_ref()
                    .is_some_and(|p| p.is_super && p.time < 24)
                {
                    ok = true;
                    break;
                }
            }
            if ok {
                for _ in 0..16 {
                    s.step_visual(c, audio);
                }
            }
            println!("{name}: super caught = {ok}");
            if want == "impact" {
                s.vfx.force_impact();
            }
        }
        "hit" => {
            for _ in 0..1200 {
                s.step_visual(c, audio);
                if !s.m.explods.is_empty()
                    && s.m
                        .fighters
                        .iter()
                        .any(|f| f.move_type == 'H' && f.hit_shake > 2 && !f.ghv.guarded)
                {
                    break;
                }
            }
        }
        "counter" | "guard" | "heavy" => {
            for _ in 0..600 {
                if s.m.round_state == 2 && s.m.fighters[0].ctrl {
                    break;
                }
                s.step_visual(c, audio);
            }
            if want == "counter" {
                s.m.fighters[1].move_type = 'A';
            }
            s.vfx.pre_tick(&s.m);
            let ev = ume_engine::matchs::Event::Hit {
                attacker_side: 0,
                defender_side: 1,
                damage: if want == "heavy" { 120.0 } else { 60.0 },
                guarded: want == "guard",
                heavy: want == "heavy",
                attacker: 0,
                defender: 1,
            };
            s.vfx.event(&ev, &s.m);
            for _ in 0..3 {
                s.vfx.update(1.0 / 60.0);
            }
        }
        "proj" => {
            for _ in 0..1500 {
                s.step_visual(c, audio);
                if s.m.projectiles.iter().any(|p| !p.dying && p.time > 8) {
                    break;
                }
            }
        }
        "ko" => {
            if let Some(f) = s.m.fighters.get_mut(1) {
                f.life = 1.0;
            }
            for _ in 0..3000 {
                s.step_visual(c, audio);
                if s.vfx.ko_active() {
                    break;
                }
            }
            for _ in 0..14 {
                s.step_visual(c, audio);
            }
        }
        "boxes" => {
            s.r.debug = true;
            for _ in 0..900 {
                s.step_visual(c, audio);
                if s.m.fighters.iter().any(|f| !f.boxes(1).is_empty()) {
                    break;
                }
            }
        }
        _ => {}
    }
    snap(app, dir, name, |a| a.draw_session_only()).await;
    crate::pixel_tour::finish_probe(app, want, name);
}

/// Team battle shot: `simul` = 2v2 on the field together, otherwise sequence (3v3, second bout with carried life).
async fn team_fight(
    app: &mut App,
    dir: &str,
    name: &str,
    simul: bool,
    n: usize,
    stage_id: &str,
    ticks: usize,
    human: bool,
) {
    let (a_ids, b_ids) = (
        ["kaito", "akio", "lian", "daichi", "jin"],
        ["suzume", "bruno", "sylene", "garan", "marlow"],
    );
    let a: Vec<usize> = a_ids.iter().take(n).map(|i| find(app, i)).collect();
    let b: Vec<usize> = b_ids.iter().take(n).map(|i| find(app, i)).collect();
    let mut t = Team::new(a.clone(), b.clone(), simul);
    let mut foe = 0;
    if !simul {
        t.ko[1][0] = true;
        t.idx[1] = 1;
        t.life = [0.62, 1.0];
        foe = 1;
    }
    app.mode = if human {
        if simul {
            Mode::TeamSimul
        } else {
            Mode::TeamSeq
        }
    } else {
        Mode::Watch
    };
    app.sel = (a[0], b[foe], stage(app, stage_id));
    app.team = Some(t);
    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..ticks {
            s.step_visual(c, audio);
        }
        for _ in 0..1500 {
            s.step_visual(c, audio);
            if s.m
                .fighters
                .iter()
                .any(|f| f.move_type == 'H' && f.hit_shake > 2)
            {
                break;
            }
        }
    }
    snap(app, dir, name, |a| a.draw_session_only()).await;
    app.team = None;
}

async fn duel(app: &mut App, dir: &str, name: &str, mode: Mode, ticks: usize) {
    app.mode = mode;
    app.team = None;
    app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "dojo"));
    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..ticks {
            s.step_visual(c, audio);
        }
        s.m.fighters[1].life *= 0.55;
        for f in s.m.fighters.iter_mut() {
            f.power = 1400.0;
        }
    }
    snap(app, dir, name, |a| a.draw_session_only()).await;
}

async fn team_win(app: &mut App, dir: &str, name: &str, simul: bool, n: usize) {
    let (a_ids, b_ids) = (
        ["kaito", "akio", "lian", "daichi", "jin"],
        ["suzume", "bruno", "sylene", "garan", "marlow"],
    );
    let a: Vec<usize> = a_ids.iter().take(n).map(|i| find(app, i)).collect();
    let b: Vec<usize> = b_ids.iter().take(n).map(|i| find(app, i)).collect();
    let mut t = Team::new(a.clone(), b.clone(), simul);
    let foe = if simul {
        0
    } else {
        t.ko[1][0] = true;
        t.idx[1] = 1;
        1
    };
    let cur = if simul {
        0
    } else {
        t.idx[0] = 2;
        t.ko[0][0] = true;
        t.ko[0][1] = true;
        2
    };
    app.mode = if simul {
        Mode::TeamSimul
    } else {
        Mode::TeamSeq
    };
    app.sel = (a[cur], b[foe], stage(app, "plains"));
    app.team = Some(t);
    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..600 {
            if s.m.round_state == 2 {
                break;
            }
            s.step_visual(c, audio);
        }
        for f in
            s.m.fighters
                .iter_mut()
                .filter(|f| f.side == 1 && !f.is_helper)
        {
            f.life = 0.0;
        }
        for _ in 0..1500 {
            s.step_visual(c, audio);
            if s.m.round_state == 4 && s.m.round_time > 30 {
                break;
            }
        }
    }
    snap(app, dir, name, |a| a.draw_session_only()).await;
    app.team = None;
}

async fn fx_tour(app: &mut App, dir: &str) {
    for (name, want) in [
        ("fx1-hit", "hit"),
        ("fx2-heavy", "heavy"),
        ("fx3-counter", "counter"),
        ("fx4-guard", "guard"),
        ("fx5-super", "super"),
    ] {
        fight(app, dir, name, "kaito", "shion", "dojo", 330, want).await;
    }
}

async fn hud_tour(app: &mut App, dir: &str) {
    duel(app, dir, "h1-p1-vs-cpu", Mode::Cpu, 330).await;
    duel(app, dir, "h2-p1-vs-p2", Mode::Versus, 330).await;
    team_fight(app, dir, "h3-3v3-parallel", true, 3, "neon", 330, true).await;
    team_fight(app, dir, "h4-5v5-parallel", true, 5, "plains", 330, true).await;
    team_fight(app, dir, "h5-5v5-sequence", false, 5, "volcano", 200, true).await;
    team_win(app, dir, "h6-team-win-parallel", true, 5).await;
    team_win(app, dir, "h7-team-win-sequence", false, 5).await;
}

/// Story / cinematic features: face-off, awakening, defeats, boss, cutscenes, zoomed-out 5v5.
async fn story(app: &mut App, dir: &str) {
    // "CHEFE FINAL" banner right when the boss fight starts (the face-off ends ~270 ticks in)
    if std::env::var("UME_STORY_SHOT").as_deref() == Ok("banner") {
        fight(
            app,
            dir,
            "68-boss-banner",
            "kaito",
            "oboro",
            "volcano",
            280,
            "faceoff",
        )
        .await;
        return;
    }
    fight(
        app,
        dir,
        "50-faceoff-1",
        "kaito",
        "shion",
        "dojo",
        82,
        "faceoff",
    )
    .await;
    fight(
        app,
        dir,
        "51-faceoff-2",
        "kaito",
        "shion",
        "dojo",
        195,
        "faceoff",
    )
    .await;
    fight(
        app,
        dir,
        "52-transform-start",
        "valdric",
        "kaito",
        "spirit",
        100,
        "transform",
    )
    .await;
    fight(
        app,
        dir,
        "53-transformed",
        "valdric",
        "kaito",
        "spirit",
        100,
        "awake",
    )
    .await;
    fight(
        app,
        dir,
        "54-defeat-round",
        "kaito",
        "shion",
        "dojo",
        100,
        "defeat",
    )
    .await;
    fight(
        app,
        dir,
        "55-defeat-final",
        "kaito",
        "shion",
        "bamboo",
        100,
        "defeatfinal",
    )
    .await;
    fight(
        app,
        dir,
        "56-boss-susanoo",
        "oboro",
        "kaito",
        "volcano",
        120,
        "super",
    )
    .await;
    fight(
        app,
        dir,
        "57-boss-kyoka",
        "oboro",
        "kaito",
        "spirit",
        120,
        "super2",
    )
    .await;
    fight(
        app,
        dir,
        "58-boss-awakened",
        "oboro",
        "kaito",
        "volcano",
        100,
        "awake",
    )
    .await;
    fight(
        app,
        dir,
        "59-boss-fight",
        "kaito",
        "oboro",
        "spirit",
        400,
        "hit",
    )
    .await;
    team_fight(app, dir, "60-5v5-zoomout", true, 5, "dojo", 330, true).await;
    team_fight(app, dir, "61-5v5-zoomout-neon", true, 5, "neon", 420, true).await;
    // the real arcade path to the final boss: last win on the ladder → pre-boss cutscene → VS → face-off → fight against the CPU boss
    {
        use crate::session::Pilot;
        use ume_engine::ai::Ai;
        let boss = find(app, "oboro");
        let mut ladder: Vec<usize> = (0..app.reg.chars.len())
            .filter(|&i| !app.reg.chars[i].boss && !app.reg.chars[i].mugen)
            .take(8)
            .collect();
        let others = ladder.len();
        ladder.push(boss);
        app.mode = Mode::Arcade;
        app.team = None;
        app.sel = (
            find(app, "kaito"),
            ladder[others - 1],
            stage(app, "volcano"),
        );
        app.arcade = Some((ladder, others - 1));
        app.ending_shown = false;
        app.screen = Screen::Results { winner: 0, cur: 0 };
        frames(app, 3).await;
        println!(
            "arcade → screen is cutscene: {}",
            matches!(app.screen, Screen::Cutscene { kind: 1, .. })
        );
        if let Screen::Cutscene { .. } = app.screen {
            app.screen = Screen::Cutscene {
                kind: 1,
                page: 1,
                t: 0.0,
            };
        }
        screen(app, dir, "65-arcade-preboss-cutscene", 80).await;
        app.screen = Screen::Vs { t: 0.0, step: 0 };
        app.sel.2 = stage(app, "volcano");
        app.start_fight().await;
        {
            let App { session, .. } = app;
            let s = session.as_mut().unwrap();
            println!(
                "boss fight: boss={} level={}",
                s.m.roots().any(|f| f.def.boss),
                match &s.pilots[1] {
                    Pilot::Cpu(a) => a.level,
                    _ => -1,
                }
            );
            s.pilots[0] = Pilot::Cpu(Ai::new(6, 5));
            drop(s);
        }
        let (mut seen, mut awakened, mut shot_super, mut shot_awake) =
            (std::collections::BTreeSet::new(), false, false, false);
        for t in 0..2400 {
            let (st, form, ft, sup, rs) = {
                let App {
                    session, c, audio, ..
                } = app;
                let s = session.as_mut().unwrap();
                s.step_visual(c, audio);
                let f = s.m.fighters.iter().find(|f| f.def.boss).unwrap();
                (
                    f.state_no,
                    f.form,
                    f.form_time,
                    s.m.pause
                        .as_ref()
                        .is_some_and(|p| p.is_super && p.time < 30),
                    s.m.round_state,
                )
            };
            seen.insert(st);
            if form > 0 {
                awakened = true;
            }
            if st >= 3000 && !shot_super && sup {
                shot_super = true;
                snap(app, dir, "66-arcade-boss-super", |a| a.draw_session_only()).await;
            }
            if form > 0 && !shot_awake && ft > 90 {
                shot_awake = true;
                snap(app, dir, "67-arcade-boss-awakened", |a| {
                    a.draw_session_only()
                })
                .await;
            }
            if rs >= 3 && t > 300 {
                break;
            }
        }
        {
            let App { session, .. } = app;
            let s = session.as_mut().unwrap();
            println!("boss fight: states used {:?}, awakened={awakened}, round_state={}, boss life {:.0}/{:.0}", seen.iter().filter(|n| **n >= 1000).collect::<Vec<_>>(), s.m.round_state, s.m.fighters.iter().find(|f| f.def.boss).map(|f| f.life).unwrap_or(0.0), s.m.fighters.iter().find(|f| f.def.boss).map(|f| f.life_max).unwrap_or(0.0));
        }
        app.session = None;
    }
    app.sel = (find(app, "kaito"), find(app, "oboro"), 0);
    app.arcade = Some((vec![find(app, "oboro")], 0));
    for (kind, page, name) in [
        (0u8, 1usize, "62-cutscene-opening"),
        (1, 3, "63-cutscene-boss"),
        (2, 1, "64-cutscene-ending"),
    ] {
        app.screen = Screen::Cutscene { kind, page, t: 0.0 };
        screen(app, dir, name, 70).await;
    }
}

async fn native_coverage(app: &mut App, dir: &str) {
    let selected: Vec<String> = std::env::var("UME_CHARS")
        .unwrap_or_default()
        .split(',')
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect();
    let fighters: Vec<String> = app
        .reg
        .chars
        .iter()
        .filter(|fighter| !fighter.mugen)
        .map(|fighter| fighter.id.trim_start_matches("umf:").to_string())
        .filter(|id| selected.is_empty() || selected.contains(id))
        .collect();
    for id in fighters {
        app.mode = Mode::Watch;
        app.team = None;
        app.sel = (find(app, &id), find(app, "kaito"), stage(app, "brazil"));
        app.start_fight().await;
        {
            let session = app.session.as_mut().unwrap();
            session.m.round_state = 2;
            session.m.announce = None;
            session.m.round_time = 300;
            session.m.intro_ticks = 0;
            session.m.camera.x = 0.0;
            session.m.camera.y = 0.0;
            for (index, fighter) in session.m.fighters.iter_mut().enumerate() {
                fighter.pos.x = if index == 0 { -70.0 } else { 70.0 };
                fighter.pos.y = 0.0;
                fighter.vel.x = 0.0;
                fighter.vel.y = 0.0;
                fighter.change_anim(0, &fighter.def.clone(), 1);
                if index == 0 && std::env::var("UME_AWAKENED").as_deref() == Ok("1") {
                    fighter.form = 1;
                }
            }
        }
        snap(app, dir, &format!("height-{id}"), |app| {
            app.draw_session_only()
        })
        .await;
        for (label, animation) in crate::pixel_tour::coverage(app, &id) {
            let definition = app.session.as_ref().unwrap().m.fighters[0].def.clone();
            let anim = definition.anims.get(&animation).unwrap_or_else(|| {
                panic!("{id}: required {label} animation {animation} is missing")
            });
            let count = anim.frames.len();
            for frame in 0..count {
                {
                    let fighter = &mut app.session.as_mut().unwrap().m.fighters[0];
                    fighter.change_anim(animation, &definition, 1);
                    fighter.elem = frame;
                    fighter.elem_time = 0;
                }
                snap(app, dir, &format!("pose-{id}-{label}-{frame}"), |app| {
                    app.draw_session_only()
                })
                .await;
            }
        }
    }
}

pub async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    if std::env::var("UME_TOUR").as_deref() == Ok("suzume") {
        suzume_tour::suzume_abilities_tour(app, dir).await;
        return;
    }
    if std::env::var("UME_TOUR").as_deref() == Ok("native-coverage") {
        native_coverage(app, dir).await;
        return;
    }
    if matches!(
        std::env::var("UME_TOUR").as_deref(),
        Ok("mugen" | "moves" | "duel" | "clips")
    ) {
        crate::mugen_tour::run(app, dir).await;
        return;
    }
    if std::env::var("UME_TOUR").as_deref() == Ok("ui") {
        ui_tour(app, dir).await;
        return;
    }
    if std::env::var("UME_TOUR").as_deref() == Ok("story") {
        story(app, dir).await;
        return;
    }
    if std::env::var("UME_TOUR").as_deref() == Ok("story_plus") {
        story_tour::run(app, dir).await;
        return;
    }
    if std::env::var("UME_SHOTS_ONLY").as_deref() == Ok("stage-select") {
        app.screen = Screen::Stages { cur: 1 };
        screen(app, dir, "stage-select", 8).await;
        return;
    }
    if std::env::var("UME_SHOTS_ONLY").as_deref() == Ok("select") {
        app.mode = Mode::Cpu;
        let ids = ["kaname", "valdric", "akio", "kaito"];
        let cur = [find(app, ids[0]), find(app, ids[1])];
        app.screen = Screen::Select {
            cur,
            picking: 0,
            picked: [None, None],
        };
        screen(app, dir, "select-1p", 60).await;
        app.screen = Screen::Select {
            cur: [find(app, ids[2]), find(app, ids[1])],
            picking: 1,
            picked: [Some(find(app, ids[2])), None],
        };
        screen(app, dir, "select-2p", 60).await;
        app.mode = Mode::TeamSimul;
        app.team_n = 5;
        app.team_pick = vec![find(app, "kaito"), find(app, "lian"), find(app, ids[0])];
        app.screen = Screen::Select {
            cur: [find(app, ids[3]), 0],
            picking: 0,
            picked: [None, None],
        };
        screen(app, dir, "select-team5", 60).await;
        return;
    }
    if std::env::var("UME_SHOTS_ONLY").as_deref() == Ok("fx") {
        fx_tour(app, dir).await;
        return;
    }
    if std::env::var("UME_SHOTS_ONLY").as_deref() == Ok("menu") {
        app.screen = Screen::Title;
        screen(app, dir, "menu-title", 30).await;
        for (n, cur) in [(0, 0usize), (1, 1), (2, 3), (3, 7), (4, 9)] {
            app.screen = Screen::Menu { cur };
            screen(app, dir, &format!("menu-{n}"), 40).await;
        }
        return;
    }
    if std::env::var("UME_SHOTS_ONLY").as_deref() == Ok("hud") {
        hud_tour(app, dir).await;
        return;
    }
    if std::env::var("UME_SHOTS_ONLY").as_deref() == Ok("market") {
        market_tour(app, dir).await;
        return;
    }
    if std::env::var("UME_TOUR").as_deref() == Ok("pixel") {
        crate::pixel_tour::run(app, dir).await;
        return;
    }
    if std::env::var("UME_TOUR").as_deref() == Ok("perf") {
        crate::perf_tour::run(app, dir).await;
        return;
    }
    if std::env::var("UME_SHOTS_ONLY").as_deref() == Ok("stages") {
        app.screen = Screen::Fight;
        crate::stage_tour::run(app, dir).await;
        return;
    }
    app.screen = Screen::Title;
    screen(app, dir, "01-title", 30).await;
    let cap = |app: &mut App, s: Screen| {
        app.screen = s;
    };
    cap(app, Screen::Menu { cur: 1 });
    screen(app, dir, "02-menu", 45).await;
    app.mode = Mode::Cpu;
    cap(
        app,
        Screen::Select {
            cur: [find(app, "kaname"), find(app, "valdric")],
            picking: 1,
            picked: [Some(find(app, "kaname")), None],
        },
    );
    screen(app, dir, "03-select", 40).await;
    app.mode = Mode::TeamSeq;
    app.team_pick = vec![find(app, "kaito"), find(app, "lian")];
    cap(
        app,
        Screen::Select {
            cur: [find(app, "akio"), find(app, "valdric")],
            picking: 0,
            picked: [None, None],
        },
    );
    screen(app, dir, "32-team-select", 20).await;
    app.mode = Mode::Cpu;
    app.team_pick.clear();
    cap(app, Screen::Stages { cur: 2 });
    screen(app, dir, "04-stages", 40).await;
    app.sel = (find(app, "kaito"), find(app, "garan"), 0);
    cap(app, Screen::Vs { t: 0.0, step: 0 });
    screen(app, dir, "05a-vs-loading", 3).await;
    screen(app, dir, "05-vs", 70).await;
    app.screen = Screen::Fight;
    fight(
        app,
        dir,
        "06-fight-dojo",
        "kaito",
        "shion",
        "dojo",
        240,
        "hit",
    )
    .await;
    fight(
        app,
        dir,
        "07-fight-neon",
        "volta",
        "suzume",
        "neon",
        300,
        "proj",
    )
    .await;
    fight(
        app,
        dir,
        "08-fight-super",
        "akio",
        "bruno",
        "spirit",
        120,
        "super",
    )
    .await;
    fight(
        app,
        dir,
        "09-fight-bamboo",
        "lian",
        "sylene",
        "bamboo",
        400,
        "proj",
    )
    .await;
    fight(
        app,
        dir,
        "10-fight-volcano",
        "garan",
        "marlow",
        "volcano",
        500,
        "hit",
    )
    .await;
    fight(app, dir, "20-fx-ko", "daichi", "valdric", "dojo", 200, "ko").await;
    fight(
        app,
        dir,
        "23-impact-frame",
        "akio",
        "bruno",
        "spirit",
        120,
        "impact",
    )
    .await;
    fight(
        app,
        dir,
        "24-paint-splash",
        "kaito",
        "garan",
        "plains",
        240,
        "hit",
    )
    .await;
    fight(
        app,
        dir,
        "21-fx-neon-hit",
        "volta",
        "kaito",
        "neon",
        200,
        "hit",
    )
    .await;
    team_fight(app, dir, "30-team-sequence", false, 3, "dojo", 200, false).await;
    team_fight(app, dir, "31-team-parallel", true, 2, "neon", 260, false).await;
    team_fight(
        app,
        dir,
        "33-team-5v5-parallel",
        true,
        5,
        "plains",
        330,
        true,
    )
    .await;
    team_fight(
        app,
        dir,
        "34-team-5v5-sequence",
        false,
        5,
        "volcano",
        200,
        true,
    )
    .await;
    // deciding round → "FINAL ROUND" announcer
    {
        app.mode = Mode::Watch;
        app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "bamboo"));
        app.team = None;
        app.start_fight().await;
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        s.m.wins = [1, 1];
        s.m.round_no = 3;
        s.m.start_round();
        for _ in 0..22 {
            s.step_visual(c, audio);
        }
    }
    snap(app, dir, "41-final-round", |a| a.draw_session_only()).await;
    replay_determinism(app, dir).await;
    // photosensitivity mode (impact effects = Mínimo): same kind of exchange with every flash / shake / VFX layer off
    app.g.fx_gain = 0.0;
    fight(
        app,
        dir,
        "42-photosensitive-mode",
        "volta",
        "kaito",
        "neon",
        200,
        "hit",
    )
    .await;
    app.g.fx_gain = 1.0;
    // training mode: SF6-style input history + last-combo data (inputs injected, there is no keyboard in the tour)
    {
        use ume_engine::input::k;
        app.mode = Mode::Training;
        app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
        app.team = None;
        app.start_fight().await;
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..160 {
            s.step_visual(c, audio);
        }
        s.inputs = vec![
            (0, 20),
            (k::D, 3),
            (k::D | k::R, 3),
            (k::R, 4),
            (k::R | k::X, 2),
            (0, 12),
            (k::X, 2),
            (0, 8),
            (k::Y, 2),
            (k::D, 5),
            (k::D | k::A, 2),
            (0, 9),
            (k::R, 6),
            (k::R | k::W, 2),
        ];
        for _ in 0..3 {
            s.step_visual(c, audio);
        }
        s.hud.inject_last_combo(7, 214.0);
    }
    snap(app, dir, "40-training-hud", |a| a.draw_session_only()).await;
    fight(
        app,
        dir,
        "11-hitboxes",
        "shion",
        "kaito",
        "plains",
        200,
        "boxes",
    )
    .await;
    fight(
        app,
        dir,
        "11b-fight-kaname-valdric",
        "kaname",
        "valdric",
        "spirit",
        300,
        "hit",
    )
    .await;
    fight(
        app,
        dir,
        "11c-fight-daichi-jin",
        "daichi",
        "jin",
        "plains",
        300,
        "proj",
    )
    .await;
    fight(
        app,
        dir,
        "11d-super-haruki",
        "haruki",
        "daichi",
        "neon",
        100,
        "super",
    )
    .await;
    if app.reg.chars.iter().any(|c| c.mugen) {
        let kfm = app
            .reg
            .chars
            .iter()
            .find(|c| c.mugen && c.id.to_lowercase().contains("kfm"))
            .or_else(|| app.reg.chars.iter().find(|c| c.mugen))
            .map(|c| c.id.clone())
            .unwrap();
        let ms = app
            .reg
            .stages
            .iter()
            .find(|s| s.mugen && s.id.ends_with("kfm.def"))
            .or(app.reg.stages.iter().find(|s| s.mugen))
            .map(|s| s.id.clone());
        fight(
            app,
            dir,
            "12-mugen-kfm-vs-native",
            &kfm,
            "suzume",
            "plains",
            300,
            "hit",
        )
        .await;
        if let Some(ms) = ms {
            fight(
                app,
                dir,
                "13-native-on-mugen-stage",
                "marlow",
                &kfm,
                &ms,
                300,
                "hit",
            )
            .await;
        }
        if let (Some(a), Some(b)) = (
            app.reg
                .chars
                .iter()
                .find(|c| c.id.contains("true_z"))
                .map(|c| c.id.clone()),
            app.reg
                .chars
                .iter()
                .find(|c| c.id.contains("thunder_horse"))
                .map(|c| c.id.clone()),
        ) {
            fight(
                app,
                dir,
                "13b-community-mugen",
                &a,
                &b,
                "rugen_night_festival",
                300,
                "hit",
            )
            .await;
        }
        if let Some(a) = app
            .reg
            .chars
            .iter()
            .find(|c| c.id.contains("restaze"))
            .map(|c| c.id.clone())
        {
            fight(
                app,
                dir,
                "13c-community-vs-original",
                &a,
                "jin",
                "rugen_ink_dojo",
                300,
                "hit",
            )
            .await;
        }
    }
    if let Some(a) = app
        .reg
        .chars
        .iter()
        .find(|c| c.id.contains("suave"))
        .map(|c| c.id.clone())
    {
        fight(
            app,
            dir,
            "13g-suave-dude",
            &a,
            "kaito",
            "stage1.def",
            260,
            "hit",
        )
        .await;
    }
    for (n, st) in [
        ("13d-stage-ikemen-z", "stagez.def"),
        ("13f-stage-ikemen-training", "stage1.def"),
    ] {
        if app.reg.stages.iter().any(|s| s.id.ends_with(st)) {
            fight(app, dir, n, "akio", "lian", st, 240, "hit").await;
        }
    }
    app.session = None;
    cap(app, Screen::Menu { cur: 3 });
    frames(app, 30).await;
    app.trans = 0.62;
    screen(app, dir, "22-transition", 0).await;
    cap(
        app,
        Screen::Settings {
            tab: 0,
            cur: 3,
            from_fight: false,
        },
    );
    screen(app, dir, "17-video", 40).await;
    app.ui.begin();
    app.ui.loading_screen(0.62, "Carregando lutador 2…");
    get_screen_data().export_png(&format!("{dir}/18-loading.png"));
    println!("shot 18-loading");
    next_frame().await;
    app.screen = Screen::Controls;
    screen(app, dir, "14-controls", 5).await;
    // phone landscape with touch controls
    request_new_screen_size(844.0, 390.0);
    frames(app, 10).await;
    app.c.touch.enabled = true;
    app.mode = Mode::Cpu;
    app.sel = (find(app, "suzume"), find(app, "kaito"), stage(app, "dojo"));
    app.start_fight().await;
    app.mode = Mode::Cpu;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..260 {
            s.step_visual(c, audio);
        }
    }
    snap(app, dir, "15-mobile-fight", |a| a.draw_session_only()).await;
    app.session = None;
    app.screen = Screen::Menu { cur: 0 };
    screen(app, dir, "16-mobile-menu", 45).await;
    request_new_screen_size(1280.0, 720.0);
    frames(app, 10).await;
    story(app, dir).await;
}

async fn market_tour(app: &mut App, dir: &str) {
    use std::sync::Arc;
    use ume_market::catalog::{detect, Tree, TreeEntry};
    use ume_market::persist::{catalog_key, MemPersist, Persist, STATE_KEY};
    use ume_market::{Installed, MarketState, RepoRef};
    let files: Vec<(String, u64)> = [
        ("chars/kfm/kfm.def", 1),
        ("chars/kfm/kfm.sff", 900_000),
        ("chars/kfm/kfm.air", 4000),
        ("chars/kfm/kfm.cns", 9000),
        ("chars/kfm/kfm.cmd", 3000),
        ("chars/goku/goku.def", 1),
        ("chars/goku/goku.sff", 21_000_000),
        ("chars/goku/goku.air", 4000),
        ("chars/goku/goku.cns", 9000),
        ("stages/dojo.def", 400),
        ("stages/dojo.sff", 3_400_000),
        ("stages/city.def", 400),
        ("stages/city.sff", 5_100_000),
        ("music/theme.ogg", 4_200_000),
        ("music/battle.mp3", 6_800_000),
        ("packs/retro.zip", 18_000_000),
        ("packs/huge.zip", 90_000_000),
    ]
    .iter()
    .map(|(p, s)| (p.to_string(), *s))
    .collect();
    let tree = Tree {
        entries: files
            .into_iter()
            .map(|(path, size)| TreeEntry { path, size })
            .collect(),
        truncated: false,
    };
    let repo = RepoRef::parse("https://github.com/exemplo/mugen-pack").unwrap();
    let catalog = detect(&tree, "", &repo.name);
    let persist = Arc::new(MemPersist::new());
    let mut state = MarketState::default();
    state.add_repo(repo.clone());
    state.add_repo(RepoRef::parse("juninmd/ultimate-mugen").unwrap());
    state.repo_mut(&repo.key()).unwrap().item_count = catalog.items.len();
    state.repo_mut(&repo.key()).unwrap().listed_at = ume_market::now_secs();
    for it in catalog
        .items
        .iter()
        .filter(|i| i.name == "kfm" || i.name == "dojo")
    {
        state.upsert_installed(Installed {
            repo_key: repo.key(),
            item_id: it.id.clone(),
            kind: it.kind,
            name: it.name.clone(),
            bytes: it.bytes,
            files: it.files.len(),
            active: it.name == "kfm",
            installed_at: 1,
        });
    }
    persist
        .write(
            &catalog_key(&repo.key()),
            &serde_json::to_vec(&catalog).unwrap(),
        )
        .unwrap();
    persist.write(STATE_KEY, &state.to_json()).unwrap();
    app.market = crate::market::Market::with_persist(persist);
    app.market.finish_hydration();
    app.market.poll();
    app.screen = Screen::Market;
    screen(app, dir, "market-browse", 12).await;
    for (i, it) in catalog.items.iter().enumerate() {
        if i % 2 == 0 {
            app.market_ui_mark(&repo.key(), &it.id);
        }
    }
    screen(app, dir, "market-marked", 3).await;
    app.market_ui_view_installed();
    screen(app, dir, "market-installed", 3).await;
}
async fn ui_tour(app: &mut App, dir: &str) {
    for tab in 0..6 {
        app.screen = Screen::Settings {
            tab,
            cur: if tab == 1 { 2 } else { 1 },
            from_fight: false,
        };
        screen(app, dir, &format!("settings-{tab}"), 6).await;
    }
    app.settings.lang = crate::i18n::Lang::En;
    app.screen = Screen::Settings {
        tab: 3,
        cur: 2,
        from_fight: false,
    };
    screen(app, dir, "settings-3-en", 6).await;
    app.settings.lang = crate::i18n::Lang::Pt;
    app.sel = (find(app, "kaito"), find(app, "garan"), stage(app, "dojo"));
    app.mode = Mode::Cpu;
    app.screen = Screen::Vs { t: 0.0, step: 0 };
    frames(app, 40).await;
    app.screen = Screen::Vs { t: 0.55, step: 0 };
    screen(app, dir, "vs-1v1-mid", 0).await;
    app.screen = Screen::Vs { t: 1.4, step: 0 };
    screen(app, dir, "vs-1v1", 0).await;
    let roster: Vec<usize> = (0..app.reg.chars.len())
        .filter(|&i| !app.reg.chars[i].boss && !app.reg.chars[i].broken.get())
        .take(10)
        .collect();
    if roster.len() >= 10 {
        app.mode = Mode::TeamSimul;
        app.team_n = 5;
        app.sel = (roster[0], roster[5], stage(app, "neon"));
        app.team = Some(Team::new(
            roster[..5].to_vec(),
            roster[5..10].to_vec(),
            true,
        ));
        app.screen = Screen::Vs { t: 0.0, step: 0 };
        frames(app, 40).await;
        println!(
            "5v5 after load: vs={}",
            matches!(app.screen, Screen::Vs { .. })
        );
        app.screen = Screen::Vs { t: -9.0, step: 200 };
        frames(app, 3).await;
        app.trans = 0.0;
        app.screen = Screen::Vs { t: 1.5, step: 200 };
        screen(app, dir, "vs-5v5", 0).await;
        println!(
            "5v5 screen vs={} msg={}",
            matches!(app.screen, Screen::Vs { .. }),
            app.msg
        );
    }
    app.sel = (find(app, "suzume"), find(app, "shion"), stage(app, "dojo"));
    app.start_fight().await;
    if let Some(s) = app.session.as_mut() {
        s.paused = true;
    }
    screen(app, dir, "suzume-pause-movelist", 2).await;
}
