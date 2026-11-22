use super::*;
use crate::compile::extras::hitdef_from_spec;
use crate::types::HitSpec;
use serde_json::json;

fn mv(extra: serde_json::Value) -> NativeMove {
    let mut base = json!({ "id": "grenade", "name": "Grenade", "input": "~D, DB, B, x", "kind": "special", "anim": "toss" });
    base.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    serde_json::from_value(base).unwrap()
}

#[test]
fn spawn_starts_the_helper_on_its_frame_with_the_original_parameters() {
    let m = mv(json!({ "spawn": [{ "at": 2, "state": 70005, "id": 21100, "pos": [44, -6], "ownPal": 1 }, { "state": 70006 }] }));
    let cns = spawn_cns(&m, 1009);
    assert!(cns.contains("[State 1009, spawn]\ntype = Helper\ntrigger1 = AnimElem = 2\nstateno = 70005\nID = 21100\npos = 44, -6\nownpal = 1\n"), "{cns}");
    assert!(cns.contains("trigger1 = Time = 0\nstateno = 70006\n"), "a spawn without a frame fires on the first tick: {cns}");
}

#[test]
fn own_sound_plays_the_hit_sound_from_the_fighter_bank() {
    let own: HitSpec = serde_json::from_value(json!({ "at": 1, "damage": 30, "hitSound": [172, 13], "ownSound": true })).unwrap();
    let common: HitSpec = serde_json::from_value(json!({ "at": 1, "damage": 30, "hitSound": [5, 2] })).unwrap();
    assert!(hitdef_from_spec(&own, 1.0, "S, NA", false).contains("hitsound = S172, 13\n"));
    assert!(hitdef_from_spec(&common, 1.0, "S, NA", false).contains("hitsound = 5, 2\n"));
}
