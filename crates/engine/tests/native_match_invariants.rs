use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const ROUND_TICKS: usize = 60 * 40;
const SEEDS: [(u32, u32); 2] = [(7, 13), (31, 4)];
const FIRST_ABILITY_STATE: i32 = 200;
const FIRST_COMMON_STATE: i32 = 5000;
const LONGEST_AIRBORNE: i32 = 300;
const LONGEST_WITHOUT_CONTROL: i32 = 420;
const LONGEST_ATTACK: i32 = 240;
const OFF_STAGE_MARGIN: f32 = 2.0;
const LONGEST_BELOW_FLOOR: i32 = 3;
const LONGEST_OFF_CAMERA: i32 = 3;
const OFF_CAMERA_MARGIN: f32 = 10.0;
const LYING_STATES: [i32; 5] = [5110, 5120, 5150, 5170, 5900];

fn roster() -> Vec<(String, Arc<CharDef>)> {
    let mut chars: Vec<NativeChar> = std::fs::read_dir("../../assets/chars")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let stem = path.file_stem()?.to_string_lossy().to_string();
            (path.extension()? == "json" && stem != "index")
                .then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
        })
        .collect();
    chars.sort_by(|a, b| a.id.cmp(&b.id));
    chars
        .iter()
        .map(|nc| (nc.id.clone(), Arc::new(compile_native(nc, 1, true))))
        .collect()
}

#[derive(Default, Clone, Copy)]
struct Streaks {
    airborne: i32,
    no_control: i32,
    attacking: i32,
    attack_state: i32,
    sunk: i32,
    hidden: i32,
}

fn play(
    a: &Arc<CharDef>,
    b: &Arc<CharDef>,
    seeds: (u32, u32),
    used: &mut [BTreeSet<i32>; 2],
    problems: &mut Vec<String>,
    label: &str,
) {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a.clone(), 0, 0, 1);
    m.add_player(b.clone(), 1, 0, 2);
    m.start_round();
    let (mut ai_a, mut ai_b) = (Ai::new(6, seeds.0), Ai::new(6, seeds.1));
    let roots = m.root_indices();
    let mut streaks = [Streaks::default(); 2];
    let mut reported: BTreeSet<String> = BTreeSet::new();
    let mut report = |problems: &mut Vec<String>, kind: &str, who: usize, detail: String| {
        if reported.insert(format!("{kind}{who}")) {
            problems.push(format!("{label} p{} {kind}: {detail}", who + 1));
        }
    };
    for _ in 0..ROUND_TICKS {
        let inputs = [ai_a.poll(&m, roots[0]), ai_b.poll(&m, roots[1])];
        m.tick(&inputs);
        if m.round_state != 2 {
            continue;
        }
        for side in 0..2 {
            let f = &m.fighters[roots[side]];
            used[side].insert(f.state_no);
            let finite = f.pos.x.is_finite()
                && f.pos.y.is_finite()
                && f.vel.x.is_finite()
                && f.vel.y.is_finite()
                && f.life.is_finite()
                && f.power.is_finite();
            if !finite {
                report(
                    problems,
                    "non-finite",
                    side,
                    format!(
                        "pos {:?} vel {:?} life {} power {}",
                        f.pos, f.vel, f.life, f.power
                    ),
                );
                continue;
            }
            if f.pos.x.abs() > m.stage.bound_right + m.view_w / 2.0 + OFF_STAGE_MARGIN {
                report(
                    problems,
                    "off-stage",
                    side,
                    format!(
                        "x {:.0} in state {} (wall {})",
                        f.pos.x,
                        f.state_no,
                        m.stage.bound_right + m.view_w / 2.0
                    ),
                );
            }
            if f.life < 0.0 || f.life > f.life_max + 0.5 {
                report(
                    problems,
                    "life-range",
                    side,
                    format!("life {} of {}", f.life, f.life_max),
                );
            }
            if f.power < 0.0 || f.power > f.power_max + 0.5 {
                report(
                    problems,
                    "power-range",
                    side,
                    format!("power {} of {}", f.power, f.power_max),
                );
            }
            let frozen =
                m.pause.is_some() || f.hit_pause > 0 || f.hit_shake > 0 || f.bind.is_some();
            let s = &mut streaks[side];
            s.sunk = if f.pos.y <= 1.5 {
                0
            } else if frozen {
                s.sunk
            } else {
                s.sunk + 1
            };
            if s.sunk > LONGEST_BELOW_FLOOR {
                report(
                    problems,
                    "below-floor",
                    side,
                    format!(
                        "y {:.1} for {} ticks in state {}",
                        f.pos.y, s.sunk, f.state_no
                    ),
                );
            }
            let outside_view = (f.pos.x - m.camera.x).abs() > m.view_w / 2.0 + OFF_CAMERA_MARGIN;
            s.hidden = if outside_view { s.hidden + 1 } else { 0 };
            if s.hidden > LONGEST_OFF_CAMERA {
                report(
                    problems,
                    "off-camera",
                    side,
                    format!(
                        "x {:.0} while the camera is at {:.0} in state {}",
                        f.pos.x, m.camera.x, f.state_no
                    ),
                );
            }
            s.airborne = if f.alive && f.pos.y < -1.0 && !frozen {
                s.airborne + 1
            } else {
                0
            };
            s.no_control = if f.alive && !f.ctrl && !frozen && !LYING_STATES.contains(&f.state_no) {
                s.no_control + 1
            } else {
                0
            };
            s.attacking =
                if f.alive && f.move_type == 'A' && f.state_no == s.attack_state && !frozen {
                    s.attacking + 1
                } else {
                    0
                };
            s.attack_state = f.state_no;
            if s.airborne > LONGEST_AIRBORNE {
                report(
                    problems,
                    "hover",
                    side,
                    format!("{} ticks off the floor in state {}", s.airborne, f.state_no),
                );
            }
            if s.no_control > LONGEST_WITHOUT_CONTROL {
                report(
                    problems,
                    "no-control",
                    side,
                    format!(
                        "{} ticks without control in state {}",
                        s.no_control, f.state_no
                    ),
                );
            }
            if s.attacking > LONGEST_ATTACK {
                report(
                    problems,
                    "endless-attack",
                    side,
                    format!("{} ticks in attack state {}", s.attacking, f.state_no),
                );
            }
        }
    }
}

#[test]
fn every_match_keeps_fighters_on_stage_in_range_and_in_control() {
    let roster = roster();
    let mut problems = Vec::new();
    let mut used: BTreeMap<String, BTreeSet<i32>> = BTreeMap::new();
    let mut matches = 0;
    for (a_index, a) in roster.iter().enumerate() {
        for (b_index, b) in roster.iter().enumerate() {
            for seeds in SEEDS {
                let mut pair_used = [BTreeSet::new(), BTreeSet::new()];
                play(
                    &a.1,
                    &b.1,
                    seeds,
                    &mut pair_used,
                    &mut problems,
                    &format!("{} vs {}", a.0, b.0),
                );
                used.entry(roster[a_index].0.clone())
                    .or_default()
                    .extend(pair_used[0].iter().copied());
                used.entry(roster[b_index].0.clone())
                    .or_default()
                    .extend(pair_used[1].iter().copied());
                matches += 1;
            }
        }
    }
    let mut unused = Vec::new();
    for (id, def) in &roster {
        let declared: BTreeSet<i32> = def
            .states
            .keys()
            .copied()
            .filter(|n| (FIRST_ABILITY_STATE..FIRST_COMMON_STATE).contains(n))
            .collect();
        let reached = &used[id];
        let missed: Vec<i32> = declared
            .iter()
            .copied()
            .filter(|n| !reached.contains(n))
            .collect();
        println!(
            "{id:8} ability states {:3}, reached by the CPU {:3}, never reached {:?}",
            declared.len(),
            declared.len() - missed.len(),
            missed
        );
        unused.push((id.clone(), declared.len(), missed));
    }
    println!("{matches} matches played, {} problems", problems.len());
    assert!(
        problems.is_empty(),
        "match invariants broken:\n{}",
        problems.join("\n")
    );
}
