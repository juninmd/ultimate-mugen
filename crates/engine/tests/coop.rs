mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::commands::parse_command;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;

fn pilot(name: &str) -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char(name, "")).ok().unwrap();
    d.commands = vec![
        parse_command("holdfwd", "F", 1, 1),
        parse_command("holdback", "B", 1, 1),
        parse_command("holdup", "U", 1, 1),
        parse_command("holddown", "D", 1, 1),
        parse_command("x", "x", 1, 1),
        parse_command("y", "y", 1, 1),
    ];
    Arc::new(d)
}

/// Two humans on side 0 (simultaneous team) against one AI-driven fighter on side 1.
fn coop_match() -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = true;
    m.add_player(pilot("p1"), 0, 0, 1);
    m.add_player(pilot("p2"), 0, 0, 2);
    m.add_player(pilot("cpu"), 1, 4, 1);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0, 0]);
    }
    m
}

fn inputs(i: u32) -> [u32; 3] {
    let p1 = match (i / 9) % 4 {
        0 => k::L,
        1 => k::R,
        2 => k::X,
        _ => 0,
    };
    let p2 = match (i / 13) % 4 {
        0 => k::R,
        1 => k::Y,
        2 => k::L,
        _ => 0,
    };
    let cpu = match (i / 17) % 3 {
        0 => k::L,
        1 => k::X,
        _ => 0,
    };
    [p1, p2, cpu]
}

#[test]
fn two_humans_and_an_ai_run_sixty_seconds_headless() {
    let mut m = coop_match();
    assert_eq!(
        m.roots().filter(|f| f.side == 0).count(),
        2,
        "both co-op fighters are on the field"
    );
    for i in 0..3600 {
        m.tick(&inputs(i));
        assert!(
            m.fighters
                .iter()
                .all(|f| f.pos.x.is_finite() && f.pos.y.is_finite() && f.life.is_finite()),
            "state stays finite after {i} ticks"
        );
        if m.match_over {
            break;
        }
    }
    assert!(m.game_time > 3000, "the fight runs for the full minute");
    assert_eq!(
        m.roots().filter(|f| f.side == 0).count(),
        2,
        "nobody is removed from the side"
    );
}

#[test]
fn a_side_only_falls_when_both_members_are_koed() {
    let mut m = coop_match();
    m.fighters[0].life = 0.0;
    for _ in 0..6 {
        m.tick(&[0, 0, 0]);
    }
    assert!(!m.fighters[0].alive, "the downed ally stays down");
    assert_ne!(
        m.round_state, 3,
        "the round continues while the partner is alive"
    );
    assert!(m.fighters[1].alive, "the partner is still fighting");
    m.fighters[1].life = 0.0;
    for _ in 0..6 {
        m.tick(&[0, 0, 0]);
    }
    assert_eq!(
        m.round_state, 3,
        "the round ends when the whole side is KO'd"
    );
    assert_eq!(m.winner_side, 1, "the AI side wins the round");
}

#[test]
fn both_coop_fighters_take_input_and_move_independently() {
    let mut m = coop_match();
    let (a, b) = (m.fighters[0].pos.x, m.fighters[1].pos.x);
    for _ in 0..40 {
        m.tick(&[k::R, k::L, 0]);
    }
    assert_ne!(m.fighters[0].pos.x, a, "P1 moved with its own input");
    assert_ne!(m.fighters[1].pos.x, b, "P2 moved with its own input");
    assert_ne!(
        m.fighters[0].pos.x, m.fighters[1].pos.x,
        "the two humans are independent"
    );
}
