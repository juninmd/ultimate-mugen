mod support;
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn def(id: &str) -> Arc<CharDef> {
    let nc: NativeChar = serde_json::from_str(
        &std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap(),
    )
    .unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

fn duel(a: &str, b: &str) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def(a), 0, 0, 1);
    m.add_player(def(b), 1, 0, 2);
    m.start_round();
    m
}

fn cast(m: &mut Match, i: usize, state: i32) {
    m.fighters[i].state_no = state;
    m.fighters[i].tick_passive(true);
}

#[test]
fn kaito_builds_storm_from_specials_and_gets_stronger() {
    let mut m = duel("kaito", "volta");
    assert_eq!(m.fighters[0].signature_attack(), 1.0);
    for _ in 0..3 {
        cast(&mut m, 0, 1000);
        cast(&mut m, 0, 0);
    }
    let view = m.fighters[0].signature_view().unwrap();
    assert_eq!(view.tiers, 3);
    assert!(view.tier >= 1, "{view:?}");
    assert!(m.fighters[0].signature_attack() > 1.0);
    assert!(m.fighters[0].passive_speed() > 1.0);
}

#[test]
fn storm_decays_when_idle_and_resets_each_round() {
    let mut m = duel("kaito", "volta");
    for _ in 0..4 {
        cast(&mut m, 0, 1000);
        cast(&mut m, 0, 0);
    }
    let peak = m.fighters[0].passive.signature.meter;
    assert!(peak > 0.0);
    for _ in 0..600 {
        m.fighters[0].tick_passive(true);
    }
    assert!(m.fighters[0].passive.signature.meter < peak);
    cast(&mut m, 0, 1000);
    m.fighters[0].reset_passive_for_round();
    assert_eq!(m.fighters[0].passive.signature.meter, 0.0);
}

#[test]
fn super_spends_the_storm() {
    let mut m = duel("kaito", "volta");
    for _ in 0..4 {
        cast(&mut m, 0, 1000);
        cast(&mut m, 0, 0);
    }
    cast(&mut m, 0, 3000);
    assert!(m.fighters[0].passive.signature.meter > 0.0);
    cast(&mut m, 0, 0);
    assert_eq!(m.fighters[0].passive.signature.meter, 0.0);
}

#[test]
fn volta_overload_costs_life_at_the_top_but_never_kills() {
    let mut m = duel("volta", "kaito");
    for _ in 0..12 {
        cast(&mut m, 0, 1000);
        cast(&mut m, 0, 0);
    }
    assert_eq!(m.fighters[0].passive.signature.meter, 100.0);
    assert!(m.fighters[0].signature_attack() >= 1.6);
    let before = m.fighters[0].life;
    for _ in 0..120 {
        m.fighters[0].tick_passive(true);
    }
    assert!(m.fighters[0].life < before);
    m.fighters[0].life = 1.0;
    for _ in 0..600 {
        m.fighters[0].tick_passive(true);
    }
    assert!(m.fighters[0].life >= 1.0);
}

#[test]
fn fighters_without_signature_are_untouched() {
    let m = duel("bruno", "kaito");
    assert!(m.fighters[0].signature_view().is_none());
    assert_eq!(m.fighters[0].signature_attack(), 1.0);
}

#[test]
fn suzume_marlow_jin_and_sylene_declare_their_signatures() {
    for id in ["suzume", "marlow", "jin", "sylene"] {
        let mut m = duel(id, "kaito");
        for _ in 0..12 {
            cast(&mut m, 0, 1000);
            cast(&mut m, 0, 0);
        }
        let view = m.fighters[0].signature_view().unwrap_or_else(|| panic!("{id} has no signature"));
        assert!(view.tiers >= 3 && view.tier >= 1, "{id}: {view:?}");
        let (a, d, s) = (m.fighters[0].signature_attack(), m.fighters[0].signature_defence(), m.fighters[0].passive_speed());
        assert!(a > 1.0 || d > 1.0 || s > 1.0, "{id} gets no bonus from its signature");
    }
}
