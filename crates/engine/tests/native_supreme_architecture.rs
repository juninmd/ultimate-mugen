//! Homologation tests for UMF Supreme Architecture:
//! Hit priority, asymmetric hitstop, OTG hitflag, ground bounce, cornerpush,
//! custom sparks/sounds, throws/grabs, target combo cancels, and Statedef -2 passives.
use ume_engine::native::{compile_native, HitSpec, NativeChar, NativeMove, PassiveSpec, ThrowSpec};

fn base_char() -> NativeChar {
    let raw = include_str!("../../../assets/chars/kaito.json");
    serde_json::from_str(raw).expect("valid kaito template")
}

#[test]
fn test_hit_priority_and_advanced_hitdef_flags() {
    let mut nc = base_char();
    nc.id = "supreme_fighter".into();
    nc.moves.clear();
    nc.moves.push(NativeMove {
        id: "heavy_launcher".into(),
        name: "Heavy Launcher".into(),
        input: "F+y".into(),
        simple: None,
        kind: "normal".into(),
        when: Some("stand".into()),
        anim: "strong".into(),
        hits: vec![HitSpec {
            at: 2,
            damage: 85.0,
            guard_damage: Some(15.0),
            level: Some("heavy".into()),
            guard: Some("mid".into()),
            knockback: Some([-3.0, -9.0]),
            air_knockback: None,
            launch: Some(true),
            hitstop: None,
            hitstun: Some(24),
            spark: Some(2),
            shake: Some(6),
            priority: Some(6),
            hitflag: Some("MAFD".into()),
            pausetime: Some([7, 14]),
            cornerpush: Some(-4.5),
            hit_sound: Some([5, 3]),
            guard_sound: Some([6, 1]),
            spark_pos: Some([35.0, -55.0]),
            ground_bounce: Some(true),
        }],
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
        flight: None,
        timestop: None,
        throw: None,
        cancels: Some(vec!["gale".into()]),
    });

    let def = compile_native(&nc, 1, false);
    let state = def.states.get(&200).expect("statedef 200 exists");
    let hd = state
        .controllers
        .iter()
        .find(|sc| sc.kind == "hitdef")
        .expect("hitdef found");

    assert_eq!(hd.raw("priority"), Some("6, Hit"));
    assert_eq!(hd.raw("hitflag"), Some("MAFD"));
    assert_eq!(hd.raw("pausetime"), Some("7, 14"));
    assert_eq!(hd.raw("ground.cornerpush.veloff"), Some("-4.5"));
    assert_eq!(hd.raw("hitsound"), Some("5, 3"));
    assert_eq!(hd.raw("guardsound"), Some("6, 1"));
    assert_eq!(hd.raw("sparkxy"), Some("35, -55"));
    assert_eq!(hd.raw("down.bounce"), Some("1"));

    // Verify cancels
    let minus1 = def.states.get(&-1).expect("statedef -1 exists");
    let has_cancel_chain = minus1
        .controllers
        .iter()
        .any(|sc| sc.raw("value") == Some("200"));
    assert!(has_cancel_chain);
}

#[test]
fn test_throw_and_passives_mechanics() {
    let mut nc = base_char();
    nc.id = "thrower".into();
    nc.moves.clear();
    nc.passives = Some(PassiveSpec {
        meter_regen: Some(1.2),
        life_regen: Some(0.3),
    });

    nc.moves.push(NativeMove {
        id: "judo_throw".into(),
        name: "Judo Throw".into(),
        input: "F+z".into(),
        simple: None,
        kind: "normal".into(),
        when: Some("stand".into()),
        anim: "jab".into(),
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
        flight: None,
        timestop: None,
        throw: Some(ThrowSpec {
            damage: 130.0,
            bind: Some([35.0, -15.0]),
            p2_state: Some(820),
            at: None,
            launch: None,
        }),
        cancels: None,
    });

    let def = compile_native(&nc, 1, false);

    // Verify Statedef -2 passives
    let minus2 = def.states.get(&-2).expect("statedef -2 exists");
    let has_power_add = minus2.controllers.iter().any(|sc| sc.kind == "poweradd");
    let has_life_add = minus2.controllers.iter().any(|sc| sc.kind == "lifeadd");
    assert!(
        has_power_add,
        "Statedef -2 contains PowerAdd for meter regen"
    );
    assert!(has_life_add, "Statedef -2 contains LifeAdd for life regen");

    // Verify Throw controllers
    let state = def.states.get(&200).expect("throw state exists");
    let has_bind = state
        .controllers
        .iter()
        .any(|sc| sc.kind == "targetbind" && sc.raw("pos") == Some("35, -15"));
    let has_target_dmg = state
        .controllers
        .iter()
        .any(|sc| sc.kind == "targetlifeadd" && sc.raw("value") == Some("-130"));
    assert!(has_bind, "TargetBind positioned properly");
    assert!(has_target_dmg, "TargetLifeAdd applied damage");
}
