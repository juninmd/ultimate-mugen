use std::collections::BTreeMap;
use serde_json::{json, Value};
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::{Event, Match};

const STALL_TICKS: usize = 300;

#[derive(Default)]
struct Tally {
    fights: u32,
    kos: u32,
    timeouts: u32,
    ko_frames: u64,
    hits: u32,
    guards: u32,
    hyper: u32,
    stall_ticks: u64,
    total_ticks: u64,
    longest_stall: usize,
    max_combo: i32,
    wins: BTreeMap<String, (u32, u32)>,
}

fn fight(roster: &[String], a: usize, b: usize, level: i32, frames: usize, seed: u32, t: &mut Tally) -> Result<(), String> {
    let p1 = crate::debug_fight::load_fighter(&roster[a])?;
    let p2 = crate::debug_fight::load_fighter(&roster[b])?;
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(p1, 0, 8, 1);
    m.add_player(p2, 1, 8, 2);
    m.start_round();
    m.round_state = 2;
    for f in m.fighters.iter_mut() {
        f.ctrl = true;
    }
    let mut ai0 = Ai::new(level, seed);
    let mut ai1 = Ai::new(level, seed + 1);
    let mut quiet = 0usize;
    t.fights += 1;
    let mut ended = false;
    for tick in 0..frames {
        let i0 = ai0.poll(&m, 0);
        let i1 = ai1.poll(&m, 1);
        m.tick(&[i0, i1]);
        t.total_ticks += 1;
        let mut hit = false;
        for ev in m.events.drain(..).collect::<Vec<_>>() {
            match ev {
                Event::Hit { guarded, heavy, .. } => {
                    hit = true;
                    t.hits += 1;
                    t.guards += guarded as u32;
                    t.hyper += heavy as u32;
                }
                _ => {}
            }
        }
        for f in m.fighters.iter() {
            t.max_combo = t.max_combo.max(f.combo_count);
        }
        if hit {
            quiet = 0;
        } else {
            quiet += 1;
            if quiet > STALL_TICKS {
                t.stall_ticks += 1;
            }
            t.longest_stall = t.longest_stall.max(quiet);
        }
        if m.fighters.iter().any(|f| f.life <= 0.0) {
            let winner = if m.fighters[0].life >= m.fighters[1].life { a } else { b };
            let loser = if winner == a { b } else { a };
            t.wins.entry(roster[winner].clone()).or_default().0 += 1;
            t.wins.entry(roster[loser].clone()).or_default().1 += 1;
            t.kos += 1;
            t.ko_frames += tick as u64;
            ended = true;
            break;
        }
    }
    if !ended {
        t.timeouts += 1;
    }
    Ok(())
}

pub fn run(roster: &[String], level: i32, frames: usize) -> Result<Value, String> {
    let mut t = Tally::default();
    let n = roster.len();
    for i in 0..n {
        for j in (0..n).filter(|&j| j != i) {
            fight(roster, i, j, level, frames, 11 + (i * n + j) as u32, &mut t)?;
        }
    }
    let avg_ko = if t.kos == 0 { 0.0 } else { t.ko_frames as f64 / t.kos as f64 / 60.0 };
    let stall_pct = t.stall_ticks as f64 / t.total_ticks.max(1) as f64 * 100.0;
    Ok(json!({
        "level": level,
        "fights": t.fights,
        "kos": t.kos,
        "timeouts": t.timeouts,
        "avg_ko_seconds": (avg_ko * 10.0).round() / 10.0,
        "hits": t.hits,
        "guard_pct": (t.guards as f64 / t.hits.max(1) as f64 * 1000.0).round() / 10.0,
        "heavy_hits": t.hyper,
        "stall_pct": (stall_pct * 10.0).round() / 10.0,
        "longest_stall_seconds": t.longest_stall as f64 / 60.0,
        "max_combo": t.max_combo,
        "win_pct": t.wins.iter().map(|(k, (w, l))| (k.clone(), (*w as f64 / (*w + *l).max(1) as f64 * 1000.0).round() / 10.0)).collect::<BTreeMap<_, _>>(),
    }))
}
