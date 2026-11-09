use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

fn load_native(id: &str) -> Arc<CharDef> {
    let p = format!("{}/../../assets/chars/{id}.json", env!("CARGO_MANIFEST_DIR"));
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

#[test]
fn oboro_has_multi_form_abilities_and_phase2_and_phase3_moves() {
    let oboro = load_native("oboro");
    assert!(oboro.boss);
    assert_eq!(oboro.forms.len(), 3);
    assert_eq!(oboro.forms[0].name, "Mangekyo Desperto");
    assert_eq!(oboro.forms[1].name, "Eclipse Absoluto");
    assert_eq!(oboro.forms[2].name, "Eclipse Primordial");
    let move_names: Vec<&str> = oboro.movelist.iter().map(|m| m.name.as_str()).collect();
    assert!(move_names.contains(&"Corte do Eclipse Carmesim"));
    assert!(move_names.contains(&"Chama do Eclipse Eterno"));
    assert!(move_names.contains(&"Julgamento do Eclipse Absoluto"));
    assert!(move_names.contains(&"Colapso Primordial"));
    assert!(move_names.contains(&"Supernova do Eclipse Primordial"));
    assert!(oboro.commands.iter().any(|c| c.name == "fly_toggle"));
    assert!(oboro.commands.iter().any(|c| c.name == "teleport_cmd"));
    assert!(oboro.commands.iter().any(|c| c.name == "speed_toggle"));
}

#[test]
fn oboro_transforms_into_next_form_when_defeated_at_least_once() {
    let kaito = load_native("kaito");
    let oboro = load_native("oboro");
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(kaito, 0, 0, 1);
    m.add_player(oboro, 1, 0, 1);
    m.wins = [1, 0];
    m.round_no = 2;
    m.start_round();
    for _ in 0..320 {
        m.tick(&[0, 0]);
        if m.round_state == 2 {
            break;
        }
    }
    assert_eq!(m.round_state, 2);
    let boss = &m.fighters[1];
    assert_eq!(boss.form, 1);
    assert_eq!(boss.active_form_name(), "Mangekyo Desperto");
    assert!(boss.attack_mul > 1.3);
    assert!(boss.defence_mul > 1.2);
}

#[test]
fn awaken_boss_phase_starts_oboro_in_primordial_form_with_boosted_stats_and_ai() {
    let suzume = load_native("suzume");
    let oboro = load_native("oboro");
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(suzume, 0, 0, 1);
    m.add_player(oboro, 1, 0, 1);
    let base_life = m.fighters[1].life_max;
    m.awaken_boss_phase(3);
    let boss = &m.fighters[1];
    assert_eq!(boss.form, 3);
    assert_eq!(boss.active_form_name(), "Eclipse Primordial");
    assert!(boss.life_max > base_life * 1.25);
    assert_eq!(boss.power, boss.power_max);
    m.start_round();
    for _ in 0..320 {
        m.tick(&[0, 0]);
        if m.round_state == 2 {
            break;
        }
    }
    let mut ai_p1 = Ai::new(6, 11);
    let mut ai_boss = Ai::new(8, 99);
    for _ in 0..360 {
        let b0 = ai_p1.poll(&m, 0);
        let b1 = ai_boss.poll(&m, 1);
        m.tick(&[b0, b1]);
    }
    assert_eq!(m.fighters[1].form, 3);
    assert!(m.fighters[1].alive);
}
