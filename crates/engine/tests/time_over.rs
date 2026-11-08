mod support;
use support::*;

fn timed_duel(ticks_total: i32) -> ume_engine::matchs::Match {
    let a = mugen_char("a", "");
    let b = mugen_char("b", "");
    let mut m = duel(a, b);
    m.round_ticks = ticks_total;
    m.timer = ticks_total;
    m
}

fn run_until(m: &mut ume_engine::matchs::Match, state: i32, round: i32) {
    for _ in 0..3000 {
        m.tick(&[0, 0]);
        if m.round_state == state && m.round_no == round {
            return;
        }
    }
    panic!("never reached state {state} round {round}");
}

#[test]
fn time_over_announces_and_resolves_the_round() {
    let mut m = timed_duel(30);
    run_until(&mut m, 3, 1);
    assert_eq!(m.announce.as_ref().unwrap().text, "TIME OVER");
    assert!(!m.won_by_ko);
    run_until(&mut m, 0, 2);
    assert_eq!(m.round_no, 2);
}

#[test]
fn higher_life_wins_on_time_over() {
    let mut m = timed_duel(30);
    m.fighters[1].life = m.fighters[1].life_max * 0.5;
    run_until(&mut m, 3, 1);
    assert_eq!(m.winner_side, 0);
    run_until(&mut m, 4, 1);
    assert_eq!(m.wins, [1, 0]);
}

#[test]
fn draw_on_time_over_credits_both_without_ending_the_match() {
    let mut m = timed_duel(30);
    m.rounds_to_win = 2;
    run_until(&mut m, 3, 1);
    assert_eq!(m.winner_side, -2);
    run_until(&mut m, 4, 1);
    assert_eq!(m.wins, [1, 1]);
    run_until(&mut m, 0, 2);
    assert!(!m.match_over);
}

#[test]
fn infinite_time_never_times_out() {
    let mut m = timed_duel(0);
    for _ in 0..4000 {
        m.tick(&[0, 0]);
    }
    assert_eq!(m.round_state, 2);
}

#[test]
fn controls_return_after_time_over() {
    let mut m = timed_duel(30);
    run_until(&mut m, 0, 2);
    run_until(&mut m, 2, 2);
    assert!(m
        .fighters
        .iter()
        .filter(|f| !f.is_helper)
        .all(|f| f.ctrl && f.alive));
    assert_eq!(m.timer, 30);
}
