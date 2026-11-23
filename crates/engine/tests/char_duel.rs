use crate::support;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use support::duel::{load_spec, mean_ci, wilson};
use support::render::{frame, sheet};
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::{Event, Match};

#[derive(Default, Clone)]
struct Side {
    damage: f32,
    hits: u32,
    guarded: u32,
    supers: u32,
    transforms: u32,
    cmds: BTreeMap<String, u32>,
}

struct Game {
    winner: i32,
    fight_ticks: i32,
    ko_tick: Option<i32>,
    life_frac: [f32; 2],
    side: [Side; 2],
}

fn play(defs: [&Arc<CharDef>; 2], level: i32, seeds: [u32; 2], shots: Option<&Path>) -> Game {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(defs[0].clone(), 0, level, 1);
    m.add_player(defs[1].clone(), 1, level, 2);
    m.start_round();
    let mut ai = [Ai::new(level, seeds[0]), Ai::new(level, seeds[1])];
    let roots = m.root_indices();
    let mut g = Game {
        winner: -1,
        fight_ticks: 0,
        ko_tick: None,
        life_frac: [1.0; 2],
        side: Default::default(),
    };
    let mut prev: [Vec<String>; 2] = Default::default();
    let mut caps: Vec<(String, _)> = vec![];
    let mut want = ["fight", "first-hit", "first-super", "mid", "ko"]
        .map(String::from)
        .to_vec();
    for _ in 0..(60 * 120) {
        let inputs = [ai[0].poll(&m, roots[0]), ai[1].poll(&m, roots[1])];
        m.tick(&inputs);
        if m.round_state == 2 {
            g.fight_ticks += 1;
        }
        let snap = |name: &str, m: &Match, want: &mut Vec<String>, caps: &mut Vec<(String, _)>| {
            if shots.is_some() && want.iter().any(|w| w == name) {
                want.retain(|w| w != name);
                caps.push((name.to_string(), frame(m, false)));
            }
        };
        let mut later: Vec<&str> = vec![];
        for e in m.events.drain(..) {
            match e {
                Event::Hit {
                    attacker_side,
                    damage,
                    guarded,
                    ..
                } => {
                    let s = &mut g.side[attacker_side.min(1)];
                    s.damage += damage;
                    if guarded {
                        s.guarded += 1
                    } else {
                        s.hits += 1
                    }
                    later.push("first-hit");
                }
                Event::Super { side, .. } => {
                    g.side[side.min(1)].supers += 1;
                    later.push("first-super");
                }
                Event::Transform { fighter } => {
                    if let Some(i) = roots.iter().position(|&r| m.fighters[r].id == fighter) {
                        g.side[i].transforms += 1;
                    }
                }
                _ => {}
            }
        }
        for n in later {
            snap(n, &m, &mut want, &mut caps);
        }
        if g.fight_ticks == 5 {
            snap("fight", &m, &mut want, &mut caps);
        }
        if g.fight_ticks == 900 {
            snap("mid", &m, &mut want, &mut caps);
        }
        for k in 0..2 {
            let mut now: Vec<String> = m.fighters[roots[k]]
                .cmd
                .active
                .iter()
                .map(|s| s.to_string())
                .collect();
            now.sort();
            for c in &now {
                if !prev[k].contains(c) && !c.starts_with("hold") {
                    *g.side[k].cmds.entry(c.clone()).or_default() += 1;
                }
            }
            prev[k] = now;
        }
        let alive = [m.fighters[roots[0]].alive, m.fighters[roots[1]].alive];
        if g.ko_tick.is_none() && (!alive[0] || !alive[1]) {
            g.ko_tick = Some(g.fight_ticks);
            for _ in 0..25 {
                m.tick(&[0, 0]);
            }
            snap("ko", &m, &mut want, &mut caps);
        }
        if m.round_state >= 3 {
            break;
        }
    }
    for (k, &r) in roots.iter().enumerate().take(2) {
        let f = &m.fighters[r];
        g.life_frac[k] = (f.life / f.life_max).clamp(0.0, 1.0);
    }
    g.winner = if g.life_frac[0] > g.life_frac[1] {
        0
    } else if g.life_frac[1] > g.life_frac[0] {
        1
    } else {
        -1
    };
    if let Some(dir) = shots {
        let order = ["fight", "first-hit", "first-super", "mid", "ko"];
        let frames: Vec<_> = order
            .iter()
            .filter_map(|n| caps.iter().find(|c| c.0 == *n).map(|c| c.1.clone()))
            .collect();
        for (n, px) in &caps {
            px.save_png(dir.join(format!("{n}.png"))).unwrap();
        }
        if !frames.is_empty() {
            sheet(&frames, 2).save_png(dir.join("sheet.png")).unwrap();
        }
    }
    g
}

#[test]
#[ignore = "needs external MUGEN characters or UME_EVIDENCE_DIR; run with --ignored"]
fn seeded_duels_report_win_rate_and_damage() {
    let (Ok(a), Ok(b), Ok(out)) = (
        std::env::var("UME_DUEL_A"),
        std::env::var("UME_DUEL_B"),
        std::env::var("UME_EVIDENCE_DIR"),
    ) else {
        return;
    };
    let n: u32 = std::env::var("UME_DUEL_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let level: i32 = std::env::var("UME_DUEL_LEVEL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let tag = std::env::var("UME_DUEL_TAG").unwrap_or_else(|_| "duel".into());
    let out = PathBuf::from(out);
    let (da, db) = (load_spec(&a), load_spec(&b));
    let (mut score, mut w, mut l, mut d) = (vec![], 0, 0, 0);
    let mut cols: BTreeMap<&str, [Vec<f64>; 2]> = BTreeMap::new();
    let (mut cmd_tot, mut rows) = ([BTreeMap::<String, u32>::new(), BTreeMap::new()], vec![]);
    let mut sup = [0u32; 2];
    for seed in 1..=n {
        for swap in 0..2usize {
            let seeds = [seed * 7919 + 1, seed * 104729 + 3];
            let defs = if swap == 0 { [&da, &db] } else { [&db, &da] };
            let shots = (seed == 1).then(|| {
                let p = out.join(format!(
                    "{tag}-frames-{}",
                    if swap == 0 { "A-left" } else { "A-right" }
                ));
                std::fs::create_dir_all(&p).unwrap();
                p
            });
            let g = play(defs, level, seeds, shots.as_deref());
            let (ia, ib) = (swap, 1 - swap);
            let res = if g.winner < 0 {
                d += 1;
                0.5
            } else if g.winner as usize == ia {
                w += 1;
                1.0
            } else {
                l += 1;
                0.0
            };
            score.push(res);
            let secs = (g.fight_ticks.max(1) as f64) / 60.0;
            for (who, i) in [(0usize, ia), (1, ib)] {
                let s = &g.side[i];
                cols.entry("dmg_dealt").or_default()[who].push(s.damage as f64);
                cols.entry("dps").or_default()[who].push(s.damage as f64 / secs);
                cols.entry("life_left").or_default()[who].push(g.life_frac[i] as f64);
                cols.entry("hits").or_default()[who].push(s.hits as f64);
                cols.entry("guarded").or_default()[who].push(s.guarded as f64);
                cols.entry("supers").or_default()[who].push(s.supers as f64);
                sup[who] += s.supers;
                for (c, k) in &s.cmds {
                    *cmd_tot[who].entry(c.clone()).or_default() += k;
                }
            }
            cols.entry("seconds").or_default()[0].push(secs);
            cols.entry("ko").or_default()[0].push(g.ko_tick.is_some() as u8 as f64);
            rows.push(json!({"seed": seed, "a_side": swap, "winner": match g.winner { -1 => "draw", x if x as usize == ia => "A", _ => "B" }, "seconds": secs, "ko": g.ko_tick.is_some(), "a_life": g.life_frac[ia], "b_life": g.life_frac[ib], "a_dmg": g.side[ia].damage, "b_dmg": g.side[ib].damage}));
        }
    }
    let games = score.len() as f64;
    let s = score.iter().sum::<f64>() / games;
    let stats: BTreeMap<_, _> = cols
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                json!({"A": mean_ci(&v[0]), "B": mean_ci(&v[1])}),
            )
        })
        .collect();
    let rep = json!({"A": a, "B": b, "level": level, "seeds": n, "games": games, "A_wins": w, "B_wins": l, "draws": d,
        "A_score": s, "A_score_ci95": wilson(s, games), "stats_mean_lo_hi": stats, "A_commands": cmd_tot[0], "B_commands": cmd_tot[1], "supers_total": sup, "rows": rows});
    std::fs::write(
        out.join(format!("{tag}.json")),
        serde_json::to_string_pretty(&rep).unwrap(),
    )
    .unwrap();
    println!(
        "{tag}: games {games} A {w} B {l} draw {d} score {s:.3} ci {:?} supers {sup:?}",
        wilson(s, games)
    );
}
