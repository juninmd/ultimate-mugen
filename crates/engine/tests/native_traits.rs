//! Homologation tests for UMF 2.0 declarative traits: armor, parry, timestop, burst, flight.
mod support;
use support::*;
use ume_engine::native::{compile_native, NativeChar, NativeMove, ArmorSpec, ParrySpec};

fn base_char() -> NativeChar {
    let raw = include_str!("../../../assets/chars/kaito.json");
    serde_json::from_str(raw).expect("valid kaito template")
}

#[test]
fn test_trait_compilation_and_cns_generation() {
    let mut nc = base_char();
    nc.id = "trait_tester".into();
    nc.moves.clear();

    // 1. Armor trait
    nc.moves.push(NativeMove {
        id: "armored_strike".into(),
        name: "Armored Strike".into(),
        input: "D, DF, F, x".into(),
        simple: None,
        kind: "special".into(),
        when: Some("stand".into()),
        anim: "special".into(),
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
        armor: Some(ArmorSpec { hits: 1, threshold: Some(50.0) }),
        parry: None,
        burst: None,
        flight: None,
        timestop: None,
        throw: None,
        cancels: None,
    });

    // 2. Parry trait
    nc.moves.push(NativeMove {
        id: "just_parry".into(),
        name: "Just Parry".into(),
        input: "F+x".into(),
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
        armor: None,
        parry: Some(ParrySpec { window: [1, 8], spark: Some(5) }),
        burst: None,
        flight: None,
        timestop: None,
        throw: None,
        cancels: None,
    });

    // 3. TimeStop trait
    nc.moves.push(NativeMove {
        id: "za_warudo".into(),
        name: "Time Stop".into(),
        input: "D, DB, B, y".into(),
        simple: None,
        kind: "super".into(),
        when: Some("stand".into()),
        anim: "special".into(),
        hits: vec![],
        moves: vec![],
        projectile: None,
        cost: Some(1000.0),
        invuln: None,
        air_physics: None,
        note: None,
        teleport: None,
        fx: None,
        voice: None,
        armor: None,
        parry: None,
        burst: None,
        flight: None,
        timestop: Some(120),
        throw: None,
        cancels: None,
    });

    // 4. Burst trait
    nc.moves.push(NativeMove {
        id: "combo_break".into(),
        name: "Defensive Burst".into(),
        input: "x+y+z".into(),
        simple: None,
        kind: "special".into(),
        when: Some("stand".into()),
        anim: "idle".into(),
        hits: vec![],
        moves: vec![],
        projectile: None,
        cost: Some(1000.0),
        invuln: None,
        air_physics: None,
        note: None,
        teleport: None,
        fx: None,
        voice: None,
        armor: None,
        parry: None,
        burst: Some(true),
        flight: None,
        timestop: None,
        throw: None,
        cancels: None,
    });

    // 5. Flight trait
    nc.moves.push(NativeMove {
        id: "air_levitation".into(),
        name: "Sky Flight".into(),
        input: "D, U, z".into(),
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
        armor: None,
        parry: None,
        burst: None,
        flight: Some(true),
        timestop: None,
        throw: None,
        cancels: None,
    });

    let def = compile_native(&nc, 1, false);

    // Verify Armor controller
    let mut found_armor = false;
    let mut found_parry = false;
    let mut found_pause = false;
    let mut found_burst_inv = false;
    let mut found_flight_hover = false;

    for (_, state) in &def.states {
        for sc in &state.controllers {
            if sc.kind == "hitoverride" { found_armor = true; }
            if sc.kind == "reversaldef" { found_parry = true; }
            if sc.kind == "pause" { found_pause = true; }
            if sc.kind == "nothitby" && sc.raw("value") == Some("SCA") { found_burst_inv = true; }
            if sc.kind == "velset" && sc.raw("x").is_some_and(|x| x.contains("holdfwd")) { found_flight_hover = true; }
        }
    }

    assert!(found_armor, "Armor trait emitted HitOverride controller");
    assert!(found_parry, "Parry trait emitted ReversalDef controller");
    assert!(found_pause, "TimeStop trait emitted Pause controller");
    assert!(found_burst_inv, "Burst trait emitted NotHitBy invulnerability");
    assert!(found_flight_hover, "Flight trait emitted 8-way hover controller");

    // Verify -1 burst trigger allows activation during hitstun
    let burst_state = def.states.get(&-1).expect("statedef -1 exists");
    let has_hitstun_escape = burst_state.controllers.iter().any(|sc| {
        sc.label.contains("Defensive Burst") && sc.triggers.len() >= 2
    });
    assert!(has_hitstun_escape, "Burst trait registered trigger2 for combo break");
}

#[test]
fn test_trait_runtime_match_simulation() {
    let mut nc = base_char();
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
        armor: Some(ArmorSpec { hits: 1, threshold: None }),
        parry: None,
        burst: None,
        flight: None,
        timestop: Some(90),
        throw: None,
        cancels: None,
    });

    let def = std::sync::Arc::new(compile_native(&nc, 1, false));
    let mut m = duel(def, mugen_char("dummy", ""));
    enter(&mut m, 0, 1000);
    ticks(&mut m, 2);

    assert!(m.fighters[0].hit_overrides[0].is_some(), "Runtime HitOverride active");
    assert!(m.pause.is_some(), "Runtime Pause active during Time Stop");
}
