//! Runtime check for the UMF 2.0 timestop and armor traits.
mod support;
use support::*;
use ume_native::{compile_native, ArmorSpec, NativeChar, NativeMove};

fn base_char() -> NativeChar {
    let raw = include_str!("../../../assets/chars/kaito.json");
    serde_json::from_str(raw).expect("valid kaito template")
}

#[test]
fn test_trait_runtime_match_simulation() {
    let mut nc = base_char();
    nc.abilities = None;
    nc.id = "trait_sim".into();
    nc.moves.clear();
    nc.moves.push(NativeMove {
        id: "za_warudo".into(),
        name: "Time Stop".into(),
        input: "D, DB, B, y".into(),
        simple: None,
        kind: "special".into(),
        when: Some("stand".into()),
        anim: "idle".into(),
        hits: vec![],
        moves: vec![],
        projectile: None,
        cost: None,
        invuln: None,
        air_physics: None,
        note: None,
        teleport: None,
        fx: None,
        voice: None,
        sounds: vec![],
        armor: Some(ArmorSpec { hits: 1, threshold: None }),
        parry: None,
        burst: None,
        flight: None,
        timestop: Some(90),
        throw: None,
        cancels: None,
        rekka: None,
        blink: None, vfx: vec![], quake: vec![],
        requires: None,
    });

    let def = std::sync::Arc::new(compile_native(&nc, 1, false));
    let mut m = duel(def, mugen_char("dummy", ""));
    enter(&mut m, 0, 1000);
    ticks(&mut m, 2);

    assert!(m.fighters[0].hit_overrides[0].is_some(), "Runtime HitOverride active");
    assert!(m.pause.is_some(), "Runtime Pause active during Time Stop");
}
