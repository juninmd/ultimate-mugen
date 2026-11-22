#![allow(dead_code)]
use super::gogeta::Duel;
use super::render::{frame, sheet};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::Arc;
use tiny_skia::Pixmap;
use ume_engine::chardef::CharDef;
use ume_engine::matchs::{Event, Match};

pub struct Run {
    pub shots: Vec<Pixmap>,
    pub sounds: BTreeSet<(i32, i32)>,
    pub missing: BTreeSet<(i32, i32)>,
    pub states: BTreeSet<i32>,
    pub min: (f32, f32),
    pub power: (f32, f32),
    pub end: (i32, bool, f32),
    pub damage: f32,
}

impl Run {
    pub fn empty() -> Self {
        Run { shots: vec![], sounds: BTreeSet::new(), missing: BTreeSet::new(), states: BTreeSet::new(), min: (0.0, 0.0), power: (0.0, 0.0), end: (0, false, 0.0), damage: 0.0 }
    }

    pub fn start(r: &Duel) -> Self {
        let f = &r.m.fighters[0];
        Run { min: (f.pos.x, f.pos.y), power: (f.power, f.power), ..Run::empty() }
    }

    pub fn played(&self) -> BTreeSet<(i32, i32)> {
        self.sounds.difference(&self.missing).copied().collect()
    }
}

pub fn sounds(m: &mut Match, run: &mut Run) {
    for ev in m.events.drain(..) {
        if let Event::Sound { def: Some(def), group, index, own: true, .. } = ev {
            run.sounds.insert((group, index));
            if !def.sounds.contains_key(&(group, index)) {
                run.missing.insert((group, index));
            }
        }
    }
}

pub fn observe(r: &mut Duel, ticks: usize, every: usize, run: &mut Run) {
    let life = r.m.fighters[1].life;
    for t in 0..ticks {
        r.tick(1);
        sounds(&mut r.m, run);
        let f = &r.m.fighters[0];
        run.states.insert(f.state_no);
        run.min = (run.min.0.min(f.pos.x), run.min.1.min(f.pos.y));
        if t % every.max(1) == every.max(1) - 1 && run.shots.len() < 6 {
            run.shots.push(frame(&r.m, false));
        }
    }
    let f = &r.m.fighters[0];
    run.end = (f.state_no, f.ctrl, f.pos.y);
    run.damage += life - r.m.fighters[1].life;
}

pub fn forced(d: &Arc<CharDef>, state: i32, ticks: usize, airborne: bool) -> Run {
    let mut r = Duel::new(d.clone(), 50.0);
    if airborne {
        r.m.fighters[0].pos.y = -70.0;
    }
    r.m.events.clear();
    let mut run = Run::start(&r);
    r.m.change_state(0, state, d.clone());
    observe(&mut r, ticks, ticks / 6, &mut run);
    run
}

pub fn move_state(d: &Arc<CharDef>, command: &str) -> Option<i32> {
    d.movelist.iter().find(|mv| mv.command == command).map(|mv| mv.state)
}

pub fn row(report: &mut String, name: &str, side: &str, r: &Run) {
    let _ = writeln!(report, "| {name} | {side} | {:?} | ({:.0}, {:.0}) | {:.0} -> {:.0} | {:.0} | {:?} | {:?} | {:?} |",
        r.states, r.min.0, r.min.1, r.power.0, r.power.1, r.damage, r.sounds, r.missing, r.end);
}

pub fn save(out: &std::path::Path, name: &str, a: &Run, b: &Run) {
    let mut all: Vec<Pixmap> = a.shots.clone();
    all.resize_with(6, || frame(&Match::new(ume_engine::common::common_def(), None, ume_engine::common::flat_stage()), false));
    all.extend(b.shots.iter().cloned());
    sheet(&all, 6).save_png(out.join(format!("{name}.png"))).unwrap();
}
