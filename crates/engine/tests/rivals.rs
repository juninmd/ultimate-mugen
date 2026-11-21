use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_native::{compile_native, NativeChar};
use ume_engine::rivals::{ATTACK_MAX, ATTACK_MIN};

fn def(id: &str) -> Arc<CharDef> {
    let nc: NativeChar = serde_json::from_str(
        &std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap(),
    )
    .unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

#[test]
fn kaito_and_akio_have_lines_for_each_other() {
    let (kaito, akio) = (def("kaito"), def("akio"));
    assert_eq!(kaito.rivals.id, "kaito");
    assert!(kaito.rivals.line_against("akio", false).is_some());
    assert!(kaito.rivals.line_against("akio", true).is_some());
    assert!(akio.rivals.line_against("kaito", false).is_some());
    assert!(kaito.rivals.line_against("shion", false).is_none());
}

#[test]
fn matchup_bonus_only_applies_against_the_named_rival() {
    let suzume = def("suzume");
    assert!(suzume.rivals.attack_against("oboro") > 1.0);
    assert_eq!(suzume.rivals.attack_against("kaito"), 1.0);
}

#[test]
fn every_declared_matchup_bonus_stays_in_range() {
    for id in ["kaito", "suzume", "sylene", "bruno", "marlow", "shion", "valdric", "oboro", "akio", "kaname"] {
        let d = def(id);
        for other in ["kaito", "akio", "oboro", "garan", "valdric", "suzume", "sylene", "bruno"] {
            let a = d.rivals.attack_against(other);
            assert!((ATTACK_MIN..=ATTACK_MAX).contains(&a), "{id} vs {other}: {a}");
        }
    }
}

#[test]
fn fighters_without_rivals_are_unchanged() {
    let d = def("jin");
    assert!(d.rivals.is_empty());
    assert_eq!(d.rivals.attack_against("kaito"), 1.0);
}
