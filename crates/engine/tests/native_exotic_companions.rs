use crate::support;
use serde_json::json;
use std::sync::Arc;
use support::*;
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn base_with(patch: serde_json::Value) -> NativeChar {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    v["id"] = json!("exotic_companions");
    v["abilities"] = serde_json::Value::Null;
    v["passives"] = serde_json::Value::Null;
    v["moves"] = json!([
        {"id": "jab", "name": "Jab", "input": "x", "kind": "normal", "anim": "jab", "hits": [{"at": 2, "damage": 40, "level": "light"}]}
    ]);
    if let Some(obj) = patch.as_object() {
        for (k, val) in obj {
            v[k] = val.clone();
        }
    }
    serde_json::from_value(v).unwrap()
}

fn press(m: &mut Match, p: usize, cmd_name: &str) {
    let def = m.fighters[p].def.clone();
    let cmd = def
        .commands
        .iter()
        .find(|c| c.name == cmd_name)
        .unwrap_or_else(|| panic!("missing cmd {cmd_name}"));
    let mut inputs = [0u32; 2];
    for step in &cmd.steps {
        inputs[p] = step.dir | step.btns;
        m.tick(&inputs);
        m.tick(&inputs);
    }
    inputs[p] = 0;
    m.tick(&inputs);
}

#[test]
fn stand_fights_independently_syncs_and_transfers_damage() {
    let nc = base_with(json!({
        "helpers": {
            "stand": {
                "id": 7000,
                "input": "z",
                "idleAnim": "idle",
                "offset": [36, 0],
                "hittable": true,
                "lifeTransfer": 0.5,
                "moves": [
                    {"id": "ora", "name": "Ora", "input": "c", "anim": "jab", "hits": [{"at": 2, "damage": 50, "level": "medium"}]},
                    {"id": "sync", "name": "Sync", "triggerMove": "jab", "anim": "jab", "hits": [{"at": 2, "damage": 20, "level": "light"}]}
                ]
            }
        }
    }));
    let def = Arc::new(compile_native(&nc, 1, true));
    let dummy = Arc::new(compile_native(&base_with(json!({})), 1, true));
    let mut m = duel(def, dummy);
    face_off(&mut m, 64.0);

    press(&mut m, 0, "stand_toggle");
    ticks(&mut m, 2);
    let stand_idx = m
        .fighters
        .iter()
        .position(|f| f.is_helper && f.helper_id == 7000)
        .expect("stand spawned");
    assert_eq!(
        m.fighters[0].state_no, 0,
        "parent stays idle after summoning stand"
    );

    let p2_before = m.fighters[1].life;
    press(&mut m, 0, "stand_m_ora");
    ticks(&mut m, 20);
    assert!(
        m.fighters[1].life < p2_before,
        "stand dealt independent damage"
    );
    assert_eq!(
        m.fighters[0].state_no, 0,
        "parent stayed out of attack state"
    );

    enter(&mut m, 0, 200);
    ticks(&mut m, 4);
    assert!(
        m.fighters[stand_idx].state_no >= 7010,
        "stand synced to parent move"
    );
    ticks(&mut m, 30);

    let p1_before = m.fighters[0].life;
    m.fighters[0].pos.x = m.fighters[1].pos.x - 68.0;
    ticks(&mut m, 2);
    enter(&mut m, 1, 200);
    ticks(&mut m, 12);
    assert!(
        m.fighters[0].life < p1_before,
        "parent took transferred damage when stand was hit"
    );

    press(&mut m, 0, "stand_toggle");
    ticks(&mut m, 10);
    assert!(
        !m.fighters
            .iter()
            .any(|f| !f.destroyed && f.helper_id == 7000),
        "stand recalled"
    );
}

#[test]
fn autonomous_clones_chase_attack_poof_and_expire() {
    let nc = base_with(json!({
        "helpers": {
            "clones": {
                "id": 7100,
                "input": "y",
                "count": 2,
                "maxActive": 2,
                "cost": 0,
                "duration": 90,
                "spawnOffset": [30, 0],
                "walkSpeed": 4.0,
                "attackRange": 30.0,
                "oneHitPoof": true,
                "attacks": [
                    {"id": "cjab", "name": "Clone Jab", "input": "x", "kind": "normal", "anim": "jab", "hits": [{"at": 2, "damage": 30, "level": "light"}]}
                ]
            }
        }
    }));
    let def = Arc::new(compile_native(&nc, 1, true));
    let mut m = duel(def.clone(), def);
    face_off(&mut m, 140.0);

    press(&mut m, 0, "clone_summon");
    ticks(&mut m, 3);
    let clone_count = m
        .fighters
        .iter()
        .filter(|f| !f.destroyed && f.is_helper && f.helper_id == 7100)
        .count();
    assert_eq!(clone_count, 2, "spawned two autonomous clones");

    let p2_before = m.fighters[1].life;
    let mut clone_attacked = false;
    for _ in 0..45 {
        m.tick(&[0, 0]);
        if m.fighters
            .iter()
            .any(|f| !f.destroyed && f.helper_id == 7100 && f.state_no == 7120)
        {
            clone_attacked = true;
        }
    }
    assert!(
        clone_attacked,
        "clone autonomously transitioned into attack state"
    );
    assert!(
        m.fighters[1].life < p2_before,
        "clone dealt damage to opponent"
    );

    let mut clone_indices: Vec<usize> = m
        .fighters
        .iter()
        .enumerate()
        .filter(|(_, f)| !f.destroyed && f.helper_id == 7100)
        .map(|(i, _)| i)
        .collect();
    for &ci in &clone_indices {
        enter(&mut m, ci, 7100);
    }
    let survivor = clone_indices.remove(0);
    m.fighters[1].hit_shake = 0;
    m.fighters[1].facing = -1.0;
    m.fighters[survivor].pos.x = m.fighters[1].pos.x - 28.0;
    for ci in clone_indices {
        m.fighters[ci].pos.x = m.fighters[1].pos.x + 120.0;
    }
    enter(&mut m, 1, 200);
    ticks(&mut m, 10);
    assert!(
        m.fighters
            .iter()
            .filter(|f| !f.destroyed && f.helper_id == 7100)
            .count()
            < 2,
        "one-hit poof destroyed hit clone"
    );

    ticks(&mut m, 70);
    assert_eq!(
        m.fighters
            .iter()
            .filter(|f| !f.destroyed && f.helper_id == 7100)
            .count(),
        0,
        "clones expired"
    );
}
