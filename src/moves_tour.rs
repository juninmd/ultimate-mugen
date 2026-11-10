//! Move showcase for the screenshot tour: `UME_TOUR=moves UME_CHARS=akio,bruno ultimate-mugen --shots <dir> --res 1280x720 --mute`
//! forces every special, super and throw of each listed fighter against a dummy rival, prints the damage it dealt and saves the
//! frame a few ticks after the first hit (`<dir>/M-<fighter>-<move>.png`).
//! `UME_BOXES=1` draws the hit and hurt boxes on the captured frames; `UME_AFTER_HIT=<ticks>` changes the capture delay.
pub mod clips;
pub mod duel;
use crate::app::{App, Mode};
use crate::gfx::*;
use crate::shots::{find, stage};
use std::sync::Arc;
use ume_engine::chardef::CharDef;

const MAX_WAIT_TICKS: usize = 150;
const TICKS_AFTER_HIT: usize = 5;

struct Showcase {
    name: String,
    state: i32,
    gap: f32,
    airborne: bool,
}

fn showcases(def: &CharDef) -> Vec<Showcase> {
    let Some(minus_one) = def.states.get(&-1) else { return vec![] };
    let mut out = vec![];
    for controller in &minus_one.controllers {
        let Some(name) = controller.label.strip_prefix("State -1, ") else { continue };
        let Some(state) = controller.raw("value").and_then(|v| v.trim().parse::<i32>().ok()) else { continue };
        let Some(body) = def.states.get(&state) else { continue };
        let is_throw = body.controllers.iter().any(|c| c.label.ends_with("throw_grab"));
        let is_special = (1000..3800).contains(&state);
        if !is_throw && !is_special || out.iter().any(|s: &Showcase| s.state == state) {
            continue;
        }
        let gap = if is_throw { 34.0 } else if state >= 3000 { 48.0 } else { 52.0 };
        let airborne = body.param("type").is_some_and(|t| t.raw.eq_ignore_ascii_case("A"));
        let gap = std::env::var("UME_GAP").ok().and_then(|g| g.parse().ok()).unwrap_or(gap);
        out.push(Showcase { name: name.to_string(), state, gap, airborne });
    }
    out.sort_by_key(|s| s.state);
    out
}

fn slug(name: &str) -> String {
    let plain = |c: char| match c { 'á' | 'à' | 'â' | 'ã' => 'a', 'é' | 'ê' => 'e', 'í' => 'i', 'ó' | 'ô' | 'õ' => 'o', 'ú' | 'ü' => 'u', 'ç' => 'c', other => other };
    name.to_lowercase().chars().map(plain).map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect::<String>().trim_matches('-').to_string()
}

pub async fn run(app: &mut App, dir: &str) {
    let chars: Vec<String> = std::env::var("UME_CHARS").unwrap_or_default().split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    let rival = std::env::var("UME_RIVAL").unwrap_or_else(|_| "kaito".into());
    let arena = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    for hero in chars {
        app.mode = Mode::Watch;
        app.sel = (find(app, &hero), find(app, &rival), stage(app, &arena));
        app.start_fight().await;
        {
            let App { session, c, audio, .. } = app;
            let s = session.as_mut().unwrap();
            for pilot in &mut s.pilots {
                *pilot = crate::session::Pilot::Dummy(Default::default());
            }
            s.r.debug = std::env::var("UME_BOXES").is_ok();
            for _ in 0..600 {
                if s.m.round_state == 2 && s.m.fighters[0].ctrl {
                    break;
                }
                s.step_visual(c, audio);
            }
        }
        let def: Arc<CharDef> = app.session.as_ref().unwrap().m.fighters[0].def.clone();
        for show in showcases(&def) {
            let center = app.session.as_ref().unwrap().m.camera.x;
            {
                let m = &mut app.session.as_mut().unwrap().m;
                m.fighters[0].pos.x = center - show.gap / 2.0;
                m.fighters[1].pos.x = center + show.gap / 2.0;
                m.fighters[0].pos.y = if show.airborne { -90.0 } else { 0.0 };
                m.fighters[1].pos.y = 0.0;
                m.fighters[0].facing = 1.0;
                m.fighters[1].facing = -1.0;
                m.fighters[0].power = 3000.0;
                m.fighters[1].life = m.fighters[1].life_max;
                let rival_def = m.fighters[1].def.clone();
                m.change_state(1, 0, rival_def);
                m.change_state(0, show.state, def.clone());
            }
            let mut after_hit = None;
            let (mut start, mut peak) = (0.0, 0.0);
            for tick in 0..MAX_WAIT_TICKS {
                {
                    let App { session, c, audio, .. } = app;
                    let s = session.as_mut().unwrap();
                    if tick == 0 {
                        start = s.m.fighters[1].life;
                    }
                    s.step_visual(c, audio);
                    peak = f32::max(peak, start - s.m.fighters[1].life);
                }
                if after_hit.is_none() && peak > 0.0 {
                    after_hit = Some(tick + std::env::var("UME_AFTER_HIT").ok().and_then(|t| t.parse().ok()).unwrap_or(TICKS_AFTER_HIT));
                }
                if after_hit == Some(tick) {
                    app.draw_session_only();
                    let path = format!("{dir}/M-{hero}-{}.png", slug(&show.name));
                    get_screen_data().export_png(&path);
                    next_frame().await;
                }
                if after_hit.is_some_and(|t| tick >= t + 70) {
                    break;
                }
            }
            println!("{hero} | {} | state {} | damage {:.0}{}", show.name, show.state, peak, if after_hit.is_none() { " | NO HIT" } else { "" });
        }
    }
}
