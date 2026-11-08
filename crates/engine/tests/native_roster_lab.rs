mod support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const GAPS: [f32; 14] = [14.0, 24.0, 34.0, 44.0, 54.0, 64.0, 76.0, 90.0, 110.0, 135.0, 165.0, 200.0, 250.0, 320.0];
const ATTACK_RANGES: [(i32, i32); 4] = [(200, 299), (400, 499), (600, 699), (1000, 3799)];

struct Probe { state: i32, anim: i32, damage: f32, reach: f32, hits: usize, ticks_to_ctrl: i32 }

fn characters() -> Vec<NativeChar> {
    let mut out: Vec<NativeChar> = vec![];
    let dir = std::env::var("UME_CHARS_DIR").unwrap_or_else(|_| "../../assets/chars".into());
    let only: Vec<String> = std::env::var("UME_LAB_ONLY").map(|v| v.split(',').map(str::to_string).collect()).unwrap_or_default();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "json") && path.file_stem().is_some_and(|s| s != "index" && (only.is_empty() || only.iter().any(|o| s.to_string_lossy() == o.as_str()))) {
            out.push(serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap());
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn arena(def: &Arc<ume_engine::chardef::CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    m
}

fn run(base: &Match, def: &Arc<ume_engine::chardef::CharDef>, state: i32) -> Probe {
    let air = (600..700).contains(&state) || def.states.get(&state).and_then(|d| d.param("type")).is_some_and(|t| t.raw.eq_ignore_ascii_case("A")) && state < 3800;
    let mut probe = Probe { state, anim: 0, damage: 0.0, reach: 0.0, hits: 0, ticks_to_ctrl: -1 };
    for gap in GAPS {
        let mut m = base.clone();
        face_off(&mut m, gap);
        m.fighters[0].power = 3000.0;
        if air { m.fighters[0].pos.y = if state < 700 { -40.0 } else { -90.0 }; }
        let start = m.fighters[1].life;
        enter(&mut m, 0, state);
        probe.anim = m.fighters[0].anim_no;
        let mut last = start;
        let mut ctrl_at = -1;
        for t in 0..240 {
            m.tick(&[0, 0]);
            for f in &m.fighters { assert!(f.pos.x.is_finite() && f.pos.y.is_finite() && f.vel.x.is_finite() && f.vel.y.is_finite(), "state {state} gap {gap}: non finite"); }
            if m.fighters[1].life < last { probe.hits += 1; last = m.fighters[1].life; }
            if ctrl_at < 0 && t > 3 && m.fighters[0].ctrl { ctrl_at = t; }
        }
        let dealt = start - m.fighters[1].life;
        if dealt > 0.0 { probe.reach = probe.reach.max(gap); if dealt > probe.damage { probe.damage = dealt; } if probe.ticks_to_ctrl < 0 { probe.ticks_to_ctrl = ctrl_at; } }
    }
    probe
}

fn lab(nc: &NativeChar) -> Vec<Probe> {
    let def = Arc::new(compile_native(nc, 1, true));
    let base = arena(&def);
    let mut states: Vec<i32> = def.states.keys().copied().filter(|s| ATTACK_RANGES.iter().any(|(a, b)| (a..=b).contains(&s))).collect();
    states.sort();
    states.into_iter().map(|s| run(&base, &def, s)).collect()
}

#[test]
fn every_native_attack_state_connects_and_stays_finite() {
    let mut failures = vec![];
    for nc in characters() {
        let probes = lab(&nc);
        println!("== {} ({} attack states)", nc.id, probes.len());
        for p in &probes {
            println!("  state {:5} anim {:5} damage {:6.1} hits {:3} reach {:4.0} ctrl@{}", p.state, p.anim, p.damage, p.hits, p.reach, p.ticks_to_ctrl);
            if p.damage <= 0.0 { failures.push(format!("{} state {} never connected (anim {})", nc.id, p.state, p.anim)); }
        }
    }
    assert!(failures.is_empty(), "dead moves:\n{}", failures.join("\n"));
}
