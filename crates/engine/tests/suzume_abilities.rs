//! Integration and homologation test suite for Suzume with all new core features:
//! fly, teleport, super speed, multi-transformations, move swaps, and character swap.
mod support;
use std::sync::Arc;
use support::*;
use ume_engine::native::{compile_native, register_native, NativeChar};

fn suzume() -> NativeChar {
    let raw = include_str!("../../../assets/chars/suzume.json");
    serde_json::from_str(raw).expect("valid suzume.json")
}

fn oboro() -> NativeChar {
    let raw = include_str!("../../../assets/chars/oboro.json");
    serde_json::from_str(raw).expect("valid oboro.json")
}

#[test]
fn test_suzume_abilities_compilation_and_movelist() {
    let nc = suzume();
    let def = compile_native(&nc, 1, false);

    assert_eq!(def.name, "Suzume Kagerou");
    assert!(def.commands.iter().any(|c| c.name == "fly_toggle"));
    assert!(def.commands.iter().any(|c| c.name == "teleport_cmd"));
    assert!(def.commands.iter().any(|c| c.name == "speed_toggle"));

    assert!(def.states.contains_key(&3900));
    assert!(def.states.contains_key(&3901));
    assert!(def.states.contains_key(&3910));

    let fly = def
        .movelist
        .iter()
        .find(|m| m.command == "fly_toggle")
        .unwrap();
    assert_eq!(fly.cost, 50.0);
    assert!(fly.drain);

    let tp = def
        .movelist
        .iter()
        .find(|m| m.command == "teleport_cmd")
        .unwrap();
    assert_eq!(tp.cost, 250.0);
    assert!(!tp.drain);

    let sp = def
        .movelist
        .iter()
        .find(|m| m.command == "speed_toggle")
        .unwrap();
    assert_eq!(sp.cost, 100.0);
    assert!(sp.drain);

    let shadow = def
        .movelist
        .iter()
        .find(|m| m.command == "m_shadow")
        .unwrap();
    assert_eq!(shadow.cost, 250.0);

    let storm = def
        .movelist
        .iter()
        .find(|m| m.command == "m_storm")
        .unwrap();
    assert_eq!(storm.cost, 1000.0);
}

#[test]
fn test_suzume_multi_transformations_and_metamorphosis() {
    register_native(Arc::new(oboro()));

    let nc = suzume();
    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = duel(def.clone(), def.clone());
    m.round_state = 2;
    m.fighters[0].power = 3000.0;
    m.fighters[0].ctrl = true;

    assert_eq!(m.fighters[0].def.name, "Suzume Kagerou");
    assert_eq!(m.fighters[0].form, 0);
    let base_atk = m.fighters[0].attack_mul;
    let base_def = m.fighters[0].defence_mul;

    // Form 1: Lótus da Noite (Parcial)
    assert!(m.try_transform(0));
    assert_eq!(m.fighters[0].form, 1);
    assert_eq!(m.fighters[0].power, 2000.0);
    assert!((m.fighters[0].attack_mul - base_atk * 1.35).abs() < 1e-4);
    assert!((m.fighters[0].defence_mul - base_def * 1.15).abs() < 1e-4);

    // Form 2: Sombra Carmesim (Corpo Todo)
    m.fighters[0].ctrl = true;
    m.pause = None;
    assert!(m.try_transform(0));
    assert_eq!(m.fighters[0].form, 2);
    assert_eq!(m.fighters[0].power, 500.0);
    assert!((m.fighters[0].attack_mul - base_atk * 1.35 * 1.6).abs() < 1e-4);
    assert!((m.fighters[0].defence_mul - base_def * 1.15 * 1.35).abs() < 1e-4);

    // Add power for Form 3
    m.fighters[0].power = 1000.0;
    m.fighters[0].ctrl = true;
    m.pause = None;

    // Form 3: Invocação de Oboro (Troca de Personagem)
    assert!(m.try_transform(0));
    assert_eq!(m.fighters[0].form, 3);
    assert_eq!(m.fighters[0].power, 0.0);
    assert_eq!(m.fighters[0].def.name, "Mikazuki Oboro");

    // Revert form restores base Suzume
    m.fighters[0].revert_form();
    assert_eq!(m.fighters[0].form, 0);
    assert_eq!(m.fighters[0].def.name, "Suzume Kagerou");
    assert!((m.fighters[0].attack_mul - base_atk).abs() < 1e-4);
    assert!((m.fighters[0].defence_mul - base_def).abs() < 1e-4);
}
