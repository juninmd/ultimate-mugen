use super::*;
use crate::commands::macros::{input_macro, sequence_matches};
use crate::input::k::*;
fn feed(seq: &[u32], f: i32) -> InputHistory {
    let mut h = InputHistory::default();
    for b in seq {
        h.push(*b, f);
    }
    h
}
#[test]
fn qcf() {
    assert!(match_command(
        &feed(&[0, D, D, D | R, R, R | X], 1),
        &parse_command("q", "~D, DF, F, x", 15, 1)
    ));
}
#[test]
fn wrong() {
    assert!(!match_command(
        &feed(&[0, R, D | R, D, X], 1),
        &parse_command("q", "~D, DF, F, x", 15, 1)
    ));
}
#[test]
fn left() {
    assert!(match_command(
        &feed(&[0, D, D, D | L, L, L | X], -1),
        &parse_command("q", "~D, DF, F, x", 15, 1)
    ));
}
#[test]
fn dash_hold() {
    let h = feed(&[0, R, 0, R], 1);
    assert!(match_command(&h, &parse_command("f", "F, F", 10, 1)));
    assert!(match_command(&h, &parse_command("h", "/$F", 1, 1)));
}
#[test]
fn combo() {
    assert!(match_command(
        &feed(&[0, A, A | B], 1),
        &parse_command("ab", "a+b", 15, 1)
    ));
    let mut v = vec![0];
    v.extend(std::iter::repeat_n(L, 40));
    v.push(R);
    v.push(R | X);
    assert!(match_command(
        &feed(&v, 1),
        &parse_command("c", "~30$B, F, x", 20, 1)
    ));
}
#[test]
fn super_motion() {
    let q = [D, D, D | R, D | R, R, R];
    let mut v = vec![0];
    v.extend(q);
    v.push(0);
    v.extend(&q[..5]);
    v.push(R | X);
    assert!(match_command(
        &feed(&v, 1),
        &parse_command("s", "~D, DF, F, D, DF, F, x", 20, 1)
    ));
}
#[test]
fn neutral_step() {
    assert!(match_command(
        &feed(&[0, R, 0, 0, X], 1),
        &parse_command("n", "N, x", 15, 1)
    ));
    let mut held_forward = vec![0];
    held_forward.extend(std::iter::repeat_n(R, 20));
    held_forward.push(R | X);
    assert!(!match_command(
        &feed(&held_forward, 1),
        &parse_command("n", "N, x", 15, 1)
    ));
    assert!(match_command(
        &feed(&[0, 0, X], 1),
        &parse_command("n", "N, x", 15, 1)
    ));
    assert!(match_command(
        &feed(&[0, D, D | R, 0, X], 1),
        &parse_command("n", "~D, N, x", 15, 1)
    ));
    assert!(match_command(
        &feed(&[0, 0], 1),
        &parse_command("n", "N", 1, 1)
    ));
    assert!(!match_command(
        &feed(&[0, U], 1),
        &parse_command("n", "N", 1, 1)
    ));
}
#[test]
fn neutral_with_button_in_one_step() {
    assert!(match_command(
        &feed(&[0, 0, X], 1),
        &parse_command("n", "N+x", 1, 1)
    ));
    assert!(!match_command(
        &feed(&[0, 0, R | X], 1),
        &parse_command("n", "N+x", 1, 1)
    ));
}
#[test]
fn a_button_that_never_gets_released_stops_matching_until_the_history_clears() {
    let mut h = feed(&[0, D, D, D | R, D | R, R, R | X], 1);
    let qcf = parse_command("q", "~D, DF, F, x", 15, 1);
    let mut st = CommandState::default();
    st.update(&h, std::slice::from_ref(&qcf), false);
    assert!(st.active.contains("q"));
    // the key-up was lost (window lost focus while x was held): no press edge ever happens again
    for _ in 0..30 {
        h.push(R | X, 1);
        st.update(&h, std::slice::from_ref(&qcf), false);
    }
    assert!(
        !st.active.contains("q"),
        "a permanently held button cannot fire the same command twice"
    );
    // releasing everything and rolling the motion again works, which is what the shell restore must allow
    for _ in 0..3 {
        h.push(0, 1);
    }
    for b in [D, D, D | R, D | R, R, R | X] {
        h.push(b, 1);
    }
    st.update(&h, std::slice::from_ref(&qcf), false);
    assert!(st.active.contains("q"));
}

fn pausing_state(c: &CommandDef, h: &InputHistory, ticks: usize) -> Vec<bool> {
    let mut st = CommandState::default();
    let mut h = h.clone();
    let mut out = vec![];
    st.update(&h, std::slice::from_ref(c), true);
    out.push(st.active.contains(c.name.as_str()));
    for _ in 0..ticks {
        h.push(0, 1);
        st.update(&h, std::slice::from_ref(c), true);
        out.push(st.active.contains(c.name.as_str()));
    }
    out
}
#[test]
fn pause_buffer_keeps_command_while_paused() {
    let h = feed(&[0, X], 1);
    let mut c = parse_command("p", "x", 15, 1);
    assert_eq!(pausing_state(&c, &h, 4), [true, false, false, false, false]);
    c.pause_buffer = 3;
    assert_eq!(
        pausing_state(&c, &h, 5),
        [true, true, true, false, false, false]
    );
}
#[test]
fn pause_buffer_only_arms_during_pause() {
    let mut c = parse_command("p", "x", 15, 1);
    c.pause_buffer = 3;
    let mut st = CommandState::default();
    let mut h = feed(&[0, X], 1);
    st.update(&h, std::slice::from_ref(&c), false);
    assert!(st.active.contains("p"));
    h.push(0, 1);
    st.update(&h, std::slice::from_ref(&c), false);
    assert!(!st.active.contains("p"));
}

#[test]
fn macro_fires_the_command_for_both_facings() {
    for raw in [
        "~D, DF, F, x",
        "F, F, a",
        "~D, DB, B, y",
        "x+y",
        "~30$B, F, x",
        "D, D, a",
        "~D, DF, F, ~D, DF, F, x",
    ] {
        let c = parse_command("m", raw, 15, 1);
        for facing in [1.0, -1.0] {
            let seq = input_macro(&c, facing);
            assert!(!seq.is_empty(), "{raw} facing {facing}");
            assert!(sequence_matches(&c, &seq, facing), "{raw} facing {facing}");
        }
    }
}
#[test]
fn macro_is_empty_for_a_command_without_steps() {
    assert!(input_macro(&parse_command("e", "", 15, 1), 1.0).is_empty());
}
