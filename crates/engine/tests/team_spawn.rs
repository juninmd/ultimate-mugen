use crate::support;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;

fn squad(per_side: usize, simul: bool, seed: u32) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = simul;
    m.rng = seed;
    for side in 0..2 {
        for k in 0..per_side {
            m.add_player(mugen_char(&format!("f{side}{k}"), ""), side, 0, 1);
        }
    }
    m.set_zoom_for_teams(per_side, 0.5);
    m.start_round();
    m
}

fn xs(m: &Match) -> Vec<f32> {
    m.roots().map(|f| f.pos.x).collect()
}

#[test]
fn seven_a_side_spawns_spread_out_and_apart() {
    let m = squad(7, true, 12345);
    let mut sorted = xs(&m);
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(sorted.len(), 14);
    for pair in sorted.windows(2) {
        assert!(pair[1] - pair[0] >= 8.0, "fighters never spawn on top of each other: {sorted:?}");
    }
    let (lo, hi) = (m.stage.bound_left, m.stage.bound_right);
    assert!(sorted.iter().all(|&x| x >= lo && x <= hi), "spawns stay inside the playable stage");
}

#[test]
fn spawns_are_random_but_reproducible_from_the_seed() {
    assert_eq!(xs(&squad(5, true, 7)), xs(&squad(5, true, 7)), "same seed, same spawn (replays and rollback)");
    assert_ne!(xs(&squad(5, true, 7)), xs(&squad(5, true, 99)), "another seed gives another layout");
}

#[test]
fn everyone_faces_the_opposing_team() {
    let m = squad(4, true, 31);
    let centre = |side: usize| {
        let v: Vec<f32> = m.roots().filter(|f| f.side == side).map(|f| f.pos.x).collect();
        v.iter().sum::<f32>() / v.len() as f32
    };
    for f in m.roots() {
        let foe = centre(1 - f.side);
        assert_eq!(f.facing > 0.0, foe >= f.pos.x, "{} faces its foes", f.def.name);
    }
}

#[test]
fn classic_one_on_one_keeps_the_fixed_spots() {
    let m = squad(1, false, 5);
    let (a, b) = (m.roots().next().unwrap().pos.x, m.roots().nth(1).unwrap().pos.x);
    assert_eq!((a, b), (m.stage.p1x, m.stage.p2x));
}
