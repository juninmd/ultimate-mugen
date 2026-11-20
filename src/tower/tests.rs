use super::*;

#[test]
fn civil_date_matches_known_epoch_dates() {
    assert_eq!(daily::civil_from_days(0), (1970, 1, 1));
    assert_eq!(daily::civil_from_days(20_000), (2024, 10, 4));
}

#[test]
fn same_day_produces_identical_tower_and_different_day_changes() {
    let roster: Vec<usize> = (0..12).collect();
    let a1 = TowerRun::generate("2026-11-20".into(), &roster, 15, Some(11), 0, 4);
    let a2 = TowerRun::generate("2026-11-20".into(), &roster, 15, Some(11), 0, 4);
    let b1 = TowerRun::generate("2026-11-21".into(), &roster, 15, Some(11), 0, 4);
    assert_eq!(a1, a2);
    assert_ne!(a1.seed, b1.seed);
    assert_ne!(a1.floors, b1.floors);
    assert_eq!(a1.floors.len(), TOWER_FLOORS);
    assert!(a1.floors.last().unwrap().is_boss);
    assert_eq!(a1.floors.last().unwrap().modifier, Modifier::BossOverdrive);
}

#[test]
fn clearing_all_floors_reaches_victory_and_awards_bonus() {
    let roster: Vec<usize> = (0..8).collect();
    let mut run = TowerRun::generate("2026-11-20".into(), &roster, 10, Some(7), 0, 4);
    for floor in 0..(TOWER_FLOORS - 1) {
        assert!(!run.won_floor(500));
        assert_eq!(run.current, floor + 1);
        assert_eq!(run.phase, TowerPhase::Ladder);
    }
    assert!(run.won_floor(1000));
    assert_eq!(run.phase, TowerPhase::Victory);
    assert_eq!(run.wins, TOWER_FLOORS);
    assert!(run.score >= BOSS_CLEAR_BONUS);
}

#[test]
fn losing_allows_retrying_the_current_floor() {
    let roster: Vec<usize> = (0..6).collect();
    let mut run = TowerRun::generate("2026-11-20".into(), &roster, 8, Some(5), 0, 4);
    run.won_floor(400);
    run.lost_floor(100);
    assert_eq!(run.phase, TowerPhase::Defeated);
    assert_eq!(run.current, 1);
    run.retry_floor();
    assert_eq!(run.phase, TowerPhase::Ladder);
    assert_eq!(run.current, 1);
}
