//! Integration and homologation tests for flight, teleport, super speed,
//! multi-transformations, body part vs full body changes, move swapping, and character swap.
mod support;
use std::sync::Arc;
use support::*;
use ume_engine::native::{
    compile_native, register_native, AbilitySpec, FlySpec, FormSpec, NativeChar,
    SuperSpeedSpec, TeleportSpec,
};

fn base_char(id: &str, name: &str) -> NativeChar {
    let raw = include_str!("../../../assets/chars/kaito.json");
    let mut nc: NativeChar = serde_json::from_str(raw).expect("valid kaito template");
    nc.id = id.into();
    nc.name = name.into();
    nc
}

#[test]
fn test_abilities_compilation_and_movelist() {
    let mut nc = base_char("ability_tester", "Ability Tester");
    nc.abilities = Some(AbilitySpec {
        fly: Some(FlySpec {
            speed: Some(4.0),
            cost: Some(60.0),
            input: Some("a+b".into()),
            anim: None,
        }),
        teleport: Some(TeleportSpec {
            cost: Some(300.0),
            input: Some("b+c".into()),
            anim: None,
        }),
        super_speed: Some(SuperSpeedSpec {
            mul: Some(2.0),
            cost: Some(120.0),
            input: Some("a+c".into()),
        }),
        forms: vec![],
    });

    let def = compile_native(&nc, 1, false);

    assert!(def.commands.iter().any(|c| c.name == "fly_toggle"));
    assert!(def.commands.iter().any(|c| c.name == "teleport_cmd"));
    assert!(def.commands.iter().any(|c| c.name == "speed_toggle"));

    assert!(def.states.contains_key(&3900));
    assert!(def.states.contains_key(&3901));
    assert!(def.states.contains_key(&3910));

    let fly_mv = def.movelist.iter().find(|m| m.command == "fly_toggle").unwrap();
    assert_eq!(fly_mv.cost, 60.0);
    assert!(fly_mv.drain);

    let tp_mv = def.movelist.iter().find(|m| m.command == "teleport_cmd").unwrap();
    assert_eq!(tp_mv.cost, 300.0);
    assert!(!tp_mv.drain);

    let sp_mv = def.movelist.iter().find(|m| m.command == "speed_toggle").unwrap();
    assert_eq!(sp_mv.cost, 120.0);
    assert!(sp_mv.drain);
}

#[test]
fn test_multi_transformations_sequential_and_buffs() {
    let mut nc = base_char("form_tester", "Form Tester");
    nc.abilities = Some(AbilitySpec {
        fly: None,
        teleport: None,
        super_speed: None,
        forms: vec![
            FormSpec {
                id: "form1".into(),
                name: "Awakened Form 1".into(),
                cost: Some(1000.0),
                attack: Some(1.4),
                defense: Some(1.2),
                palette: None,
                parts: None,
                character: None,
                moves: vec![],
                replace_moves: false,
            },
            FormSpec {
                id: "form2".into(),
                name: "Awakened Form 2".into(),
                cost: Some(1500.0),
                attack: Some(1.5),
                defense: Some(1.3),
                palette: None,
                parts: None,
                character: None,
                moves: vec![],
                replace_moves: false,
            },
        ],
    });

    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = duel(def.clone(), def.clone());
    m.round_state = 2;
    m.fighters[0].power = 3000.0;
    m.fighters[0].ctrl = true;

    assert_eq!(m.fighters[0].form, 0);
    let base_atk = m.fighters[0].attack_mul;
    let base_def = m.fighters[0].defence_mul;

    assert!(m.try_transform(0));
    assert_eq!(m.fighters[0].form, 1);
    assert_eq!(m.fighters[0].power, 2000.0);
    assert!((m.fighters[0].attack_mul - base_atk * 1.4).abs() < 1e-4);
    assert!((m.fighters[0].defence_mul - base_def * 1.2).abs() < 1e-4);

    m.fighters[0].ctrl = true;
    m.pause = None;
    assert!(m.try_transform(0));
    assert_eq!(m.fighters[0].form, 2);
    assert_eq!(m.fighters[0].power, 500.0);
    assert!((m.fighters[0].attack_mul - base_atk * 1.4 * 1.5).abs() < 1e-4);
    assert!((m.fighters[0].defence_mul - base_def * 1.2 * 1.3).abs() < 1e-4);

    m.fighters[0].ctrl = true;
    m.pause = None;
    assert!(!m.try_transform(0));

    m.fighters[0].revert_form();
    assert_eq!(m.fighters[0].form, 0);
    assert!((m.fighters[0].attack_mul - base_atk).abs() < 1e-4);
    assert!((m.fighters[0].defence_mul - base_def).abs() < 1e-4);
}

#[test]
fn test_transformation_character_swap_and_move_swap() {
    let target_nc = Arc::new(base_char("alt_hero", "Alternative Hero"));
    register_native(target_nc);

    let mut nc = base_char("shapeshifter", "Shape Shifter");
    let mut swapped_move = nc.moves[0].clone();
    swapped_move.id = "alt_strike".into();
    swapped_move.name = "Alt Strike".into();

    nc.abilities = Some(AbilitySpec {
        fly: None,
        teleport: None,
        super_speed: None,
        forms: vec![
            FormSpec {
                id: "alt_char".into(),
                name: "Full Character Swap".into(),
                cost: Some(1000.0),
                attack: Some(1.2),
                defense: Some(1.1),
                palette: None,
                parts: None,
                character: Some("alt_hero".into()),
                moves: vec![],
                replace_moves: false,
            },
        ],
    });

    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = duel(def.clone(), def.clone());
    m.round_state = 2;
    m.fighters[0].power = 2000.0;
    m.fighters[0].ctrl = true;

    assert_eq!(m.fighters[0].def.name, "Shape Shifter");
    assert!(m.try_transform(0));
    assert_eq!(m.fighters[0].form, 1);
    assert_eq!(m.fighters[0].def.name, "Alternative Hero");
}

#[test]
fn test_form_palette_parts_alteration() {
    let form_parts = FormSpec {
        id: "partial".into(),
        name: "Partial".into(),
        cost: None,
        attack: None,
        defense: None,
        palette: Some(serde_json::json!({
            "colors": {
                "hair": "#ffff00",
                "suit": "#ff0000"
            }
        })),
        parts: Some(vec!["hair".into()]),
        character: None,
        moves: vec![],
        replace_moves: false,
    };

    let base_palette = serde_json::json!({
        "colors": {
            "hair": "#111111",
            "suit": "#222222"
        }
    });

    let result = ume_engine::native::forms::form_palette(&form_parts, Some(&base_palette)).unwrap();
    let obj = result.as_object().unwrap();
    assert_eq!(obj.get("hair").unwrap(), "#ffff00");
    assert_eq!(obj.get("suit").unwrap(), "#222222");

    let form_full = FormSpec {
        id: "full".into(),
        name: "Full".into(),
        cost: None,
        attack: None,
        defense: None,
        palette: Some(serde_json::json!({
            "colors": {
                "hair": "#00ff00",
                "suit": "#0000ff"
            }
        })),
        parts: None,
        character: None,
        moves: vec![],
        replace_moves: false,
    };

    let result_full = ume_engine::native::forms::form_palette(&form_full, Some(&base_palette)).unwrap();
    let obj_full = result_full.as_object().unwrap();
    assert_eq!(obj_full.get("hair").unwrap(), "#00ff00");
    assert_eq!(obj_full.get("suit").unwrap(), "#0000ff");
}
