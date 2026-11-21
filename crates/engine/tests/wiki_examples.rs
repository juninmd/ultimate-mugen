use ume_native::{compile_native, parse_char};

const KAITO: &str = include_str!("../../../assets/chars/kaito.json");
const HELLO: &str = include_str!("../../../docs/examples/umf/hello-fighter.json");
const MY_STAGE: &str = include_str!("../../../docs/examples/stage/my-stage.def");

#[test]
fn the_wiki_hello_fighter_inherits_kaito_and_compiles() {
    let kaito = parse_char(KAITO).unwrap();
    let hello = parse_char(HELLO).unwrap();
    assert_eq!(hello.id, "hello");
    assert_eq!(hello.moves.len(), kaito.moves.len());
    let gale = hello.moves.iter().find(|entry| entry.id == "gale").unwrap();
    assert_eq!(gale.name, "Sky Gale");
    assert_eq!(gale.cost, Some(500.0));
    let _ = compile_native(&hello, 1, false);
}

#[test]
fn the_wiki_stage_example_has_the_sections_the_loader_reads() {
    let secs = ume_formats::ini::parse_ini(MY_STAGE);
    for section in ["Info", "Camera", "PlayerInfo", "StageInfo", "BGdef", "BG Sky", "BG Floor"] {
        assert!(
            ume_formats::ini::find_section(&secs, section).is_some(),
            "missing [{section}]"
        );
    }
}
