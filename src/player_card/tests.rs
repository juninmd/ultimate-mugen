use super::*;
use super::record::SETS_KEPT;

#[test]
fn rank_moves_through_thresholds() {
    assert_eq!(rank(0).index, 0);
    assert_eq!(rank(39).index, 0);
    assert_eq!(rank(40).index, 1);
    let mid = rank(80);
    assert_eq!(mid.index, 1);
    assert!((mid.progress - 0.5).abs() < 1e-6);
    assert_eq!(rank(9999).index, 6);
    assert!(rank(9999).next.is_none());
}

#[test]
fn sets_update_streaks_and_roster() {
    let mut r = Record::default();
    r.apply_set(true, "kaito", "garan");
    r.apply_set(true, "kaito", "volta");
    r.apply_set(false, "sylene", "garan");
    r.apply_set(true, "kaito", "garan");
    assert_eq!(r.matches, 4);
    assert_eq!(r.wins, 3);
    assert_eq!(r.streak, 1);
    assert_eq!(r.best_streak, 2);
    assert_eq!(r.xp, 10 + 10 + 3 + 10);
    assert_eq!(r.top_roster(1), vec![("kaito", 3)]);
    assert_eq!(r.rivals["garan"], [2, 1]);
    assert_eq!(r.sets.len(), 4);
    assert!(r.sets[0].won);
}

#[test]
fn sets_history_is_capped() {
    let mut r = Record::default();
    for _ in 0..(SETS_KEPT + 5) {
        r.apply_set(true, "kaito", "garan");
    }
    assert_eq!(r.sets.len(), SETS_KEPT);
}

#[test]
fn achievements_level_by_tiers() {
    let mut r = Record::default();
    for _ in 0..10 {
        r.apply_set(true, "kaito", "garan");
    }
    let list = achievements(&r);
    assert_eq!(list[0].level(), 2);
    assert_eq!(list[0].next_tier(), Some(50));
    assert_eq!(list[2].level(), 3);
    assert_eq!(list[4].level(), 1);
}

#[test]
fn broken_saved_text_falls_back_to_empty_record() {
    assert_eq!(Record::parse("{not json"), Record::default());
    let card = Record::parse(r#"{"card":{"name":"Ana"}}"#).card;
    assert_eq!(card.name, "Ana");
    assert_eq!(card.flag, "BR");
}
