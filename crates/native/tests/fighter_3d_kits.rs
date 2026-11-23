use ume_native::{anim_numbers, compile_native, parse_char};

#[test]
fn ichigo3d_extends_kaito_and_compiles_its_getsuga_kit() {
    parse_char(include_str!("../../../assets/chars/kaito.json")).expect("kaito parses");
    let ichigo = parse_char(include_str!("../../../assets/chars3d/ichigo3d.json")).expect("ichigo3d parses");
    let names: Vec<&str> = ichigo.moves.iter().map(|m| m.name.as_str()).collect();
    for wanted in ["Getsuga Tenshou", "Shunpo", "Hollow Cero", "Getsuga Juujishou", "Kuro Getsuga", "Hollow Rampage"] {
        assert!(names.contains(&wanted), "missing {wanted}: {names:?}");
    }
    assert!(!names.contains(&"Gale Fist Slow"));
    assert!(anim_numbers(&ichigo).contains_key("getsugaFx"));
    let def = compile_native(&ichigo, 1, false);
    assert!(def.states.len() > 50);
}

#[test]
fn ichigo3d_getsuga_and_cero_charge_release_into_tiers() {
    parse_char(include_str!("../../../assets/chars/kaito.json")).expect("kaito parses");
    let ichigo = parse_char(include_str!("../../../assets/chars3d/ichigo3d.json")).expect("ichigo3d parses");
    let def = compile_native(&ichigo, 1, false);
    let holds: Vec<&str> = def.commands.iter().map(|c| c.name.as_str()).filter(|n| n.starts_with("hold_")).collect();
    assert!(holds.contains(&"hold_y") && holds.contains(&"hold_a"), "{holds:?}");
    let names: Vec<&str> = ichigo.moves.iter().map(|m| m.name.as_str()).collect();
    for wanted in ["Getsuga Tenshou (Charge)", "Getsuga Tenshou (Max)", "Hollow Cero (Max)"] {
        assert!(names.contains(&wanted), "missing {wanted}: {names:?}");
    }
    let listed: Vec<&str> = def.movelist.iter().map(|m| m.name.as_str()).collect();
    assert!(!listed.contains(&"Getsuga Tenshou (Max)"), "release tiers must not have their own input");
}
