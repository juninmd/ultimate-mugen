mod support;
use support::gogeta::*;
use ume_engine::input::InputSource;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

#[test]
fn fuzzed_duels_never_need_the_watchdog() {
    let Some(d) = load(1) else { return };
    let cmds: Vec<_> = d
        .commands
        .iter()
        .filter(|c| !c.name.starts_with("ai") && !c.name.starts_with("hold"))
        .collect();
    let mut report = String::new();
    let mut rescued_total = 0;
    let seeds: u64 = std::env::var("UME_FUZZ_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8);
    for seed in 1..=seeds {
        let mut rng = Lcg(seed * 7919);
        let mut r = Duel::new(d.clone(), 80.0);
        for f in r.m.fighters.iter_mut() {
            f.life = 1.0e7;
            f.life_max = 1.0e7;
        }
        let mut max_hit_ctrl = 0;
        let mut hist: [Vec<(i32, i32)>; 2] = [vec![], vec![]];
        let mut streak = [0i32; 2];
        let mut worst_noctrl = 0;
        for t in 0..12000 {
            for p in 0..2 {
                if r.q[p].is_empty() {
                    if rng.next() % 2 == 0 {
                        let c = cmds[rng.next() as usize % cmds.len()];
                        let s = synth(c);
                        r.push(p, &s);
                    } else {
                        let bits = (rng.next() as u32) & 0x3FF & !0x300;
                        let n = 2 + rng.next() as usize % 10;
                        r.push(p, &vec![bits; n]);
                    }
                }
            }
            if t % 600 == 0 {
                for f in r.m.fighters.iter_mut() {
                    f.power = f.power_max;
                }
            }
            if r.m.round_state != 2 {
                r.m.round_state = 2;
            }
            r.tick(1);
            for p in 0..2 {
                let f = &r.m.fighters[p];
                streak[p] = if f.move_type == 'H' && f.ctrl {
                    streak[p] + 1
                } else {
                    0
                };
                max_hit_ctrl = max_hit_ctrl.max(streak[p]);
                worst_noctrl = worst_noctrl.max(f.no_ctrl_ticks);
                if hist[p].last().map(|x| x.1) != Some(f.state_no) {
                    hist[p].push((t, f.state_no));
                }
                if f.no_ctrl_ticks == 399 {
                    let h: Vec<_> = hist[p].iter().rev().take(14).collect();
                    println!("HIST seed {seed} p{p} {h:?}");
                }
                if f.no_ctrl_ticks == 399 {
                    let o = &r.m.fighters[1 - p];
                    println!("STUCK seed {seed} p{p} state {} time {} anim {} pos {:?} vel {:?} pause {} shake {} | opp state {} time {} anim {} bind {:?} movetype {} hit_pause {}", f.state_no, f.state_time, f.anim_no, f.pos, f.vel, f.hit_pause, f.hit_shake, o.state_no, o.state_time, o.anim_no, f.bind.is_some(), f.move_type, f.hit_pause);
                }
            }
        }
        let rescued: u32 = r.m.fighters.iter().map(|f| f.ikemen.rescued).sum();
        rescued_total += rescued;
        report.push_str(&format!(
            "seed {seed}: rescued {rescued}, longest no-control run {worst_noctrl}, longest hit-state-with-control {max_hit_ctrl}\n"
        ));
    }
    println!("{report}");
    if let Ok(out) = std::env::var("UME_EVIDENCE_DIR") {
        std::fs::write(std::path::Path::new(&out).join("fuzz.txt"), &report).unwrap();
    }
    assert_eq!(rescued_total, 0, "{report}");
}

fn native_roster() -> Vec<(String, std::sync::Arc<ume_engine::chardef::CharDef>)> {
    let mut chars: Vec<ume_engine::native::NativeChar> = std::fs::read_dir("../../assets/chars")
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
        .map(|nc| {
            (
                nc.id.clone(),
                std::sync::Arc::new(ume_engine::native::compile_native(nc, 1, true)),
            )
        })
        .collect()
}

#[test]
fn goku_against_every_native_fighter_never_gets_stuck() {
    let Some(goku) = load(1) else { return };
    let cmds: Vec<_> = goku
        .commands
        .iter()
        .filter(|c| !c.name.starts_with("ai") && !c.name.starts_with("hold"))
        .collect();
    let mut problems: Vec<String> = vec![];
    let mut rows = String::new();
    for (name, nat) in native_roster() {
        for goku_side in 0..2usize {
            let mut m = ume_engine::matchs::Match::new(
                ume_engine::common::common_def(),
                None,
                ume_engine::common::flat_stage(),
            );
            if goku_side == 0 {
                m.add_player(goku.clone(), 0, 0, 1);
                m.add_player(nat.clone(), 1, 0, 2);
            } else {
                m.add_player(nat.clone(), 0, 0, 1);
                m.add_player(goku.clone(), 1, 0, 2);
            }
            m.start_round();
            for _ in 0..900 {
                m.tick(&[0, 0]);
                if m.round_state == 2 {
                    break;
                }
            }
            for f in m.fighters.iter_mut() {
                f.life = 1.0e7;
                f.life_max = 1.0e7;
            }
            let mut ai = ume_engine::ai::Ai::new(6, 11 + goku_side as u32);
            let roots = m.root_indices();
            let mut rng = Lcg(31 + goku_side as u64 * 977);
            let mut q: std::collections::VecDeque<u32> = Default::default();
            let mut run = [0i32; 2];
            let mut worst = 0;
            let mut worst_info = String::new();
            for t in 0..9000 {
                if q.is_empty() {
                    if rng.next() % 2 == 0 {
                        q.extend(synth(cmds[rng.next() as usize % cmds.len()]));
                    } else {
                        let bits = (rng.next() as u32) & 0x3FF & !0x300;
                        for _ in 0..2 + rng.next() as usize % 8 {
                            q.push_back(bits);
                        }
                    }
                }
                if t % 600 == 0 {
                    for f in m.fighters.iter_mut() {
                        f.power = f.power_max;
                    }
                }
                if m.round_state != 2 {
                    m.round_state = 2;
                }
                let gi = q.pop_front().unwrap_or(0);
                let ni = ai.poll(&m, roots[1 - goku_side]);
                let inputs = if goku_side == 0 { [gi, ni] } else { [ni, gi] };
                m.tick(&inputs);
                for s in 0..2 {
                    let f = &m.fighters[roots[s]];
                    let stuck =
                        f.alive && f.move_type == 'H' && f.state_no != 5110 && m.pause.is_none();
                    run[s] = if stuck { run[s] + 1 } else { 0 };
                    if run[s] > worst {
                        worst = run[s];
                        worst_info = format!(
                            "side {s} state {} anim {} opp state {} gap {}",
                            f.state_no,
                            f.anim_no,
                            m.fighters[roots[1 - s]].state_no,
                            (m.fighters[roots[0]].pos.x - m.fighters[roots[1]].pos.x).abs()
                        );
                    }
                }
            }
            let rescued: u32 = m.fighters.iter().map(|f| f.ikemen.rescued).sum();
            rows.push_str(&format!("goku_side={goku_side} vs {name}: rescued {rescued}, longest hit-state run {worst} ({worst_info})\n"));
            if rescued > 0 || worst > 1500 {
                problems.push(format!("goku_side={goku_side} vs {name}: rescued {rescued}, hit-state run {worst} ({worst_info})"));
            }
        }
    }
    if let Ok(out) = std::env::var("UME_EVIDENCE_DIR") {
        std::fs::write(std::path::Path::new(&out).join("goku-vs-native.txt"), &rows).unwrap();
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
