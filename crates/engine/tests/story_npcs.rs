use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::input::InputSource;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const NPCS: [(&str, &str); 5] = [
    ("npc_brute", "bruno"),
    ("npc_razor", "suzume"),
    ("npc_spitfire", "akio"),
    ("npc_colossus", "daichi"),
    ("npc_phantom", "jin"),
];

fn load(id: &str) -> NativeChar {
    let path = format!("{}/../../assets/chars/{id}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn compiled(id: &str) -> Arc<CharDef> {
    Arc::new(compile_native(&load(id), 1, false))
}

#[test]
fn story_npcs_are_hidden_and_listed_in_the_index() {
    let index: Vec<String> = serde_json::from_str(
        &std::fs::read_to_string(format!("{}/../../assets/chars/index.json", env!("CARGO_MANIFEST_DIR"))).unwrap(),
    )
    .unwrap();
    for (id, _) in NPCS {
        let nc = load(id);
        assert!(nc.npc && !nc.boss, "{id} is an npc, not an arcade boss");
        assert!(index.iter().any(|entry| entry == id), "{id} missing from index.json");
        assert!(!compiled(id).boss, "{id} must not get boss empowerment");
    }
}

#[test]
fn every_npc_wears_a_palette_different_from_its_base_fighter() {
    for (id, base) in NPCS {
        let (npc, origin) = (load(id), load(base));
        assert_eq!(npc.sprite_set.as_deref(), Some(base), "{id} reuses the base pixel art");
        assert_ne!(npc.palettes[0], origin.palettes[0], "{id} keeps the base palette");
        assert_ne!(npc.palettes[0], npc.palettes[1], "{id} has two identical palettes");
    }
}

#[test]
fn every_npc_has_its_own_ability_over_the_base_fighter() {
    let brute = load("npc_brute");
    assert!(brute.moves.iter().any(|m| m.id == "strong" && m.armor.is_some()));
    assert!(load("bruno").moves.iter().all(|m| m.id != "strong" || m.armor.is_none()));
    let spitfire = load("npc_spitfire");
    assert!(spitfire.moves.iter().any(|m| m.id == "jab" && m.projectile.is_some()));
    let colossus = load("npc_colossus");
    assert!(colossus.stats.life.unwrap() > load("daichi").stats.life.unwrap(), "giant is sturdier than its base");
    let razor = load("npc_razor");
    assert!(razor.speed.unwrap() > load("suzume").speed.unwrap(), "knife runner animates faster");
    let phantom = load("npc_phantom");
    assert!(phantom.moves.iter().any(|m| m.id == "lkick" && m.invuln.is_some()));
    assert!(phantom.speed.unwrap() > load("jin").speed.unwrap(), "shadow runner animates faster");
}

#[test]
fn a_hero_survives_a_wave_of_four_thugs_without_the_simulation_breaking() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = true;
    m.add_player(compiled("kaito"), 0, 0, 1);
    for (id, _) in NPCS.iter().take(4) {
        m.add_player(compiled(id), 1, 0, 1);
    }
    m.start_round();
    let mut ais: Vec<Ai> = (0..5).map(|slot| Ai::new(4, 20 + slot as u32)).collect();
    for _ in 0..1800 {
        let bits: Vec<u32> = ais.iter_mut().enumerate().map(|(slot, ai)| ai.poll(&m, slot)).collect();
        m.tick(&bits);
    }
    for f in m.fighters.iter().filter(|f| !f.is_helper) {
        assert!(f.life.is_finite() && f.pos.x.is_finite() && f.pos.y.is_finite(), "{} went non-finite", f.id);
        assert!(f.life <= f.life_max);
    }
}
