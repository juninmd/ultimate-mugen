use crate::app::App;
use crate::shots::fight;

const ATTACKER_SHOTS: [(&str, usize, &str); 3] = [
    ("idle", 120, "faceoff"),
    ("transform-start", 100, "transform"),
    ("transform-done", 100, "awake"),
];
const VICTIM_SHOTS: [(&str, usize, &str); 3] = [
    ("hit", 100, "hit"),
    ("knockout", 100, "ko"),
    ("defeat", 100, "defeat"),
];

pub async fn run(app: &mut App, dir: &str) {
    let chars: Vec<String> = std::env::var("UME_CHARS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if chars.len() < 2 {
        println!("set UME_CHARS=pixel_art_char,opponent");
        return;
    }
    let stage = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    let (hero, rival) = (&chars[0], &chars[1]);
    for (name, ticks, want) in ATTACKER_SHOTS {
        fight(
            app,
            dir,
            &format!("{hero}-{name}"),
            hero,
            rival,
            &stage,
            ticks,
            want,
        )
        .await;
    }
    if hero == "kaito" {
        fight(app, dir, "kaito-dragon", hero, rival, &stage, 100, "dragon").await;
        fight(app, dir, "kaito-rush", hero, rival, &stage, 100, "rush").await;
    }
    for (name, ticks, want) in VICTIM_SHOTS {
        fight(
            app,
            dir,
            &format!("{hero}-{name}"),
            rival,
            hero,
            &stage,
            ticks,
            want,
        )
        .await;
    }
}

pub(crate) fn probe(
    s: &mut crate::session::Session,
    c: &mut crate::controls::Controls,
    audio: &mut crate::audio::Audio,
    want: &str,
    name: &str,
) {
    for pilot in &mut s.pilots {
        *pilot = crate::session::Pilot::Dummy;
    }
    for _ in 0..600 {
        if s.m.round_state == 2 && s.m.fighters[0].ctrl {
            break;
        }
        s.step_visual(c, audio);
    }
    for _ in 0..90 {
        s.step_visual(c, audio);
    }
    let distance = if want == "rush" { 32.0 } else { 100.0 };
    s.m.fighters[0].pos.x = -distance;
    s.m.fighters[1].pos.x = distance;
    s.m.fighters[1].pos.y = 0.0;
    s.m.fighters[1].vel = Default::default();
    let d = s.m.fighters[0].def.clone();
    let move_name = if want == "rush" {
        "storm rush"
    } else {
        "storm dragon fist"
    };
    let state = d
        .states
        .get(&-1)
        .and_then(|state| {
            state
                .controllers
                .iter()
                .find(|c| c.label.to_ascii_lowercase().contains(move_name))
        })
        .and_then(|c| c.raw("value"))
        .and_then(|v| v.parse::<i32>().ok());
    use ume_engine::input::k;
    let direction = if want == "rush" { k::L } else { k::R };
    let button = if want == "rush" { k::X } else { k::Y };
    s.pilots[0] = crate::session::Pilot::Replay(
        vec![k::D, k::D | direction, direction, direction | button],
        0,
    );
    let mut entered = false;
    for _ in 0..90 {
        s.step_visual(c, audio);
        entered |= Some(s.m.fighters[0].state_no) == state;
        if want == "dragon" && s.m.projectiles.iter().any(|p| !p.dying && p.time > 2)
            || want == "rush" && s.m.fighters[1].received_hits > 0
        {
            break;
        }
    }
    println!(
        "{name}: state = {state:?}, entered by input = {entered}, projectiles = {}",
        s.m.projectiles.len()
    );
}

pub(crate) fn finish_probe(app: &mut App, want: &str, name: &str) {
    if want == "dragon" || want == "rush" {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        let before = s.m.fighters[1].life;
        for _ in 0..90 {
            s.step_visual(c, audio);
        }
        println!(
            "{name}: subsequent damage = {}, total hits = {}",
            before - s.m.fighters[1].life,
            s.m.fighters[1].received_hits
        );
    }
}

pub(crate) fn coverage(app: &App, id: &str) -> Vec<(&'static str, i32)> {
    let mut coverage = vec![
        ("idle", 0),
        ("walk", 20),
        ("walk-back", 21),
        ("run", 100),
        ("crouch", 11),
        ("jump-up", 41),
        ("jump-down", 44),
        ("damage", 5000),
        ("transform", 196),
    ];
    if id == "kaito" {
        let d = app.session.as_ref().unwrap().m.fighters[0].def.clone();
        for (label, name) in [
            ("dragon", "storm dragon fist"),
            ("rush", "storm rush"),
            ("uppercut", "rising tempest"),
        ] {
            let animation = d
                .states
                .get(&-1)
                .and_then(|state| {
                    state
                        .controllers
                        .iter()
                        .find(|c| c.label.to_ascii_lowercase().contains(name))
                })
                .and_then(|c| c.raw("value"))
                .and_then(|v| v.parse::<i32>().ok())
                .and_then(|state| d.states.get(&state))
                .and_then(|state| state.param("anim"))
                .and_then(|v| v.raw.parse::<i32>().ok());
            if let Some(animation) = animation {
                coverage.push((label, animation));
            }
        }
    }
    coverage
}
