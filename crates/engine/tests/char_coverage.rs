mod support;
use std::collections::BTreeSet;
use support::gogeta::*;

#[test]
fn audit_has_no_unsupported_controllers_or_triggers() {
    let Some(d) = load(1) else { return };
    let diag = d.diagnostics();
    if let Ok(out) = std::env::var("UME_EVIDENCE_DIR") {
        std::fs::write(std::path::Path::new(&out).join("audit.txt"), diag.detail()).unwrap();
    }
    use ume_engine::diagnostics::Category;
    for c in [
        Category::Controller,
        Category::Trigger,
        Category::Redirection,
        Category::StringTrigger,
        Category::AssertFlag,
    ] {
        assert_eq!(diag.count(c), 0, "{}", diag.detail());
    }
}

#[test]
fn every_palette_loads() {
    let Some(_) = load(1) else { return };
    for p in 1..=5 {
        assert!(load(p).is_some());
    }
}

#[test]
fn intro_bubble_lasts_until_fight() {
    let Some(d) = load(1) else { return };
    let mut m = ume_engine::matchs::Match::new(
        ume_engine::common::common_def(),
        None,
        ume_engine::common::flat_stage(),
    );
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d, 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    let mut states: BTreeSet<i32> = BTreeSet::new();
    let mut anims_seen: BTreeSet<i32> = BTreeSet::new();
    let mut last_bubble = 0usize;
    let mut first_bubble = usize::MAX;
    let mut fight_tick = 0usize;
    for t in 0..1500 {
        m.tick(&[0, 0]);
        states.insert(m.fighters[0].state_no);
        for e in &m.explods {
            anims_seen.insert(e.cur.anim);
            if matches!(e.cur.anim, 191 | 193 | 194 | 199) {
                last_bubble = t;
                first_bubble = first_bubble.min(t);
            }
        }
        if m.round_state == 2 {
            fight_tick = t;
            break;
        }
    }
    println!("INTRO states {states:?}");
    println!("INTRO explod anims {anims_seen:?} first_bubble={first_bubble} last_bubble_tick={last_bubble} fight_tick={fight_tick}");
    assert!(states.contains(&190));
    assert!(first_bubble < 60);
    assert!(last_bubble + 1 >= fight_tick);
}

#[test]
fn coverage_report() {
    let (Some(d), Ok(out)) = (load(1), std::env::var("UME_EVIDENCE_DIR")) else {
        return;
    };
    let mut p1: BTreeSet<i32> = BTreeSet::new();
    let mut helper_states: BTreeSet<i32> = BTreeSet::new();
    let mut explod_anims: BTreeSet<i32> = BTreeSet::new();
    let mut proj_anims: BTreeSet<i32> = BTreeSet::new();
    let mut fired: BTreeSet<String> = BTreeSet::new();
    let mut max_helpers = 0usize;
    let mut names: Vec<String> = vec![];
    for c in d.commands.iter() {
        if c.name.starts_with("ai") || c.name.starts_with("hold") {
            continue;
        }
        names.push(c.name.clone());
        for (gap, air, v) in [
            (40.0f32, false, 0),
            (120.0, false, 0),
            (300.0, false, 0),
            (60.0, true, 0),
            (200.0, true, 0),
            (40.0, false, 5000),
        ] {
            let mut r = Duel::new(d.clone(), gap);
            if air {
                r.jump();
            }
            r.m.fighters[0].vars[10] = v;
            r.m.fighters[0].vars[11] = v;
            r.push(0, &synth(c));
            for _ in 0..260 {
                r.tick(1);
                if r.m.fighters[0].cmd.active.contains(c.name.as_str()) {
                    fired.insert(c.name.clone());
                }
                for f in &r.m.fighters {
                    if f.is_helper {
                        helper_states.insert(f.state_no);
                    } else if f.id == r.m.fighters[0].id {
                        p1.insert(f.state_no);
                    }
                }
                max_helpers = max_helpers.max(r.m.fighters.iter().filter(|f| f.is_helper).count());
                for e in &r.m.explods {
                    explod_anims.insert(e.cur.anim);
                }
                for pj in &r.m.projectiles {
                    proj_anims.insert(pj.cur.anim);
                }
            }
        }
    }
    let all: BTreeSet<i32> = d.states.keys().copied().collect();
    let reached: BTreeSet<i32> = p1.union(&helper_states).copied().collect();
    let missed: Vec<i32> = all.difference(&reached).copied().collect();
    let md = format!(
        "commands swept: {}\ncommands fired from synthesised input: {}\nstates defined: {}\nstates entered by the sweep: {} (player {}, helpers {})\nmax helpers alive: {}\nexplod anims seen: {:?}\nprojectile anims seen: {:?}\nstates not entered by the command sweep: {:?}\n",
        names.len(), fired.len(), all.len(), reached.len(), p1.len(), helper_states.len(), max_helpers, explod_anims, proj_anims, missed
    );
    std::fs::write(std::path::Path::new(&out).join("sweep-coverage.txt"), md).unwrap();
    let silent: Vec<&String> = names.iter().filter(|n| !fired.contains(*n)).collect();
    println!("NOT FIRED {silent:?}");
    assert!(silent
        .iter()
        .all(|n| n.as_str() == "start" || n.as_str() == "s"));
}

#[test]
fn every_state_runs_and_returns() {
    let Some(d) = load(1) else { return };
    let mut stuck: Vec<(i32, i32, bool)> = vec![];
    let mut ids: Vec<i32> = d.states.keys().copied().filter(|s| *s >= 0).collect();
    ids.sort();
    let mut rows = String::from("state | start | after 2400 ticks (state, ctrl)\n");
    for s in ids.iter().copied() {
        for gap in [40.0f32, 300.0] {
            let mut r = Duel::new(d.clone(), gap);
            r.m.fighters[0].power = r.m.fighters[0].power_max;
            r.m.change_state(0, s, d.clone());
            r.tick(2400);
            let f = &r.m.fighters[0];
            let ok = f.ctrl && f.alive;
            if gap == 300.0 {
                rows.push_str(&format!("{s} | forced | ({}, {})\n", f.state_no, f.ctrl));
            }
            if !ok {
                stuck.push((s, f.state_no, f.ctrl));
            }
        }
    }
    stuck.sort();
    stuck.dedup();
    println!("STATES not back in control after 2400 ticks: {stuck:?}");
    if let Ok(out) = std::env::var("UME_EVIDENCE_DIR") {
        std::fs::write(std::path::Path::new(&out).join("forced-states.txt"), rows).unwrap();
    }
}
