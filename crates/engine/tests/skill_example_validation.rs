//! Validation test ensuring the example character bundled with the ume-character-creator skill
//! compiles cleanly and respects all constraints.
use ume_engine::native::{compile_native, NativeChar};

#[test]
fn test_skill_example_character_compilation() {
    let raw = include_str!(
        "../../../.agents/skills/ume-character-creator/examples/exemplo_lutador_completo.json"
    );
    let nc: NativeChar = serde_json::from_str(raw).expect("valid UMF example json");
    assert_eq!(nc.id, "zephyr");
    assert!(nc.abilities.is_some());

    let def = compile_native(&nc, 1, false);
    assert_eq!(def.name, "Zephyr Skywalker");
    assert_eq!(def.forms.len(), 3);
    assert!(def.commands.iter().any(|c| c.name == "fly_toggle"));
    assert!(def.commands.iter().any(|c| c.name == "teleport_cmd"));
    assert!(def.commands.iter().any(|c| c.name == "speed_toggle"));

    let fly = def
        .movelist
        .iter()
        .find(|m| m.command == "fly_toggle")
        .unwrap();
    assert_eq!(fly.cost, 60.0);
    assert!(fly.drain);

    let super_mv = def
        .movelist
        .iter()
        .find(|m| m.name == "Furacão Primordial")
        .unwrap();
    assert_eq!(super_mv.cost, 1000.0);
    assert_eq!(
        super_mv.forms,
        u32::MAX & !(1 << 2),
        "the base super is hidden only in the form that replaces moves"
    );

    let form_mv = def
        .movelist
        .iter()
        .find(|m| m.name == "Raio Cataclismo")
        .unwrap();
    assert_eq!(form_mv.cost, 1000.0);
    assert_eq!(form_mv.forms, 1 << 2);
}
