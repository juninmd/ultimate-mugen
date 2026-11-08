//! The optional physical-detail style keys (muscle, folds, stitching, boots, face...) must be
//! deterministic, must not change collision data or sprite size, and must actually change the image.
use serde_json::{json, Value};
use ume_engine::native::rig::render_pose;

fn style(extra: Value) -> Value {
    let mut s = json!({ "build": 1.1, "top": "bare", "pants": "trunks", "gloves": "wrap", "boots": "tall", "belt": "accent", "hair": { "type": "short" } });
    if let (Some(a), Some(b)) = (s.as_object_mut(), extra.as_object()) {
        for (k, v) in b {
            a.insert(k.clone(), v.clone());
        }
    }
    s
}

#[test]
fn detail_keys_are_deterministic_and_geometry_neutral() {
    let pose = json!({ "shoulderF": 40, "elbowF": 100, "thighF": 20, "kneeF": 24, "lean": 6 });
    let plain = render_pose(&style(json!({})), &pose, None, true, None);
    let rich = style(json!({
        "muscle": 1.2, "veins": true, "folds": 1.0, "stitch": true, "weave": "hatch", "knuckles": true, "faceShade": 1.0,
        "beard": ["stubble", "moustache"], "glasses": { "type": "round" }, "earring": true, "buckle": "round",
        "footwear": { "sole": true, "laces": true, "buckles": 2 }, "tattoo": { "type": "tribal", "on": ["arm", "back"] },
        "hair": { "type": "short", "strands": 6, "locks": 3, "shine": true }, "arms": 1.1, "chest": 1.08, "waist": 0.92,
    }));
    let a = render_pose(&rich, &pose, None, true, None);
    let b = render_pose(&rich, &pose, None, true, None);
    assert_eq!(
        a.image.as_ref().unwrap().rgba,
        b.image.as_ref().unwrap().rgba,
        "deterministic"
    );
    assert_ne!(
        a.image.as_ref().unwrap().rgba,
        plain.image.as_ref().unwrap().rgba,
        "details change the picture"
    );
    assert_eq!(
        (a.width, a.height, a.axis_x, a.axis_y),
        (plain.width, plain.height, plain.axis_x, plain.axis_y),
        "sprite size unchanged"
    );
    assert_eq!(a.hurt, plain.hurt, "hurt boxes unchanged");
}

#[test]
fn absent_keys_render_like_before() {
    let pose = json!({});
    let a = render_pose(&style(json!({})), &pose, None, true, None);
    let b = render_pose(
        &style(json!({ "muscle": 0, "folds": 0, "arms": 1, "legs": 1, "chest": 1, "waist": 1 })),
        &pose,
        None,
        true,
        None,
    );
    assert_eq!(
        a.image.unwrap().rgba,
        b.image.unwrap().rgba,
        "zero / neutral values are a no-op"
    );
}
