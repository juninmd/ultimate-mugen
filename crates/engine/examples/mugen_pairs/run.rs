use std::collections::BTreeSet;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::{Event, Match};

const TICKS: usize = 60 * 60;
const STUCK_LIMIT: i32 = 420;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Ok,
    Stuck,
    Panic,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Ok => "OK",
            Verdict::Stuck => "STUCK",
            Verdict::Panic => "PANIC",
        }
    }
}

pub struct Report {
    pub verdict: Verdict,
    pub hits: u32,
    pub distinct_states: usize,
    pub stuck_ticks: i32,
    pub ticks: u32,
}

pub fn run_pair(a: Arc<CharDef>, b: Arc<CharDef>) -> Report {
    let outcome = std::panic::catch_unwind(|| fight(&a, &b));
    outcome.unwrap_or(Report {
        verdict: Verdict::Panic,
        hits: 0,
        distinct_states: 0,
        stuck_ticks: 0,
        ticks: 0,
    })
}

fn fight(a: &Arc<CharDef>, b: &Arc<CharDef>) -> Report {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a.clone(), 0, 6, 1);
    m.add_player(b.clone(), 1, 6, 2);
    m.start_round();
    let (mut ai_a, mut ai_b) = (Ai::new(6, 7), Ai::new(6, 13));
    let mut run = [0i32; 2];
    let mut worst = 0;
    let mut hits = 0u32;
    let mut states = BTreeSet::new();
    let mut ticks = 0u32;
    for _ in 0..TICKS {
        ticks += 1;
        let roots = m.root_indices();
        let ia = ai_a.poll(&m, roots[0]);
        let ib = ai_b.poll(&m, roots[1]);
        m.tick(&[ia, ib]);
        for e in m.events.drain(..) {
            if matches!(e, Event::Hit { .. }) {
                hits += 1;
            }
        }
        for (k, &r) in roots.iter().enumerate() {
            let f = &m.fighters[r];
            states.insert(f.state_no);
            let stuck = m.round_state == 2
                && f.alive
                && !f.ctrl
                && f.hit_pause == 0
                && f.hit_shake == 0
                && m.pause.is_none()
                && f.bind.is_none();
            run[k] = if stuck { run[k] + 1 } else { 0 };
            worst = worst.max(run[k]);
        }
        if m.match_over {
            break;
        }
    }
    let verdict = if worst > STUCK_LIMIT {
        Verdict::Stuck
    } else {
        Verdict::Ok
    };
    Report {
        verdict,
        hits,
        distinct_states: states.len(),
        stuck_ticks: worst,
        ticks,
    }
}
