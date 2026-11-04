use std::sync::Arc;
use ume_engine::{chardef::CharDef, common::{common_def, flat_stage}, matchs::Match};

#[test]
fn nearest_enemy_respects_alive_distance_and_stable_ties() {
    let mut m = Match::new(common_def(), None, flat_stage());
    let d = Arc::new(CharDef::empty("test"));
    for side in [0, 1, 1, 1] { m.add_player(d.clone(), side, 0, 1); }
    m.fighters[1].pos.x = 20.0;
    m.fighters[2].pos.x = -20.0;
    m.fighters[3].pos.x = 1.0;
    m.fighters[3].alive = false;
    assert_eq!(m.enemy(0, 0), Some(1));
    assert_eq!(m.enemy(0, 1), Some(2));
    m.fighters[1].destroyed = true;
    assert_eq!(m.enemy(0, 0), Some(2));
    m.fighters[2].pos.x = f32::NAN;
    assert_eq!(m.enemy(0, 0), Some(2));
}

#[test]
fn nearest_enemy_matches_sorted_selection_in_team_matches() {
    let mut m = Match::new(common_def(), None, flat_stage());
    let d = Arc::new(CharDef::empty("test"));
    for side in [0, 1, 1, 1, 1, 1] { m.add_player(d.clone(), side, 0, 1); }
    for seed in 0..500 {
        for i in 1..6 {
            m.fighters[i].pos.x = (((seed * (i * 17 + 1)) % 53) as f32) - 26.0;
            m.fighters[i].alive = (seed + i) % 4 != 0;
            m.fighters[i].destroyed = (seed + i * 3) % 7 == 0;
        }
        let expected = (0..5).filter_map(|n| m.enemy(0, n)).next();
        let mut candidates: Vec<_> = (1..6).filter(|&i| !m.fighters[i].destroyed).collect();
        candidates.sort_by(|&a, &b| (!m.fighters[a].alive).cmp(&!m.fighters[b].alive)
            .then_with(|| m.fighters[a].pos.x.abs().total_cmp(&m.fighters[b].pos.x.abs())));
        assert_eq!(expected, candidates.first().copied(), "seed {seed}");
    }
}
