use crate::support;
use serde_json::json;
use std::sync::Arc;
use support::*;
use ume_engine::flags::{asf, gsf};
use ume_engine::matchs::PauseState;
use ume_native::{compile_native, NativeChar};

fn base_with(patch: serde_json::Value) -> NativeChar {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    v["id"] = json!("exotic_lab");
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

#[test]
fn cinematic_throw_uses_changeanim2_steps_camera_free_and_failsafe() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    v["id"] = json!("cinematic");
    v["abilities"] = serde_json::Value::Null;
    v["passives"] = serde_json::Value::Null;
    v["anims"]["p2slam"] = json!({"id": 8801, "frames": [{"pose": "hit0", "ticks": -1}]});
    v["anims"]["cinethrow"] = json!({"frames": [
        {"pose": "jab1", "ticks": 3, "hit": {"part": "handF", "radius": 30}},
        {"pose": "jab1", "ticks": 6}, {"pose": "jab0", "ticks": 6}, {"pose": "jab1", "ticks": 6}
    ]});
    v["moves"] = json!([{
        "id": "cgrab", "name": "Cinematic Slam", "input": "x+a", "kind": "normal", "anim": "cinethrow",
        "throw": {
            "damage": 90, "at": 1, "p2Anim": "p2slam", "cameraFree": true,
            "camera": {"zoom": 1.35, "time": 20},
            "steps": [
                {"at": 2, "bind": [30, -25], "damage": 30, "shake": 6},
                {"at": 3, "bind": [-20, -40], "p2Anim": "p2slam", "damage": 25, "facing": -1},
                {"at": 4, "damage": 35, "launch": [-4, -7], "shake": 10}
            ]
        }
    }]);
    let nc: NativeChar = serde_json::from_value(v).unwrap();
    let def = Arc::new(compile_native(&nc, 1, true));
    let other = Arc::new(compile_native(&base_with(json!({})), 1, true));
    let mut m = duel(def, other);
    face_off(&mut m, 34.0);
    let before = m.fighters[1].life;

    enter(&mut m, 0, 200);
    let mut saw_custom_anim = false;
    let mut saw_camera_free = false;
    for _ in 0..80 {
        m.tick(&[0, 0]);
        if m.fighters[1].anim_no == 8801
            && !Arc::ptr_eq(&m.fighters[1].anim_def, &m.fighters[1].def)
        {
            saw_custom_anim = true;
        }
        if !m.fighters[0].move_camera[0] && !m.fighters[1].move_camera[0] {
            saw_camera_free = true;
        }
    }
    assert!(
        saw_custom_anim,
        "victim played attacker ChangeAnim2 id 8801"
    );
    assert!(
        saw_camera_free,
        "cameraFree disabled ScreenBound movecamera"
    );
    let dealt = before - m.fighters[1].life;
    assert!(
        dealt >= 80.0 && dealt <= 95.0,
        "multi-step throw dealt expected damage, got {dealt}"
    );

    ticks(&mut m, 90);
    face_off(&mut m, 34.0);
    enter(&mut m, 0, 200);
    ticks(&mut m, 5);
    enter(&mut m, 0, 5000);
    ticks(&mut m, 15);
    assert_ne!(
        m.fighters[1].state_no, 3800,
        "IsBound failsafe exited custom throw state"
    );
    ticks(&mut m, 120);
    assert!(
        m.fighters[1].ctrl,
        "IsBound failsafe freed victim after throw interruption"
    );
}

#[test]
fn boss_rules_and_inline_minus4_zss_work_together() {
    let nc = base_with(json!({
        "bossRules": {
            "hideLifeBar": true,
            "hideTimer": true,
            "texts": [{"text": "APEX BOSS", "pos": [160, 20], "time": 2}],
            "phases": [{
                "id": "enrage",
                "lifeBelowRatio": 0.6,
                "once": true,
                "attackMul": 2.0,
                "texts": [{"text": "PHASE TWO", "pos": [160, 40], "time": 10}],
                "modifyPlayer": [{"target": "enemy", "power": "0"}]
            }]
        },
        "scripts": {
            "zss": "[StateDef -4]\nignorehitpause {\n    if RoundState = 2 {\n        map(tick_count) := map(tick_count) + 1;\n        teamMap(boss_flag) := @anim(jab);\n    }\n}\n"
        }
    }));
    let def = Arc::new(compile_native(&nc, 1, true));
    let other = Arc::new(compile_native(&base_with(json!({})), 1, true));
    let mut m = duel(def, other);
    m.fighters[1].power = 800.0;
    ticks(&mut m, 3);

    assert!(
        m.fighters[0].has_assert(asf::NOLIFEBARDISPLAY),
        "boss hid life bar"
    );
    assert!(m.global_asserts & gsf::NOTIMEDISPLAY != 0, "boss hid timer");
    assert!(
        m.texts.iter().any(|t| t.text == "APEX BOSS"),
        "boss UI text emitted"
    );
    let count_before = m.fighters[0].ikemen.map.get("tick_count");
    assert!(count_before >= 2.0, "statedef -4 map() ran every tick");
    assert!(
        m.team_maps[0].get("boss_flag") >= 1000.0,
        "@anim(jab) resolved inside ZSS -4"
    );

    m.pause = Some(PauseState {
        by: m.fighters[1].id,
        time: 5,
        is_super: true,
        ..Default::default()
    });
    m.tick(&[0, 0]);
    assert!(
        m.fighters[0].ikemen.map.get("tick_count") >= count_before + 1.0,
        "-4 ignorehitpause ran during SuperPause"
    );
    m.pause = None;

    m.fighters[0].life = m.fighters[0].life_max * 0.5;
    ticks(&mut m, 2);
    assert_eq!(
        m.fighters[1].power, 0.0,
        "phase one-shot drained enemy power"
    );
    assert!(
        (m.fighters[0].attack_mul - 2.0).abs() < 0.01,
        "phase boosted boss attackMul"
    );
    assert!(
        m.texts.iter().any(|t| t.text == "PHASE TWO"),
        "phase banner appeared"
    );

    m.fighters[1].power = 500.0;
    ticks(&mut m, 3);
    assert_eq!(
        m.fighters[1].power, 500.0,
        "once phase mutator did not re-drain power"
    );
}
