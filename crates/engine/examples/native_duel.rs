use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const ROUND_TICKS: usize = 60 * 99;
const BASE_SEEDS: [(u32, u32); 3] = [(7, 13), (21, 5), (99, 3)];
const STUCK_LIMIT: i32 = 420;

#[derive(Default, Clone)]
struct Tally {
    wins: f32,
    games: f32,
    damage_dealt: f32,
    hits: f32,
    best_combo: i32,
    combo_hits_total: f32,
    combos: f32,
}

struct Duel {
    winner: Option<usize>,
    ticks: usize,
    knockout: bool,
    damage: [f32; 2],
    hits: [i32; 2],
    best_combo: [i32; 2],
    stuck_ticks: [i32; 2],
}

fn load(dir: &str) -> Vec<(String, Arc<ume_engine::chardef::CharDef>)> {
    let mut chars: Vec<NativeChar> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let stem = path.file_stem()?.to_string_lossy().to_string();
            (path.extension()? == "json" && stem != "index")
                .then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
        })
        .collect();
    chars.sort_by(|a, b| a.id.cmp(&b.id));
    let skipped = std::env::var("UME_DUEL_SKIP").unwrap_or_default();
    chars.retain(|nc| !skipped.split(',').any(|id| id == nc.id));
    chars
        .iter()
        .map(|nc| (nc.id.clone(), Arc::new(compile_native(nc, 1, true))))
        .collect()
}

fn duel(
    a: &Arc<ume_engine::chardef::CharDef>,
    b: &Arc<ume_engine::chardef::CharDef>,
    seeds: (u32, u32),
    level: i32,
) -> Duel {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a.clone(), 0, 0, 1);
    m.add_player(b.clone(), 1, 0, 2);
    m.start_round();
    let (mut ai_a, mut ai_b) = (Ai::new(level, seeds.0), Ai::new(level, seeds.1));
    let roots = m.root_indices();
    let mut result = Duel {
        winner: None,
        ticks: 0,
        knockout: false,
        damage: [0.0; 2],
        hits: [0; 2],
        best_combo: [0; 2],
        stuck_ticks: [0; 2],
    };
    let mut run = [0i32; 2];
    let mut fighting = 0;
    for tick in 0..ROUND_TICKS + 60 * 8 {
        let inputs = [ai_a.poll(&m, roots[0]), ai_b.poll(&m, roots[1])];
        m.tick(&inputs);
        if m.round_state != 2 && fighting == 0 {
            continue;
        }
        if m.round_state == 2 {
            fighting += 1;
        }
        for k in 0..2 {
            let f = &m.fighters[roots[k]];
            result.best_combo[k] = result.best_combo[k].max(f.combo_count);
            let stuck = m.round_state == 2
                && f.alive
                && !f.ctrl
                && f.hit_pause == 0
                && f.hit_shake == 0
                && m.pause.is_none()
                && f.bind.is_none();
            run[k] = if stuck { run[k] + 1 } else { 0 };
            if run[k] > STUCK_LIMIT {
                result.stuck_ticks[k] += 1;
            }
        }
        let lives = [m.fighters[roots[0]].life, m.fighters[roots[1]].life];
        if lives.iter().any(|life| *life <= 0.0) || fighting >= ROUND_TICKS || m.round_state > 2 {
            result.knockout = lives.iter().any(|life| *life <= 0.0);
            result.ticks = fighting;
            result.damage = [
                m.fighters[roots[1]].received_damage,
                m.fighters[roots[0]].received_damage,
            ];
            result.hits = [
                m.fighters[roots[1]].received_hits,
                m.fighters[roots[0]].received_hits,
            ];
            result.winner = if lives[0] > lives[1] {
                Some(0)
            } else if lives[1] > lives[0] {
                Some(1)
            } else {
                None
            };
            return result;
        }
        let _ = tick;
    }
    result.ticks = fighting;
    result
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/chars".into());
    let roster = load(&dir);
    let level: i32 = std::env::var("UME_DUEL_LEVEL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let offset: u32 = std::env::var("UME_DUEL_SEED_OFFSET")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let seeds: Vec<(u32, u32)> = BASE_SEEDS
        .iter()
        .map(|(a, b)| (a + offset, b + offset * 3))
        .collect();
    let mut tallies = vec![Tally::default(); roster.len()];
    let (mut games, mut knockouts, mut total_ticks, mut draws, mut stuck_games) =
        (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let (mut total_damage, mut total_hits) = (0.0f32, 0.0f32);
    for i in 0..roster.len() {
        for j in (i + 1)..roster.len() {
            for seeds in seeds.iter().copied() {
                for (first, second) in [(i, j), (j, i)] {
                    let r = duel(&roster[first].1, &roster[second].1, seeds, level);
                    games += 1.0;
                    total_ticks += r.ticks as f32;
                    if r.knockout {
                        knockouts += 1.0;
                    }
                    if r.stuck_ticks.iter().any(|t| *t > 0) {
                        stuck_games += 1.0;
                    }
                    total_damage += r.damage[0] + r.damage[1];
                    total_hits += (r.hits[0] + r.hits[1]) as f32;
                    match r.winner {
                        Some(side) => tallies[if side == 0 { first } else { second }].wins += 1.0,
                        None => draws += 1.0,
                    }
                    for (side, idx) in [(0, first), (1, second)] {
                        let t = &mut tallies[idx];
                        t.games += 1.0;
                        t.damage_dealt += r.damage[side];
                        t.hits += r.hits[side] as f32;
                        t.best_combo = t.best_combo.max(r.best_combo[side]);
                    }
                }
            }
        }
    }
    println!(
        "roster {} fighters, {} CPU-vs-CPU rounds (level {level}, {} seeds, both sides)",
        roster.len(),
        games,
        seeds.len()
    );
    let mut rows: Vec<(f32, String)> = roster
        .iter()
        .zip(&tallies)
        .map(|((id, _), t)| {
            (
                t.wins / t.games * 100.0,
                format!(
                    "{:8} win {:5.1}%  dmg/round {:6.0}  hits/round {:5.1}  best combo {:2}",
                    id,
                    t.wins / t.games * 100.0,
                    t.damage_dealt / t.games,
                    t.hits / t.games,
                    t.best_combo
                ),
            )
        })
        .collect();
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (_, line) in &rows {
        println!("{line}");
    }
    let rates: Vec<f32> = rows.iter().map(|r| r.0).collect();
    let mean = rates.iter().sum::<f32>() / rates.len() as f32;
    let spread =
        (rates.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / rates.len() as f32).sqrt();
    println!(
        "win-rate spread (std dev) {spread:.1} pts, best {:.1}%, worst {:.1}%",
        rates[0],
        rates[rates.len() - 1]
    );
    println!("knockout rate {:.1}%, draws {:.1}%, average round {:.1}s, avg damage/round {:.0}, avg hits/round {:.1}, rounds with a stuck fighter {}", knockouts / games * 100.0, draws / games * 100.0, total_ticks / games / 60.0, total_damage / games, total_hits / games, stuck_games);
    let _ = (
        tallies
            .iter()
            .map(|t| t.combo_hits_total + t.combos)
            .sum::<f32>(),
        (),
    );
}
