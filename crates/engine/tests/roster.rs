use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k::*;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar, STANDARD_ANIMS};

const IDS: [&str; 10] = ["kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian", "garan", "marlow"];
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
    let a = Arc::new(compile_native(&load("kaito"), 1, false));
    let b = Arc::new(compile_native(&load("shion"), 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a, 0, 0, 1); m.add_player(b, 1, 0, 1);
    m.start_round();
    for _ in 0..110 { m.tick(&[0, 0]); }
    for _ in 0..40 { m.tick(&[R, 0]); }
    let life = m.fighters[1].life;
    for i in 0..200 { m.tick(&[if i % 8 == 0 { X } else { 0 }, 0]); }
    assert!(m.fighters[1].life < life, "jab spam hits");
    for _ in 0..120 { m.tick(&[0, 0]); }
    let mut seen = std::collections::HashSet::new();
    let mut proj = false;
    for b in [D, D, D | R, D | R, R, R | X, 0] { m.tick(&[b, 0]); }
    for _ in 0..60 { m.tick(&[0, 0]); seen.insert(m.fighters[0].state_no); if !m.projectiles.is_empty() { proj = true; } }
    assert!(seen.iter().any(|s| (1000..1100).contains(s)), "special {seen:?}");
    assert!(proj, "projectile");
    for _ in 0..120 { m.tick(&[0, 0]); }
    m.fighters[0].power = 3000.0;
    let mut sup = false;
    for _ in 0..3 { m.tick(&[D2 | W, 0]); }
    for _ in 0..60 { m.tick(&[0, 0]); if m.fighters[0].state_no >= 3000 { sup = true; } }
    assert!(sup, "simple-mode super");
}
