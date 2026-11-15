use ume_engine::native::{compile_native, parse_char, register_kit, NativeChar};

const KAITO: &str = include_str!("../../../assets/chars/kaito.json");
const APPRENTICE: &str = include_str!("../../../docs/examples/umf/mika.json");
const KIT: &str = include_str!("../../../docs/examples/umf/storm-kit.json");

fn base() -> NativeChar {
    parse_char(KAITO).unwrap()
}

#[test]
fn a_roster_fighter_parses_the_same_with_or_without_the_kit_parser() {
    let direct: NativeChar = serde_json::from_str(KAITO).unwrap();
    assert_eq!(
        serde_json::to_value(base()).unwrap(),
        serde_json::to_value(direct).unwrap()
    );
}

#[test]
fn the_example_fighter_inherits_a_whole_fighter_in_a_few_lines() {
    let kaito = base();
    register_kit(KIT).unwrap();
    let mika = parse_char(APPRENTICE).unwrap();
    assert!(APPRENTICE.len() * 20 < KAITO.len());
    assert_eq!(mika.id, "mika");
    assert_eq!(mika.sprite_set.as_deref(), Some("kaito"));
    assert_ne!(mika.name, kaito.name);
    assert!(!mika.boss && !mika.npc);
    let names: Vec<&str> = mika.anims.iter().map(|(name, _)| name.as_str()).collect();
    let inherited: Vec<&str> = kaito.anims.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, inherited);
    let find = |id: &str| mika.moves.iter().find(|entry| entry.id == id);
    let jab = kaito.moves.iter().find(|entry| entry.id == "jab").unwrap();
    assert_eq!(find("jab").unwrap().hits[0].damage, jab.hits[0].damage);
    assert!(find("slide").is_none());
    assert!(find("squall").is_some_and(|squall| squall.cost == Some(500.0)));
    assert!(find("storm_step").is_some());
    assert!(mika.stats.life.unwrap() > kaito.stats.life.unwrap());
}

#[test]
fn the_example_fighter_compiles_to_a_playable_character() {
    let kaito = compile_native(&base(), 1, false);
    register_kit(KIT).unwrap();
    let mika = compile_native(&parse_char(APPRENTICE).unwrap(), 1, false);
    assert!(mika.states.len() > 20 && mika.states.len() + 2 >= kaito.states.len());
    assert_eq!(mika.anims.len(), kaito.anims.len());
}
