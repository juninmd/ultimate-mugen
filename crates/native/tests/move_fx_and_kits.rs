use serde_json::json;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;
use ume_native::{compile_native, parse_char};

fn base_char(extra: serde_json::Value) -> ume_native::NativeChar {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    v["id"] = json!("fx_test_fighter");
    if let (Some(dst), Some(src)) = (v.as_object_mut(), extra.as_object()) {
        for (k, val) in src {
            dst.insert(k.clone(), val.clone());
        }
    }
    parse_char(&v.to_string()).unwrap()
}

#[test]
fn move_fx_emits_explod_shake_flash_afterimage_and_directional_teleport() {
    let nc = base_char(json!({
        "moves": [
            {
                "id": "ki_rush",
                "name": "Ki Rush",
                "input": "~D, DF, F, x",
                "kind": "super",
                "anim": "special",
                "cost": 1000,
                "teleport": { "at": 1, "mode": "offset", "offset": [80, -30], "turn": false },
                "explods": [{ "at": 1, "anim": "idle", "pos": [10, -40], "bind": 12, "removeTime": 20 }],
                "afterimage": { "at": 1, "time": 30, "length": 8, "trans": "add", "palContrast": [255, 220, 100] },
                "shake": { "at": 2, "time": 18, "ampl": 6, "freq": 75 },
                "flash": { "at": 1, "time": 12, "color": [0, 0, 0], "under": true },
                "superPause": { "at": 1, "time": 40, "pos": [0, -50], "anim": 30 },
                "hits": [{ "at": 2, "damage": 40, "level": "heavy" }],
                "followUp": { "to": "ki_finisher", "on": "hit" }
            },
            {
                "id": "ki_finisher",
                "name": "Ki Finisher",
                "input": "~D, DF, F, x+y+z",
                "kind": "super",
                "anim": "strong",
                "hits": [{ "at": 2, "damage": 80, "level": "heavy", "launch": true }]
            }
        ]
    }));
    let def = compile_native(&nc, 1, false);
    let rush_state = def
        .movelist
        .iter()
        .find(|m| m.name == "Ki Rush")
        .unwrap()
        .state;
    let st = def.states.get(&rush_state).unwrap();
    let types: Vec<&str> = st.controllers.iter().map(|c| c.kind.as_str()).collect();
    assert!(types.contains(&"explod"), "expected Explod controller");
    assert!(types.contains(&"envshake"), "expected EnvShake controller");
    assert!(types.contains(&"envcolor"), "expected EnvColor controller");
    assert!(
        types.contains(&"afterimage"),
        "expected AfterImage controller"
    );
    assert!(
        types.contains(&"superpause"),
        "expected SuperPause controller"
    );
    assert!(
        types.contains(&"posadd"),
        "expected PosAdd teleport controller"
    );
    assert!(
        st.controllers
            .iter()
            .any(|c| c.kind == "changestate" && c.label.contains("followup")),
        "expected hit-confirm followUp ChangeState"
    );
}

#[test]
fn beam_projectile_and_custom_charge_compile_into_engine() {
    let nc = base_char(json!({
        "abilities": {
            "fly": { "speed": 4.2, "input": "a+b", "anim": "jumpUp", "upAnim": "jumpFwd", "downAnim": "jumpDown" },
            "teleport": { "cost": 200, "input": "b+c", "offset": 55, "invuln": 18, "turn": true },
            "charge": { "auraColor": [100, 80, 10], "shake": true }
        },
        "moves": [{
            "id": "mega_beam",
            "name": "Mega Beam",
            "input": "~D, DF, F, y",
            "kind": "super",
            "anim": "special",
            "cost": 1000,
            "projectile": {
                "at": 2,
                "anim": "shot",
                "speed": [6, 0],
                "offset": [30, -45],
                "damage": 12,
                "hits": 20,
                "missTime": 3,
                "removeTime": 60,
                "hitflag": "MAFP",
                "shake": 5
            }
        }]
    }));
    let def = Arc::new(compile_native(&nc, 1, false));
    assert!(def.states.contains_key(&3900), "fly state compiled");
    assert!(def.states.contains_key(&3910), "teleport state compiled");
    let charge_state = def.states.get(&ume_engine::rules::CHARGE_STATE).unwrap();
    assert!(
        charge_state
            .controllers
            .iter()
            .any(|c| c.kind == "envshake"),
        "custom charge shake compiled"
    );
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.start_round();
    for _ in 0..180 {
        m.tick(&[0, 0]);
    }
    assert!(m.fighters[0].life > 0.0);
}

#[test]
fn built_in_reusable_kits_expand_cleanly() {
    let fighter = json!({
        "format": "umf/2",
        "id": "kit_hero",
        "name": "Kit Hero",
        "series": "Test",
        "author": "Test",
        "color": "#ffaa00",
        "extends": ["core", "ki_fighter", "clone_arts"],
        "art": { "type": "sprites", "style": {}, "poses": {} },
        "anims": { "idle": { "frames": ["01_idle*8"] }, "strong": { "frames": ["01_idle*8"] }, "special": { "frames": ["01_idle*8"] }, "shot": { "frames": ["01_idle*8"] } },
        "moves": [{ "id": "jab", "name": "Jab", "input": "x", "kind": "normal", "anim": "idle", "hits": [{ "at": 1, "damage": 20 }] }]
    });
    let nc = parse_char(&fighter.to_string()).unwrap();
    assert!(nc.abilities.as_ref().and_then(|a| a.fly.as_ref()).is_some());
    assert!(nc
        .abilities
        .as_ref()
        .and_then(|a| a.teleport.as_ref())
        .is_some());
    assert!(nc
        .abilities
        .as_ref()
        .and_then(|a| a.charge.as_ref())
        .is_some());
    assert!(nc
        .helpers
        .as_ref()
        .and_then(|h| h.clones.as_ref())
        .is_some());
    assert!(nc.moves.iter().any(|m| m.id == "ki_beam"));
    assert!(nc.moves.iter().any(|m| m.id == "launcher"));
    let def = compile_native(&nc, 1, false);
    assert!(
        def.states.contains_key(&7100),
        "clone helper state compiled from clone_arts kit"
    );
}

#[test]
fn converted_umf_bundle_simulates_cleanly() {
    let Ok(path) = std::env::var("UME_TEST_UMF_JSON") else {
        return;
    };
    let text = std::fs::read_to_string(&path).unwrap();
    let nc = parse_char(&text).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    assert!(def.states.contains_key(&3900), "fly ability state present");
    assert!(
        def.states.contains_key(&3910),
        "teleport ability state present"
    );
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 6, 1);
    m.add_player(def, 1, 6, 2);
    m.start_round();
    let (mut a, mut b) = (
        ume_engine::ai::Ai::new(6, 7),
        ume_engine::ai::Ai::new(6, 13),
    );
    let mut states_seen = std::collections::HashSet::new();
    let mut damage_dealt = 0.0_f32;
    let mut prev = [m.fighters[0].life, m.fighters[1].life];
    for _ in 0..3600 {
        let roots = m.root_indices();
        let (ia, ib) = (a.poll(&m, roots[0]), b.poll(&m, roots[1]));
        m.tick(&[ia, ib]);
        for i in 0..2 {
            states_seen.insert(m.fighters[i].state_no);
            if m.fighters[i].life < prev[i] {
                damage_dealt += prev[i] - m.fighters[i].life;
            }
            prev[i] = m.fighters[i].life;
        }
        if m.match_over {
            break;
        }
    }
    println!(
        "CONVERTED UMF SIM OK: id={} moves={} anims={} distinct_states={} total_damage={:.0}",
        nc.id,
        nc.moves.len(),
        nc.anims.0.len(),
        states_seen.len(),
        damage_dealt
    );
    assert!(states_seen.len() >= 10, "expected active combat states");
    assert!(damage_dealt > 0.0, "expected hits to connect in CPU fight");
}
