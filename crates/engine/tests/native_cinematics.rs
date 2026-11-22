mod support;
use serde_json::json;
use std::sync::Arc;
use support::*;
use ume_engine::diagnostics::audit_cns;
use ume_engine::flags::gsf;
use ume_engine::matchs::Event;
use ume_native::{compile_native, NativeChar};

fn base_char(patch: serde_json::Value) -> NativeChar {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    v["id"] = json!("cinematic_lab");
    v["abilities"] = serde_json::Value::Null;
    v["passives"] = serde_json::Value::Null;
    if let Some(obj) = patch.as_object() {
        for (k, val) in obj {
            v[k] = val.clone();
        }
    }
    serde_json::from_value(v).unwrap()
}

#[test]
fn move_cinematic_spec_drives_camera_ui_texts_slowmo_and_events() {
    let nc = base_char(json!({
        "anims": {
            "idle": {"frames": [{"pose": "idle0", "ticks": -1}]},
            "cburst": {"frames": [
                {"pose": "jab0", "ticks": 4},
                {"pose": "jab1", "ticks": 4, "hit": {"part": "handF", "radius": 24}},
                {"pose": "str1", "ticks": 8},
                {"pose": "idle0", "ticks": 4}
            ]}
        },
        "moves": [{
            "id": "c_super",
            "name": "Tempest Finale",
            "input": "x",
            "kind": "special",
            "anim": "cburst",
            "hits": [{"at": 2, "damage": 65, "level": "heavy"}],
            "cinematic": {
                "letterbox": true,
                "hideUi": true,
                "title": "TEMPEST FINALE",
                "camera": {"zoom": 1.35, "pos": [12, -20], "delta": 0.01, "delay": 1, "time": 24, "bound": false},
                "finishSlow": 18,
                "cuts": [
                    {"at": 2, "shake": 8, "slowmo": 12, "invert": true, "lines": 0.9, "flash": {"time": 4, "color": [255, 220, 120]}, "text": "CLIMAX HIT"}
                ]
            }
        }]
    }));
    let def = Arc::new(compile_native(&nc, 1, false));
    let other = Arc::new(compile_native(&base_char(json!({})), 1, false));
    let sid = def
        .movelist
        .iter()
        .find(|mv| mv.name == "Tempest Finale")
        .unwrap()
        .state;
    let mut m = duel(def, other);
    face_off(&mut m, 44.0);
    enter(&mut m, 0, sid);

    let mut saw_hide_ui = false;
    let mut saw_zoom_req = false;
    let mut saw_cinema_invert = false;
    let mut saw_slowmo = false;
    let mut saw_title = false;
    let mut saw_cut_text = false;

    for _ in 0..45 {
        m.tick(&[0, 0]);
        if (m.global_asserts & gsf::NOBARDISPLAY) != 0 {
            saw_hide_ui = true;
        }
        if let Some(zr) = &m.zoom_request {
            if (zr.scale - 1.35).abs() < 0.1 && !zr.bound {
                saw_zoom_req = true;
            }
        }
        if m.slow_mo > 0 {
            saw_slowmo = true;
        }
        if m.texts.iter().any(|t| t.text == "TEMPEST FINALE") {
            saw_title = true;
        }
        if m.texts.iter().any(|t| t.text == "CLIMAX HIT") {
            saw_cut_text = true;
        }
        for ev in m.events.drain(..) {
            if let Event::Cinema {
                bars,
                invert,
                lines,
                ..
            } = ev
            {
                if bars >= 0.9 && invert && lines >= 0.8 {
                    saw_cinema_invert = true;
                }
            }
        }
    }
    assert!(saw_hide_ui, "cinematic hideUi asserted NoBarDisplay");
    assert!(
        saw_zoom_req,
        "cinematic camera emitted free ZoomRequest with delta"
    );
    assert!(saw_slowmo, "cinematic cut activated deterministic slow_mo");
    assert!(
        saw_cinema_invert,
        "cinematic cut emitted Event::Cinema with invert and speed lines"
    );
    assert!(saw_title, "cinematic title rendered in m.texts");
    assert!(saw_cut_text, "cinematic cut caption rendered in m.texts");
}

#[test]
fn throw_steps_switch_attacker_anim_and_emit_per_beat_camera() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    v["id"] = json!("cinematic_throw_steps");
    v["abilities"] = serde_json::Value::Null;
    v["passives"] = serde_json::Value::Null;
    v["anims"]["p2slam"] = json!({"id": 8801, "frames": [{"pose": "hitH2", "ticks": -1}]});
    v["anims"]["stepflip"] = json!({"id": 8802, "frames": [{"pose": "rise1", "ticks": 12}]});
    v["anims"]["grabseq"] = json!({"frames": [
        {"pose": "jab1", "ticks": 3, "hit": {"part": "handF", "radius": 30}},
        {"pose": "jab1", "ticks": 6}, {"pose": "jab0", "ticks": 6}, {"pose": "str1", "ticks": 8}
    ]});
    v["moves"] = json!([{
        "id": "cthrow",
        "name": "Spiral Slam",
        "input": "x+a",
        "kind": "normal",
        "anim": "grabseq",
        "throw": {
            "damage": 90,
            "at": 1,
            "p2Anim": "p2slam",
            "cameraFree": true,
            "camera": {"zoom": 1.2, "time": 20},
            "steps": [
                {"at": 2, "bind": [28, -32], "anim": "stepflip", "damage": 40, "camera": {"zoom": 1.55, "pos": [10, -30], "time": 16}, "slowmo": 8, "invert": true, "lines": 0.9},
                {"at": 3, "bind": [36, -6], "damage": 50, "launch": [-4, -7], "shake": 10}
            ]
        }
    }]);
    let nc: NativeChar = serde_json::from_value(v).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    let other = Arc::new(compile_native(&base_char(json!({})), 1, false));
    let mut m = duel(def, other);
    face_off(&mut m, 34.0);
    enter(&mut m, 0, 200);

    let mut saw_step_anim = false;
    let mut saw_step_zoom = false;
    let mut saw_step_cinema = false;
    for _ in 0..45 {
        m.tick(&[0, 0]);
        if m.fighters[0].anim_no == 8802 {
            saw_step_anim = true;
        }
        if m.zoom_request.as_ref().is_some_and(|z| z.scale >= 1.5) {
            saw_step_zoom = true;
        }
        for ev in m.events.drain(..) {
            if let Event::Cinema {
                invert: true, slow, ..
            } = ev
            {
                if slow >= 8 {
                    saw_step_cinema = true;
                }
            }
        }
    }
    assert!(saw_step_anim, "throw step switched attacker anim to 8802");
    assert!(saw_step_zoom, "throw step pushed per-beat close-up Zoom");
    assert!(
        saw_step_cinema,
        "throw step fired Event::Cinema slowmo + invert"
    );
}

#[test]
fn cinema_and_slowmo_controllers_pass_strict_diagnostics() {
    let cns = "\
[Statedef 210]\n\
type = S\n\
movetype = A\n\
physics = S\n\
anim = 200\n\
[State 210, 1]\n\
type = Cinema\n\
trigger1 = Time = 0\n\
bars = 1.0\n\
slow = 12\n\
invert = 1\n\
lines = 0.8\n\
flash = 255, 240, 200\n\
[State 210, 2]\n\
type = SlowMo\n\
trigger1 = Time = 2\n\
time = 15\n";
    let diag = audit_cns("test_cinema.cns", cns);
    assert!(
        diag.is_clean(),
        "expected clean diagnostics, got: {}",
        diag.detail()
    );
}
