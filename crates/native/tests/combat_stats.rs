use serde_json::json;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::element::Element;
use ume_engine::matchs::Match;
use ume_native::{compile_native, parse_char};

#[test]
fn guard_dizzy_weight_and_elemental_affinity_compile_into_engine() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
    v["id"] = json!("stats_test_fighter");
    v.as_object_mut().unwrap().remove("element");
    v["stats"]["guardMax"] = json!(1400);
    v["stats"]["dizzyMax"] = json!(1250);
    v["stats"]["weight"] = json!(1.2);
    v["stats"]["chipDefense"] = json!(0.75);
    v["profile"] = json!({
        "country": "jp",
        "element": "fire",
        "weakness": "water",
        "alignment": "hero"
    });
    let nc = parse_char(&v.to_string()).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    assert_eq!(def.element, Some(Element::Fire));
    assert_eq!(def.weakness, Some(Element::Water));
    assert_eq!(def.c("movement.weight", 1.0), 1.2);
    assert_eq!(def.c("data.chip.defence", 1.0), 0.75);
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    assert_eq!(m.fighters[0].ikemen.guard_max, 1400.0);
    assert_eq!(m.fighters[0].ikemen.dizzy_max, 1250.0);
    assert_eq!(
        Element::advantage_multiplier(Some(Element::Water), m.fighters[0].def.weakness),
        1.10
    );
}
