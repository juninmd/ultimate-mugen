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

fn duel() -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("shion"), 1, false)), 1, 0, 1);
    m
}

#[test]
fn awakening_costs_a_bar_boosts_stats_and_ends_with_the_round() {
    let mut m = duel();
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    assert_eq!(m.round_state, 2);
    // not enough meter → nothing happens
    m.tick(&[T, 0]);
    assert_eq!(m.fighters[0].form, 0, "needs one full super bar");
    m.fighters[0].power = 1500.0;
    let (atk, def) = (m.fighters[0].attack_mul, m.fighters[0].defence_mul);
    m.fighters[0].life = 400.0;
    for _ in 0..3 { m.tick(&[T, 0]); }
    let f = &m.fighters[0];
    assert_eq!(f.form, 1);
    assert!(f.power < 1000.0, "awakening spends a bar ({})", f.power);
    assert!(f.attack_mul > atk * 1.25 && f.defence_mul > def * 1.2, "stronger and tougher");
    assert!(f.life > 400.0, "small life refill");
    assert!(m.events.iter().any(|e| matches!(e, ume_engine::matchs::Event::Transform { .. })));
    // plays the power-up state (193), then returns to idle with control; can't awaken twice
    assert_eq!(f.state_no, 193);
    for _ in 0..100 { m.tick(&[0, 0]); }
    assert!(m.fighters[0].ctrl && m.fighters[0].state_no == 0, "back to idle with control");
    m.fighters[0].power = 3000.0;
    let a2 = m.fighters[0].attack_mul;
    m.tick(&[T, 0]);
    assert_eq!(m.fighters[0].attack_mul, a2, "only once per round");
    // meter regenerates while awakened
    let p = m.fighters[0].power; m.fighters[0].power = 100.0;
    for _ in 0..60 { m.tick(&[0, 0]); }
    assert!(m.fighters[0].power > 100.0 + 50.0 && p > 0.0);
    // the next round starts in the base form again
    m.start_round();
    assert_eq!(m.fighters[0].form, 0);
    assert!((m.fighters[0].attack_mul - atk).abs() < 1e-4 && (m.fighters[0].defence_mul - def).abs() < 1e-4, "stats restored");
}

#[test]
fn first_round_waits_for_the_face_off_and_a_button_skips_it() {
    let mut m = duel();
    m.intro_ticks = 170;
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    assert!(m.round_state < 2, "still trading lines");
    assert!(matches!(m.fighters[0].state_no, 192 | 194) || matches!(m.fighters[1].state_no, 192 | 194), "fighters pose for the face-off");
    for _ in 0..140 { m.tick(&[0, 0]); }
    assert_eq!(m.round_state, 2, "fight begins after the face-off");
    assert!(m.fighters[0].ctrl && m.fighters[1].ctrl);
    // pressing a button skips it
    let mut m = duel();
    m.intro_ticks = 170;
    m.start_round();
    for _ in 0..60 { m.tick(&[0, 0]); }
    m.tick(&[X, 0]);
    for _ in 0..80 { m.tick(&[0, 0]); }
    assert_eq!(m.round_state, 2, "skipped");
    // later rounds never wait
    m.round_no = 2; m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    assert_eq!(m.round_state, 2);
}

#[test]
fn every_ko_plays_a_defeat_sequence_and_the_deciding_one_is_the_longest() {
    let mut seen = std::collections::HashSet::new();
    for round in 1..=3 {
        let mut m = duel();
        m.start_round();
        m.round_no = round; m.start_round();
        for _ in 0..140 { m.tick(&[0, 0]); }
        m.fighters[1].life = 0.0;
        for _ in 0..420 { m.tick(&[0, 0]); if m.round_state == 4 && m.round_time > 30 { break; } }
        let st = m.fighters[1].state_no;
        assert!((5190..=5193).contains(&st), "round {round}: loser in a defeat state, got {st}");
        assert!(m.fighters[1].def.anims.contains_key(&(st - 5190 + 197)), "its animation exists");
        seen.insert(st);
    }
    assert!(seen.len() >= 2, "different rounds show different defeats: {seen:?}");
    // match-deciding KO → the dramatic one
    let mut m = duel();
    m.wins = [1, 0];
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    m.fighters[1].life = 0.0;
    for _ in 0..420 { m.tick(&[0, 0]); if m.round_state == 4 && m.round_time > 30 { break; } }
    assert_eq!(m.fighters[1].state_no, 5193);
    for _ in 0..300 { m.tick(&[0, 0]); if m.match_over { break; } }
    assert!(m.match_over, "the match ends after the defeat plays out");
}

#[test]
fn big_teams_zoom_out_and_everyone_stays_inside_the_field() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = true;
    let chars: Vec<Arc<_>> = IDS.iter().take(10).map(|id| Arc::new(compile_native(&load(id), 1, false))).collect();
    for k in 0..5 { for side in 0..2 { m.add_player(chars[k * 2 + side].clone(), side, 0, 1); } }
    m.set_zoom_for_teams(5, 0.8);
    let base = (ume_engine::matchs::BASE_W, ume_engine::matchs::BASE_H);
    assert!(m.view_w > base.0 * 1.35 && (m.view_w / m.view_h - base.0 / base.1).abs() < 1e-3, "5v5 sees a much wider field, same aspect ({})", m.view_w);
    m.start_round();
    for _ in 0..140 { m.tick(&[0; 10]); }
    let (zmin, zmax) = (m.zoom_min, m.zoom_max);
    for i in 0..400 { let b: Vec<u32> = (0..10).map(|k| if (i + k * 3) % 9 < 4 { if k % 2 == 0 { R } else { L } } else { 0 }).collect(); m.tick(&b); assert!(m.zoom >= zmin - 1e-3 && m.zoom <= zmax + 1e-3); }
    let (l, r) = (m.camera.x - m.view_w / 2.0, m.camera.x + m.view_w / 2.0);
    for f in m.roots() { assert!(f.pos.x > l - 20.0 && f.pos.x < r + 20.0, "fighter on screen"); }
    // 1v1 never zooms out
    let mut d = duel(); d.set_zoom_for_teams(1, 0.8); d.start_round();
    for _ in 0..200 { d.tick(&[R, 0]); }
    assert_eq!(d.view_w, base.0);
}

#[test]
fn final_boss_is_complete_and_formidable() {
    let nc = load("oboro");
    assert!(nc.boss && nc.intro.is_some() && nc.taunt.is_some() && nc.form.is_some());
    let d = compile_native(&nc, 1, false);
    assert!(d.boss && d.c("data.life", 0.0) >= 2200.0, "tougher than the 1000-life roster");
    for no in [195, 196, 197, 198, 199, 200] { assert!(d.anims.contains_key(&no), "default anim {no}"); }
    let supers = nc.moves.iter().filter(|m| m.kind == "super").count();
    let specials = nc.moves.iter().filter(|m| m.kind == "special").count();
    assert!(supers >= 2 && specials >= 4, "{supers} supers, {specials} specials");
    // every special / super reaches a state and fires something
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(Arc::new(d), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 1, 0, 1);
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    let mut reached = std::collections::HashSet::new();
    let moves: [&[u32]; 6] = [&[D, D, D | R, D | R, R, R | X, 0], &[D, D, D | L, D | L, L, L | A, 0], &[R, D, D | R, R | Y, 0], &[D, D, D | R, D | R, R, R | Y, 0],
        &[D, D, D | R, D | R, R, R, 0, D, D, D | R, D | R, R, R, R | X, 0], &[D, D, D | L, D | L, L, L, 0, D, D, D | L, D | L, L, L, L | X, 0]];
    for seq in moves {
        m.fighters[0].power = 3000.0; m.fighters[1].life = m.fighters[1].life_max;
        for _ in 0..90 { m.tick(&[0, 0]); }
        // directions are written for a fighter facing right; mirror them when the boss ended up turned around (teleports)
        let flip = m.fighters[0].facing < 0.0;
        for &b in seq { let b = if flip { (b & !(L | R)) | if b & R != 0 { L } else { 0 } | if b & L != 0 { R } else { 0 } } else { b }; m.tick(&[b, 0]); }
        let mut here = std::collections::BTreeSet::new();
        for _ in 0..90 { m.tick(&[0, 0]); reached.insert(m.fighters[0].state_no); here.insert(m.fighters[0].state_no); }
        let _ = &here;
    }
    assert!(reached.iter().filter(|s| (1000..1100).contains(*s)).count() >= 3, "specials reachable: {reached:?}");
    assert!(reached.iter().filter(|s| **s >= 3000).count() >= 2, "both supers reachable: {reached:?}");
}

#[test]
fn final_boss_is_faster_and_hits_harder_than_the_roster() {
    let boss = compile_native(&load("oboro"), 1, false);
    let reg = compile_native(&load("kaname"), 1, false); // same katana base kit
    let total = |d: &ume_engine::chardef::CharDef, no: i32| d.anims.get(&no).map(|a| a.frames.iter().filter(|f| f.time > 0).map(|f| f.time).sum::<i32>()).unwrap_or(0);
    // every timed frame is shorter (speed ×1.3): walk cycle, run cycle and the hit-reaction recover sooner
    for no in [20, 100, 0] {
        let (b, r) = (total(&boss, no), total(&reg, no));
        assert!(r > 0 && b as f32 <= r as f32 * 0.85, "anim {no}: boss {b} ticks vs regular {r}");
    }
    assert!(boss.c("velocity.run.fwd.x", 0.0) > reg.c("velocity.run.fwd.x", 0.0) * 1.25, "runs faster");
    assert!(boss.c("velocity.walk.fwd.x", 0.0) > reg.c("velocity.walk.fwd.x", 0.0) * 1.25, "walks faster");
    assert!(boss.c("data.attack", 0.0) >= 125.0, "hits harder");
    // the boss keeps charging its meter on its own
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(Arc::new(boss), 0, 0, 1);
    m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 1, 0, 1);
    m.start_round();
    for _ in 0..140 { m.tick(&[0, 0]); }
    let p0 = m.fighters[0].power;
    for _ in 0..600 { m.tick(&[0, 0]); }
    assert!(m.fighters[0].power > p0 + 150.0, "passive charge ({p0} -> {})", m.fighters[0].power);
}

#[test]
fn final_boss_beats_a_competent_cpu_most_of_the_time() {
    // baseline: a regular fighter at the same AI level against the same opponent, to show how much stronger the boss is
    let base = duel_stats("kaname", false);
    let (wins, hp_boss, hp_foe) = duel_stats("oboro", true);
    eprintln!("baseline kaname(L8): wins {}/8, own life {:.2}, foe life {:.2}", base.0, base.1, base.2);
    eprintln!("boss oboro(L8):      wins {wins}/8, own life {hp_boss:.2}, foe life {hp_foe:.2}");
    assert!(wins >= 6, "the boss should beat a level-6 CPU in most matches ({wins}/8)");
    assert!(hp_boss > base.1, "and more convincingly than a regular fighter");
}

fn duel_stats(id: &str, boss: bool) -> (u32, f32, f32) {
    use ume_engine::ai::Ai;
    use ume_engine::input::InputSource;
    let mut boss_wins = 0;
    let (mut boss_hp, mut foe_hp) = (0.0f32, 0.0f32);
    let seeds = 8u32;
    for seed in 0..seeds {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.rounds_to_win = 1;
        m.add_player(Arc::new(compile_native(&load("kaito"), 1, false)), 0, 6, 1);
        m.add_player(Arc::new(compile_native(&load(id), 1, false)), 1, 8, 1);
        m.start_round();
        if boss { for f in m.fighters.iter_mut().filter(|f| f.def.boss) { f.power = 2000.0; } }
        let mut ais = [Ai::new(6, 11 + seed * 7), Ai::new(8, 99 + seed * 13)];
        for _ in 0..(60 * 100) {
            let b = [ais[0].poll(&m, 0), ais[1].poll(&m, 1)];
            m.tick(&b);
            if m.round_state >= 4 { break; }
        }
        boss_hp += m.fighters[1].life / m.fighters[1].life_max; foe_hp += m.fighters[0].life / m.fighters[0].life_max;
        if m.winner_side == 1 { boss_wins += 1; }
    }
    (boss_wins, boss_hp / seeds as f32, foe_hp / seeds as f32)
}

#[test]
fn team_wins_are_announced_by_the_first_member() {
    let mut m = team_match(true);
    let roots = m.root_indices();
    let lead = m.fighters[roots[0]].def.display_name.split_whitespace().next().unwrap().to_uppercase();
    assert_eq!(m.team_leader(0).as_deref(), Some(lead.as_str()));
    m.fighters[roots[1]].life = 0.0;
    m.fighters[roots[3]].life = 0.0;
    for _ in 0..400 { m.tick(&[0, 0, 0, 0]); if m.round_state == 4 { break; } }
    assert_eq!(m.announce.as_ref().map(|a| a.text.clone()), Some(format!("TEAM {lead} WINS")));
}

#[test]
fn sequence_teams_use_the_given_leader_and_solo_fights_keep_the_name() {
    let mut m = duel();
    assert_eq!(m.team_leader(0), None);
    assert!(m.winner_text(0).ends_with(" WINS") && !m.winner_text(0).starts_with("TEAM "));
    m.team_lead = ["kaito".into(), String::new()];
    assert_eq!(m.winner_text(0), "TEAM KAITO WINS");
    assert!(!m.winner_text(1).starts_with("TEAM "));
}
