use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k::*;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar, STANDARD_ANIMS};

const IDS: [&str; 15] = ["kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian", "garan", "marlow", "kaname", "jin", "daichi", "valdric", "haruki"];
fn load(id: &str) -> NativeChar {
    let p = format!("{}/../../assets/chars/{id}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn roster_compiles() {
    for id in IDS {
        let nc = load(id);
        for pal in 1..=3 {
            let d = compile_native(&nc, pal, false);
            for (n, no) in STANDARD_ANIMS { if nc.anims.get(n).is_some() { assert!(d.anims.contains_key(no), "{id} {n}"); } }
            assert!(d.states.len() > 10, "{id} states");
            assert!(d.commands.iter().any(|c| c.name.starts_with("simple_")), "{id} simple cmds");
        }
    }
}

#[test]
fn roster_match_smoke() {
    smoke("kaito", "shion");
}

#[test]
fn new_fighters_match_smoke() {
    for id in ["kaname", "jin", "daichi", "valdric", "haruki"] { smoke(id, "kaito"); }
}

fn smoke(p1: &str, p2: &str) {
    let a = Arc::new(compile_native(&load(p1), 1, false));
    let b = Arc::new(compile_native(&load(p2), 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a, 0, 0, 1); m.add_player(b, 1, 0, 1);
    m.start_round();
    for _ in 0..110 { m.tick(&[0, 0]); }
    for _ in 0..40 { m.tick(&[R, 0]); }
    let life = m.fighters[1].life;
    for i in 0..200 { m.tick(&[if i % 8 == 0 { X } else { 0 }, 0]); }
    assert!(m.fighters[1].life < life, "{p1}: jab spam hits");
    for _ in 0..120 { m.tick(&[0, 0]); }
    let mut seen = std::collections::HashSet::new();
    let mut proj = false;
    for b in [D, D, D | R, D | R, R, R | X, 0] { m.tick(&[b, 0]); }
    for _ in 0..60 { m.tick(&[0, 0]); seen.insert(m.fighters[0].state_no); if !m.projectiles.is_empty() { proj = true; } }
    assert!(seen.iter().any(|s| (1000..1100).contains(s)), "{p1}: special {seen:?}");
    assert!(proj, "{p1}: projectile");
    for _ in 0..120 { m.tick(&[0, 0]); }
    m.fighters[0].power = 3000.0;
    let mut sup = false;
    for _ in 0..3 { m.tick(&[D2 | W, 0]); }
    for _ in 0..60 { m.tick(&[0, 0]); if m.fighters[0].state_no >= 3000 { sup = true; } }
    assert!(sup, "{p1}: simple-mode super");
}

#[test]
fn preview_is_light_and_art_res_scales() {
    use ume_engine::native::{compile_native_preview, rig::{art_res, set_art_res}};
    let nc = load("kaito");
    let full = compile_native(&nc, 1, true);
    let pv = compile_native_preview(&nc, 1);
    assert!(pv.anims.contains_key(&0) && pv.anims.contains_key(&181), "idle + win present");
    assert!(pv.anims.len() <= 2 && pv.sprites.len() * 3 < full.sprites.len(), "preview rasterizes far fewer frames");
    // higher art resolution → denser sprites, same on-screen size (scale = 1/res)
    let base = art_res();
    let (s1, w1) = { let s = full.sprites.values().find(|s| s.image.is_some()).unwrap(); (s.scale, s.width) };
    set_art_res(6.0);
    let hi = compile_native_preview(&nc, 1);
    let s2 = hi.sprites.values().find(|s| s.image.is_some()).unwrap();
    set_art_res(base);
    assert!(s2.scale < s1 && s2.width > w1 * 0.9, "6x art is denser than the default");
}

fn team_match(simul: bool) -> Match {
    let ids = ["kaito", "shion", "akio", "suzume"];
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = simul;
    // interleaved: P1, P2, P3 (P1's partner), P4 (P2's partner)
    for (k, side) in [(0, 0), (1, 1), (2, 0), (3, 1)] { m.add_player(Arc::new(compile_native(&load(ids[k]), 1, false)), side, 0, 1); }
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0, 0, 0]); }
    m
}

#[test]
fn simul_team_round_ends_only_when_whole_team_is_down() {
    let mut m = team_match(true);
    assert_eq!(m.round_state, 2, "fight started");
    let roots = m.root_indices();
    assert_eq!(roots.len(), 4);
    // teammates line up behind their leader, same facing
    assert!(m.fighters[roots[2]].pos.x < m.fighters[roots[0]].pos.x, "P3 stands behind P1");
    assert!(m.fighters[roots[3]].pos.x > m.fighters[roots[1]].pos.x, "P4 stands behind P2");
    // one member of team 1 falls → round keeps going, the fallen can't be hit and is no longer targeted
    m.fighters[roots[1]].life = 0.0;
    for _ in 0..5 { m.tick(&[0, 0, 0, 0]); }
    assert_eq!(m.round_state, 2, "partner still standing → round continues");
    assert!(!m.fighters[roots[1]].alive);
    assert_eq!(m.enemy(roots[0], 0), Some(roots[3]), "living enemy is targeted first");
    // the partner falls too → team 0 wins
    m.fighters[roots[3]].life = 0.0;
    for _ in 0..5 { m.tick(&[0, 0, 0, 0]); }
    assert_eq!(m.round_state, 3);
    assert_eq!(m.winner_side, 0);
}

#[test]
fn single_fights_still_end_on_first_ko() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("shion"), 1, false)), 1, 0, 1);
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    m.fighters[0].life = 0.0;
    for _ in 0..5 { m.tick(&[0, 0]); }
    assert_eq!((m.round_state, m.winner_side), (3, 1));
}

#[test]
fn five_vs_five_simul_runs_and_ends_only_when_whole_team_falls() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = true;
    let chars: Vec<Arc<_>> = IDS.iter().take(10).map(|id| Arc::new(compile_native(&load(id), 1, false))).collect();
    // interleaved like the app: P1, P2, P3, ... so roots()[0..2] are the two leaders
    for k in 0..5 { for side in 0..2 { m.add_player(chars[k * 2 + side].clone(), side, 0, 1); } }
    m.start_round();
    let (bl, br) = (m.stage.bound_left, m.stage.bound_right);
    for f in m.fighters.iter() { assert!(f.pos.x >= bl && f.pos.x <= br, "everyone starts inside the stage bounds ({})", f.pos.x); }
    for _ in 0..140 { m.tick(&[0; 10]); }
    assert_eq!(m.round_state, 2);
    assert_eq!(m.root_indices().len(), 10);
    // 10 fighters brawling (everybody mashes): must stay far faster than real time (600 ticks = 10 s of game)
    let t0 = std::time::Instant::now();
    for i in 0..600 { let b: Vec<u32> = (0..10).map(|k| if (i + k * 3) % 7 < 3 { R | if (i / 5 + k) % 2 == 0 { X } else { A } } else { 0 }).collect(); m.tick(&b); }
    let el = t0.elapsed().as_secs_f32();
    eprintln!("5v5: 600 ticks in {el:.3}s");
    assert!(el < 5.0, "10-fighter simulation must run well above real time (took {el}s for 10 s of game)");
    for f in m.fighters.iter_mut().filter(|f| !f.is_helper) { f.life = f.life_max; f.alive = true; }
    // knock out four of five on side 1: the round must continue
    let roots = m.root_indices();
    let side1: Vec<usize> = roots.iter().copied().filter(|&i| m.fighters[i].side == 1).collect();
    assert_eq!(side1.len(), 5);
    for &i in side1.iter().take(4) { m.fighters[i].life = 0.0; }
    for _ in 0..5 { m.tick(&[0; 10]); }
    assert_eq!(m.round_state, 2, "one defender left → still fighting");
    assert_eq!(m.enemy(roots[0], 0), Some(side1[4]), "the survivor is targeted first");
    m.fighters[side1[4]].life = 0.0;
    for _ in 0..5 { m.tick(&[0; 10]); }
    assert_eq!((m.round_state, m.winner_side), (3, 0));
}

#[test]
fn deciding_round_is_announced_as_final() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("shion"), 1, false)), 1, 0, 1);
    m.start_round();
    assert_eq!(m.announce.as_ref().unwrap().text, "ROUND 1");
    m.wins = [1, 1]; m.round_no = 3; m.start_round();
    assert_eq!(m.announce.as_ref().unwrap().text, "FINAL ROUND");
    // single-round bouts (team sequence) never say "final"
    m.rounds_to_win = 1; m.wins = [0, 0]; m.start_round();
    assert!(m.announce.as_ref().unwrap().text.starts_with("ROUND"));
}

/// Hash of everything gameplay-relevant, to compare two simulations tick by tick.
fn state_hash(m: &Match) -> u64 {
    let mut h: u64 = 1469598103934665603;
    let mut mix = |v: u64| { h ^= v; h = h.wrapping_mul(1099511628211); };
    for f in &m.fighters {
        mix(f.id as u64); mix(f.life.to_bits() as u64); mix(f.power.to_bits() as u64); mix(f.pos.x.to_bits() as u64); mix(f.pos.y.to_bits() as u64);
        mix(f.state_no as u64); mix(f.state_time as u64); mix(f.anim_no as u64); mix(f.hit_pause as u64);
    }
    mix(m.projectiles.len() as u64); mix(m.explods.len() as u64); mix(m.round_state as u64); mix(m.timer as u64);
    h
}

fn scripted(i: usize, k: usize) -> u32 { // deterministic "button masher" for fighter k
    let x = (i.wrapping_mul(2654435761).wrapping_add(k * 40503) >> 7) as u32;
    let dirs = [0, R, L, D, D | R, U];
    let btn = [0, X, Y, A, B, X | Y, D2, W, 0, 0];
    dirs[(x % 6) as usize] | btn[((x >> 3) % 10) as usize]
}

/// Replays and rollback netcode both need a bit-for-bit deterministic simulation: the same inputs must give the same state.
#[test]
fn simulation_is_deterministic_for_replays() {
    for (ids, simul) in [(&["kaito", "shion"][..], false), (&["kaito", "shion", "akio", "suzume"][..], true)] {
        let build = || {
            let mut m = Match::new(common_def(), None, flat_stage());
            m.team_simul = simul;
            for (k, id) in ids.iter().enumerate() { m.add_player(Arc::new(compile_native(&load(id), 1, false)), k % 2, 0, 1); }
            m.start_round();
            m
        };
        let (mut a, mut rec) = (build(), vec![]);
        let n = ids.len();
        let mut hashes = vec![];
        for i in 0..900 {
            let bits: Vec<u32> = (0..n).map(|k| scripted(i, k)).collect();
            a.tick(&bits);
            rec.push(bits);
            if i % 100 == 99 { hashes.push(state_hash(&a)); }
        }
        let mut b = build();
        for (i, bits) in rec.iter().enumerate() {
            b.tick(bits);
            if i % 100 == 99 { assert_eq!(state_hash(&b), hashes[i / 100], "simul={simul}: replay diverged at tick {i}"); }
        }
        assert!(a.fighters.iter().any(|f| f.life < f.life_max), "the scripted fight actually exchanged hits (simul={simul})");
    }
}

#[test]
fn combo_damage_scales_down_and_never_vanishes() {
    use ume_engine::matchs::{combo_scale, corner_push};
    let v: Vec<f32> = (0..14).map(combo_scale).collect();
    assert_eq!((v[0], v[1]), (1.0, 1.0), "the first two hits are full damage");
    assert!(v.windows(2).all(|w| w[1] <= w[0]), "monotonically non-increasing: {v:?}");
    assert!(v.iter().all(|&x| x >= 0.2) && (v[13] - 0.25).abs() < 1e-6, "floor holds");
    assert!(corner_push(0.0) >= 1.2 && corner_push(-30.0) <= 4.5, "push is bounded");
}

/// A native fighter's long combo must deal less than the same hits would at full value.
#[test]
fn native_combos_are_prorated_and_corners_push_attackers_back() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("shion"), 1, false)), 1, 0, 1);
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    // corner the defender and let the attacker mash light attacks next to him
    let edge = m.stage.bound_right + m.view_w / 2.0 - 40.0;
    m.fighters[1].pos.x = edge;
    m.fighters[0].pos.x = edge - 38.0;
    let before = m.fighters[1].life;
    let mut max_combo = 0;
    for i in 0..900 {
        m.tick(&[if i % 6 == 0 { R | X } else if i % 6 == 3 { A } else { R }, 0]);
        max_combo = max_combo.max(m.fighters[1].ghv.hitcount);
        if m.fighters[1].life <= 0.0 { break; }
    }
    assert!(m.fighters[1].life < before, "damage was dealt");
    assert!(m.corner_pushes > 0, "the cornered defender pushed the attacker back at least once");
    let _ = max_combo;
}
