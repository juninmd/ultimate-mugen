use crate::routes::Route;
use serde::Serialize;
use std::fmt::Write as _;
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::commands::input_macro;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::{Event, Match};

pub const TICKS: usize = 150;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Outcome {
    pub states: Vec<i32>,
    pub damage: f32,
    pub sounds: usize,
    pub helpers: usize,
}

pub struct Trace {
    pub lines: Vec<String>,
    pub outcome: Outcome,
}

fn duel(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    for _ in 0..800 {
        m.tick(&[0, 0]);
        if m.round_state == 2 {
            break;
        }
    }
    m.fighters[0].pos.x = -30.0;
    m.fighters[1].pos.x = 30.0;
    m.fighters[0].power = m.fighters[0].power_max;
    m.fighters[1].life_max = 100000.0;
    m.fighters[1].life = 100000.0;
    m.events.clear();
    m
}

fn inputs(def: &CharDef, route: &Route) -> Vec<u32> {
    let Some(cmd) = def.commands.iter().find(|c| c.name.eq_ignore_ascii_case(&route.command)) else { return vec![] };
    let mut seq = match route.when {
        "air" => vec![k::U, k::U, k::U, 0, 0, 0, 0, 0, 0, 0],
        _ => vec![],
    };
    let hold = if route.when == "crouch" { k::D } else { 0 };
    if hold != 0 {
        seq.extend([hold; 4]);
    }
    seq.extend(input_macro(cmd, 1.0).into_iter().map(|b| b | hold));
    seq
}

fn snapshot(m: &Match, t: usize, out: &mut String) {
    let _ = write!(out, "{t}:");
    for f in &m.fighters {
        let _ = write!(out, " [{} {} {} {:.3},{:.3} {:.3},{:.3} {} {:.1} {:.1}]", f.state_no, f.anim_no, f.elem, f.pos.x, f.pos.y, f.vel.x, f.vel.y, f.ctrl as u8, f.life, f.power);
    }
}

pub fn play(def: &Arc<CharDef>, route: &Route, mut on_tick: impl FnMut(&mut Match, usize)) -> f32 {
    let mut m = duel(def);
    let seq = inputs(def, route);
    let life = m.fighters[1].life;
    for t in 0..TICKS {
        m.tick(&[seq.get(t).copied().unwrap_or(0), 0]);
        on_tick(&mut m, t);
    }
    life - m.fighters[1].life
}

pub fn run(def: &Arc<CharDef>, route: &Route) -> Trace {
    let mut trace = Trace { lines: Vec::with_capacity(TICKS), outcome: Outcome::default() };
    let damage = play(def, route, |m, t| {
        let mut line = String::new();
        snapshot(m, t, &mut line);
        for ev in m.events.drain(..) {
            if let Event::Sound { group, index, own, .. } = ev {
                let _ = write!(line, " snd{}{group},{index}", if own { "S" } else { "F" });
                trace.outcome.sounds += 1;
            }
        }
        let s = m.fighters[0].state_no;
        if trace.outcome.states.last() != Some(&s) {
            trace.outcome.states.push(s);
        }
        trace.outcome.helpers = trace.outcome.helpers.max(m.fighters.len().saturating_sub(2));
        trace.lines.push(line);
    });
    trace.outcome.damage = damage;
    trace
}

pub fn first_divergence(a: &Trace, b: &Trace) -> Option<(usize, String, String)> {
    a.lines.iter().zip(&b.lines).enumerate().find(|(_, (x, y))| x != y).map(|(t, (x, y))| (t, x.clone(), y.clone()))
}
