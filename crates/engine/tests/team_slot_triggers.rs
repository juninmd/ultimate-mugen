mod support;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

fn ev(m: &Match, i: usize, src: &str) -> f64 { m.ev(i, &Expr::new(parse(src).unwrap())) }

fn roster(sides: &[usize], simul: bool) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = simul;
    for (slot, side) in sides.iter().enumerate() { m.add_player(mugen_char(&format!("fighter{}", slot + 1), ""), *side, 0, 1); }
    m.start_round();
    m
}

fn run_to_fight(m: &mut Match) {
    let n = m.root_indices().len();
    while m.round_state != 2 { m.tick(&vec![0; n]); }
}

#[test]
fn one_versus_one_reports_single_slots() {
    let mut m = roster(&[0, 1], false);
    run_to_fight(&mut m);
    assert_eq!(ev(&m, 0, "NumPlayer"), 2.0);
    assert_eq!(ev(&m, 0, "TeamSize"), 1.0);
    assert_eq!(ev(&m, 1, "TeamSize"), 1.0);
    assert_eq!(ev(&m, 0, "PlayerNo"), 1.0);
    assert_eq!(ev(&m, 1, "PlayerNo"), 2.0);
    assert_eq!(ev(&m, 0, "MemberNo"), 1.0);
    assert_eq!(ev(&m, 1, "MemberNo"), 1.0);
    assert_eq!(ev(&m, 0, "TeamLeader"), 1.0);
    assert_eq!(ev(&m, 1, "TeamLeader"), 2.0);
    assert_eq!(ev(&m, 0, "PlayerIndexExist(0)"), 1.0);
    assert_eq!(ev(&m, 0, "PlayerIndexExist(1)"), 1.0);
    assert_eq!(ev(&m, 0, "PlayerIndexExist(2)"), 0.0);
    assert_eq!(ev(&m, 0, "PlayerIndexExist(-1)"), 0.0);
    assert_eq!(ev(&m, 0, "LastPlayerID"), m.fighters[1].id as f64);
}

#[test]
fn parallel_five_versus_five_numbers_every_member() {
    let sides: Vec<usize> = (0..10).map(|k| k % 2).collect();
    let mut m = roster(&sides, true);
    run_to_fight(&mut m);
    assert_eq!(ev(&m, 0, "NumPlayer"), 10.0);
    for viewer in 0..10 {
        assert_eq!(ev(&m, viewer, "TeamSize"), 5.0);
        assert_eq!(ev(&m, viewer, "PlayerNo"), (viewer + 1) as f64);
        assert_eq!(ev(&m, viewer, "MemberNo"), (viewer / 2 + 1) as f64);
        assert_eq!(ev(&m, viewer, "TeamLeader"), (viewer % 2 + 1) as f64);
    }
    assert_eq!(ev(&m, 9, "PlayerIndexExist(9)"), 1.0);
    assert_eq!(ev(&m, 9, "PlayerIndexExist(10)"), 0.0);
    assert_eq!(ev(&m, 0, "LastPlayerID"), m.fighters[9].id as f64);
}

#[test]
fn sequence_three_versus_three_follows_the_fielded_member() {
    let mut m = roster(&[0, 1], false);
    m.team_turns = true;
    m.team_size = [3, 3];
    m.team_member = [1, 2];
    run_to_fight(&mut m);
    assert_eq!(ev(&m, 0, "NumPlayer"), 6.0);
    assert_eq!(ev(&m, 0, "TeamSize"), 3.0);
    assert_eq!(ev(&m, 1, "TeamSize"), 3.0);
    assert_eq!(ev(&m, 0, "MemberNo"), 2.0);
    assert_eq!(ev(&m, 1, "MemberNo"), 3.0);
    assert_eq!(ev(&m, 0, "PlayerNo"), 3.0);
    assert_eq!(ev(&m, 1, "PlayerNo"), 6.0);
    assert_eq!(ev(&m, 0, "TeamLeader"), 1.0);
    assert_eq!(ev(&m, 1, "TeamLeader"), 2.0);
}

#[test]
fn helpers_report_their_roots_slot() {
    let mut m = roster(&[0, 1, 0, 1], true);
    run_to_fight(&mut m);
    let before = m.next_id;
    let mut h = m.fighters[2].clone();
    h.id = m.next_id; m.next_id += 1;
    h.is_helper = true; h.root = m.fighters[2].id;
    m.fighters.push(h);
    assert_eq!(ev(&m, 4, "PlayerNo"), 3.0);
    assert_eq!(ev(&m, 4, "MemberNo"), 2.0);
    assert_eq!(ev(&m, 4, "TeamSize"), 2.0);
    assert_eq!(ev(&m, 0, "LastPlayerID"), before as f64);
    assert_eq!(ev(&m, 0, "PlayerIndexExist(4)"), 1.0);
}

#[test]
fn intro_and_outro_state_follow_the_round_phase() {
    let mut m = roster(&[0, 1], false);
    assert_eq!(ev(&m, 0, "IntroState"), 1.0);
    assert_eq!(ev(&m, 0, "OutroState"), 0.0);
    while m.round_state < 1 { m.tick(&[0, 0]); }
    assert_eq!(ev(&m, 0, "IntroState"), 2.0);
    run_to_fight(&mut m);
    assert_eq!(ev(&m, 0, "IntroState"), 0.0);
    assert_eq!(ev(&m, 0, "OutroState"), 0.0);
    m.fighters[1].life = 0.0;
    while m.round_state != 3 { m.tick(&[0, 0]); }
    assert_eq!(ev(&m, 0, "OutroState"), 1.0);
    while m.round_state != 4 { m.tick(&[0, 0]); }
    assert_eq!(ev(&m, 0, "OutroState"), 2.0);
    assert_eq!(ev(&m, 0, "IntroState"), 0.0);
}
