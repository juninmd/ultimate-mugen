use super::model::{Tuning, DEFENCE, LIFE, POWER, WALK};
use super::store::{parse, render, Map};

#[test]
fn round_trip_keeps_only_changed_params() {
    let mut t = Tuning::default();
    t.0[LIFE] = 150;
    t.0[WALK] = 80;
    let mut map = Map::new();
    map.insert("native:sylene".into(), t);
    map.insert("umf:neutral".into(), Tuning::default());
    let text = render(&map);
    assert!(!text.contains("neutral"));
    let back = parse(&text);
    assert_eq!(back.get("native:sylene"), Some(&t));
    assert_eq!(back.len(), 1);
}

#[test]
fn out_of_range_values_are_clamped_and_bad_text_is_empty() {
    let map = parse("[\"k\"]\nlife = 9999\npower = -4\n");
    let t = map.get("k").copied().unwrap();
    assert_eq!(t.0[LIFE], 300);
    assert_eq!(t.0[POWER], 0);
    assert!(parse("not = [valid").is_empty());
}

#[test]
fn apply_scales_life_and_publishes_defence() {
    let mut d = ume_engine::chardef::CharDef::empty("probe");
    d.constants.insert("data.life".into(), 1000.0);
    let mut t = Tuning::default();
    t.0[LIFE] = 150;
    t.0[DEFENCE] = 80;
    t.apply(&mut d);
    assert_eq!(d.constants["data.life"], 1500.0);
    assert_eq!(d.constants["tuning.defence"], 80.0);
}
