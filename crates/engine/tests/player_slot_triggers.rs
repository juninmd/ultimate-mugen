mod support;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

fn ev(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &Expr::new(parse(src).unwrap()))
}

fn roster(sides: &[usize]) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    for (slot, side) in sides.iter().enumerate() {
        m.add_player(mugen_char(&format!("fighter{}", slot + 1), ""), *side, 0, 1);
    }
    m.start_round();
    while m.round_state != 2 {
        m.tick(&vec![0; sides.len()]);
    }
    m
}

#[test]
fn p3_to_p8_names_are_empty_in_a_duel() {
    let m = roster(&[0, 1]);
    for slot in 3..=8 {
        assert_eq!(ev(&m, 0, &format!("P{slot}Name = \"fighter{slot}\"")), 0.0);
        assert_eq!(ev(&m, 0, &format!("P{slot}Name = \"\"")), 0.0);
    }
    assert_eq!(ev(&m, 0, "P3Name != \"fighter1\""), 1.0);
}

#[test]
fn p3_p4_names_resolve_by_slot_order_in_two_versus_two() {
    let m = roster(&[0, 1, 0, 1]);
    for viewer in 0..4 {
        assert_eq!(ev(&m, viewer, "P3Name = \"fighter3\""), 1.0);
        assert_eq!(ev(&m, viewer, "P4Name = \"FIGHTER4\""), 1.0);
        assert_eq!(ev(&m, viewer, "P3Name = \"fighter4\""), 0.0);
        assert_eq!(ev(&m, viewer, "P4Name != \"fighter4\""), 0.0);
        assert_eq!(ev(&m, viewer, "P5Name = \"fighter5\""), 0.0);
    }
}

#[test]
fn p3_to_p8_names_resolve_in_four_versus_four() {
    let m = roster(&[0, 1, 0, 1, 0, 1, 0, 1]);
    for slot in 3..=8 {
        assert_eq!(ev(&m, 0, &format!("P{slot}Name = \"fighter{slot}\"")), 1.0);
    }
}

#[test]
fn p2command_reads_the_opponent_buffer() {
    let mut m = roster(&[0, 1]);
    m.fighters[1].cmd.active.insert("x".into());
    assert_eq!(ev(&m, 0, "P2Command = \"x\""), 1.0);
    assert_eq!(ev(&m, 0, "P2Command != \"x\""), 0.0);
    assert_eq!(ev(&m, 0, "Command = \"x\""), 0.0);
    assert_eq!(ev(&m, 1, "P2Command = \"x\""), 0.0);
    m.fighters[0].cmd.active.insert("y".into());
    assert_eq!(ev(&m, 1, "P2Command = \"y\""), 1.0);
}
