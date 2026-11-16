use super::*;
use serde_json::json;

fn ids(value: &Value) -> Vec<String> {
    value["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn plain_definitions_pass_through_untouched() {
    let plain = json!({ "id": "a", "moves": [{ "id": "jab", "input": "x" }] });
    assert_eq!(expand(plain.clone()).unwrap(), plain);
}

#[test]
fn extends_brings_kit_moves_and_keeps_own_moves_first() {
    let fighter = json!({ "id": "a", "extends": "core", "moves": [{ "id": "jab", "input": "x" }] });
    let expanded = expand(fighter).unwrap();
    assert_eq!(ids(&expanded), ["jab", "launcher", "escape_burst"]);
    assert_eq!(expanded["format"], "umf/1");
    assert!(expanded.get("extends").is_none());
}

#[test]
fn a_listed_move_overrides_only_the_fields_it_names() {
    let fighter = json!({ "id": "a", "extends": "core", "moves": ["escape_burst", { "id": "launcher", "name": "Sky Hook", "cost": null, "when": "stand" }] });
    let expanded = expand(fighter).unwrap();
    assert_eq!(ids(&expanded), ["escape_burst", "launcher"]);
    let launcher = &expanded["moves"][1];
    assert_eq!(
        (
            launcher["name"].as_str(),
            launcher["when"].as_str(),
            launcher["anim"].as_str()
        ),
        (Some("Sky Hook"), Some("stand"), Some("strong"))
    );
    assert_eq!(launcher["hits"][0]["damage"], 58);
}

#[test]
fn use_clones_a_move_under_a_new_id_and_remove_drops_one() {
    let fighter = json!({
        "id": "a",
        "extends": "core",
        "moves": [
            { "id": "ex_launcher", "use": "launcher", "input": "x+y", "cost": 500 },
            { "id": "ex_launcher_2", "use": "ex_launcher", "cost": 1000 },
            { "id": "escape_burst", "remove": true }
        ]
    });
    let expanded = expand(fighter).unwrap();
    assert_eq!(ids(&expanded), ["ex_launcher", "ex_launcher_2", "launcher"]);
    assert_eq!(expanded["moves"][1]["input"], "x+y");
    assert_eq!(expanded["moves"][1]["cost"], 1000);
    assert!(expanded["moves"][0].get("use").is_none());
}

#[test]
fn objects_merge_deeply_and_animations_replace_whole() {
    register_kit(r#"{"format":"umf-kit/1","id":"test_base","stats":{"life":900,"walk":[2,2]},"art":{"type":"rig","poses":{"a":{"x":1},"b":{"x":2}}},"anims":{"idle":{"frames":[{"pose":"a","ticks":4},{"pose":"b","ticks":4}]},"walk":{"frames":[{"pose":"a","ticks":2}]}}}"#).unwrap();
    let fighter = json!({
        "id": "a",
        "extends": ["core", "test_base"],
        "stats": { "life": 1100 },
        "art": { "poses": { "b": { "x": 9 } } },
        "anims": { "idle": { "frames": [{ "pose": "b", "ticks": 6 }] }, "walk": null }
    });
    let expanded = expand(fighter).unwrap();
    assert_eq!(expanded["stats"], json!({ "life": 1100, "walk": [2, 2] }));
    assert_eq!(
        expanded["art"]["poses"],
        json!({ "a": { "x": 1 }, "b": { "x": 9 } })
    );
    assert_eq!(
        expanded["anims"],
        json!({ "idle": { "frames": [{ "pose": "b", "ticks": 6 }] } })
    );
    assert_eq!(ids(&expanded), ["launcher", "escape_burst"]);
}

#[test]
fn mistakes_are_reported_with_the_offending_name() {
    assert!(expand(json!({ "id": "a", "extends": "nope" }))
        .unwrap_err()
        .contains("nope"));
    assert!(
        expand(json!({ "id": "a", "moves": [{ "id": "x", "use": "ghost" }] }))
            .unwrap_err()
            .contains("ghost")
    );
    assert!(
        expand(json!({ "id": "a", "extends": "core", "moves": ["ghost"] }))
            .unwrap_err()
            .contains("ghost")
    );
    assert!(register_kit(r#"{"format":"umf/1","id":"x"}"#).is_err());
    register_kit(r#"{"format":"umf-kit/1","id":"loop_a","extends":"loop_a"}"#).unwrap_err();
}

#[test]
fn the_built_in_kit_is_listed() {
    assert!(kit_ids().contains(&"core".to_string()));
}
