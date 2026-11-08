//! Ability reels for the screenshot tour: `UME_TOUR=clips UME_CHARS=akio,jin ultimate-mugen --shots <dir> --res 1280x720 --mute`
//! records a short frame sequence of every normal, special, super, throw, movement ability, the energy charge (hold `R`) and the
//! transformation of each fighter against a dummy rival, as `<dir>/C-<fighter>-<nn>-<move>-<frame>.png`, and prints the damage dealt.
//! `UME_CLIP_STRIDE` sets the ticks between frames, `UME_BOXES=1` draws the hit and hurt boxes.
use super::{showcases, slug};
use crate::app::{App, Mode};
use crate::gfx::*;
use crate::shots::{find, stage};
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::input::k;

const NORMAL_STATES: std::ops::Range<i32> = 200..700;
const MOVEMENT_STATES: [(i32, &str, f32); 2] = [(3900, "fly", 120.0), (3910, "teleport", 150.0)];
const NORMAL_FRAMES: usize = 7;
const MOVE_FRAMES: usize = 10;
const CHARGE_FRAMES: usize = 12;
const TRANSFORM_FRAMES: usize = 12;
const CHARGE_STRIDE_FACTOR: usize = 2;
const WARM_UP_TICKS: usize = 600;
const AIR_STATE_OFFSET: i32 = 20000;
const AIR_START_HEIGHT: f32 = -22.0;
const AIR_FRAMES: usize = 9;
const SETTLE_TICKS: usize = 8;
const SUPER_STATE: i32 = 3000;
const SUPER_FRAMES: usize = 18;

struct Clip {
    name: String,
    state: Option<i32>,
    gap: f32,
    height: f32,
    frames: usize,
    stride_factor: usize,
    hold: u32,
    transform: bool,
}

fn plain(name: String, state: i32, gap: f32, airborne: bool, frames: usize) -> Clip {
    Clip {
        name,
        state: Some(state),
        gap,
        height: if airborne { -90.0 } else { 0.0 },
        frames,
        stride_factor: 1,
        hold: 0,
        transform: false,
    }
}

fn clips(def: &CharDef) -> Vec<Clip> {
    let mut out = vec![];
    let mut normals: Vec<i32> = def
        .states
        .keys()
        .copied()
        .filter(|state| NORMAL_STATES.contains(state))
        .collect();
    normals.sort();
    for state in normals {
        let airborne = def.states[&state]
            .param("type")
            .is_some_and(|t| t.raw.eq_ignore_ascii_case("A"));
        out.push(plain(
            format!("normal {state}"),
            state,
            42.0,
            airborne,
            NORMAL_FRAMES,
        ));
    }
    for show in showcases(def) {
        let mut clip = plain(show.name, show.state, show.gap, show.airborne, MOVE_FRAMES);
        if show.state >= SUPER_STATE {
            clip.frames = SUPER_FRAMES;
            clip.stride_factor = 2;
        }
        out.push(clip);
    }
    for (state, name, gap) in MOVEMENT_STATES {
        if def.states.contains_key(&state) {
            out.push(plain(name.to_string(), state, gap, false, MOVE_FRAMES));
        }
    }
    out.push(Clip {
        name: "charge energy".into(),
        state: None,
        gap: 140.0,
        height: 0.0,
        frames: CHARGE_FRAMES,
        stride_factor: CHARGE_STRIDE_FACTOR,
        hold: k::S,
        transform: false,
    });
    if !def.forms.is_empty() {
        out.push(Clip {
            name: "transformation".into(),
            state: None,
            gap: 140.0,
            height: 0.0,
            frames: TRANSFORM_FRAMES,
            stride_factor: CHARGE_STRIDE_FACTOR,
            hold: 0,
            transform: true,
        });
    }
    out
}

fn air_clips(def: &CharDef) -> Vec<Clip> {
    let Some(minus_one) = def.states.get(&-1) else {
        return vec![];
    };
    let mut out: Vec<Clip> = vec![];
    for controller in &minus_one.controllers {
        let Some(name) = controller.label.strip_prefix("State -1, air ") else {
            continue;
        };
        let Some(state) = controller
            .raw("value")
            .and_then(|v| v.trim().parse::<i32>().ok())
        else {
            continue;
        };
        if state < AIR_STATE_OFFSET || out.iter().any(|clip| clip.state == Some(state)) {
            continue;
        }
        let mut clip = plain(format!("air {name}"), state, 46.0, true, AIR_FRAMES);
        clip.height = AIR_START_HEIGHT;
        if state - AIR_STATE_OFFSET >= SUPER_STATE {
            clip.frames = SUPER_FRAMES;
            clip.stride_factor = 2;
        }
        out.push(clip);
    }
    out
}

pub async fn run(app: &mut App, dir: &str) {
    let heroes: Vec<String> = std::env::var("UME_CHARS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let rival = std::env::var("UME_RIVAL").unwrap_or_else(|_| "kaito".into());
    let arena = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    let stride: usize = std::env::var("UME_CLIP_STRIDE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let air_only = std::env::var("UME_CLIP_AIR").is_ok();
    for hero in heroes {
        app.mode = Mode::Watch;
        app.sel = (find(app, &hero), find(app, &rival), stage(app, &arena));
        app.start_fight().await;
        {
            let App {
                session, c, audio, ..
            } = app;
            let s = session.as_mut().unwrap();
            for pilot in &mut s.pilots {
                *pilot = crate::session::Pilot::Dummy;
            }
            s.r.debug = std::env::var("UME_BOXES").is_ok();
            for _ in 0..WARM_UP_TICKS {
                if s.m.round_state == 2 && s.m.fighters[0].ctrl {
                    break;
                }
                s.step_visual(c, audio);
            }
        }
        let def: Arc<CharDef> = app.session.as_ref().unwrap().m.fighters[0].def.clone();
        let reel = if air_only {
            air_clips(&def)
        } else {
            clips(&def)
        };
        let label = if air_only {
            format!("{hero}_air")
        } else {
            hero.clone()
        };
        for (index, clip) in reel.into_iter().enumerate() {
            let center = app.session.as_ref().unwrap().m.camera.x;
            {
                let m = &mut app.session.as_mut().unwrap().m;
                m.fighters[0].pos.x = center - clip.gap / 2.0;
                m.fighters[1].pos.x = center + clip.gap / 2.0;
                m.fighters[0].pos.y = clip.height;
                m.fighters[1].pos.y = 0.0;
                m.fighters[0].vel.x = 0.0;
                m.fighters[0].vel.y = 0.0;
                m.fighters[0].facing = 1.0;
                m.fighters[1].facing = -1.0;
                m.fighters[0].life = m.fighters[0].life_max;
                m.fighters[1].life = m.fighters[1].life_max;
                m.fighters[0].power = if clip.hold != 0 { 0.0 } else { 3000.0 };
                let rival_def = m.fighters[1].def.clone();
                m.change_state(1, 0, rival_def);
                m.change_state(0, 0, def.clone());
                if let Some(state) = clip.state {
                    m.change_state(0, state, def.clone());
                }
            }
            if clip.state.is_none() {
                for _ in 0..SETTLE_TICKS {
                    let App {
                        session, c, audio, ..
                    } = app;
                    session.as_mut().unwrap().step_visual(c, audio);
                }
                let m = &mut app.session.as_mut().unwrap().m;
                if clip.transform {
                    m.try_transform(0);
                }
                m.fighters[0].assert_input = (
                    clip.hold,
                    (clip.frames * stride * clip.stride_factor) as i32,
                );
            }
            let mut peak = 0.0f32;
            let name = slug(&clip.name);
            for frame in 0..clip.frames {
                for _ in 0..stride * clip.stride_factor {
                    let App {
                        session, c, audio, ..
                    } = app;
                    let s = session.as_mut().unwrap();
                    s.step_visual(c, audio);
                    peak = peak.max(s.m.fighters[1].life_max - s.m.fighters[1].life);
                }
                app.draw_session_only();
                get_screen_data()
                    .export_png(&format!("{dir}/C-{label}-{index:02}-{name}-{frame:02}.png"));
                next_frame().await;
            }
            let power = app.session.as_ref().unwrap().m.fighters[0].power;
            println!(
                "{label} | {:02} | {} | damage {peak:.0} | power {power:.0}",
                index, clip.name
            );
        }
    }
}
