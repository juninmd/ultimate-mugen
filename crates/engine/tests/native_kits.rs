use std::collections::HashSet;
use ume_engine::native::{compile_native, NativeChar};

const KIT: [&str; 16] = ["kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian", "garan", "marlow", "kaname", "jin", "daichi", "valdric", "haruki", "oboro"];

fn load(id: &str) -> NativeChar {
    let path = format!("{}/../../assets/chars/{id}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn every_fighter_has_the_full_toolkit() {
    for id in KIT {
        let nc = load(id);
        let has = |prefix: &str| nc.moves.iter().any(|m| m.id.starts_with(prefix));
        assert!(has("ex_"), "{id} ex move");
        assert!(has("launcher") && has("overhead") && has("divekick"), "{id} normals");
        if has("ult_") { assert!(has("slide") && has("escape_burst"), "{id} mixups"); }
        assert!(nc.passives.is_some(), "{id} passive");
        let supers = nc.moves.iter().filter(|m| m.kind == "super").count();
        assert!(supers >= 2, "{id} supers");
    }
}

#[test]
fn toolkit_ids_and_inputs_do_not_collide() {
    for id in KIT {
        let nc = load(id);
        let mut ids = HashSet::new();
        let mut inputs = HashSet::new();
        let forms = nc.abilities.iter().flat_map(|a| a.forms.iter()).flat_map(|f| f.moves.iter());
        for m in nc.moves.iter().chain(forms) {
            assert!(ids.insert(m.id.clone()), "{id} duplicate id {}", m.id);
        }
        for m in &nc.moves {
            let key = format!("{}|{}", m.input, m.when.as_deref().unwrap_or("stand"));
            assert!(inputs.insert(key), "{id} duplicate input {} for {}", m.input, m.id);
        }
    }
}

#[test]
fn toolkit_compiles_and_reaches_the_move_list() {
    for id in KIT {
        let nc = load(id);
        let def = compile_native(&nc, 1, false);
        for m in nc.moves.iter().filter(|m| m.id.starts_with("ex_") || m.id.starts_with("ult_") || m.id == "launcher" || m.id == "escape_burst") {
            assert!(def.movelist.iter().any(|e| e.name == m.name), "{id} movelist {}", m.name);
        }
        assert!(def.ai_commands.len() >= nc.moves.len(), "{id} ai commands");
        if nc.moves.iter().any(|m| m.id.starts_with("ult_")) {
            assert!(def.forms.len() >= 2, "{id} awakening and ascended forms");
        }
    }
}
