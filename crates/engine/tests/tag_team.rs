use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k::*;
use ume_engine::matchs::{Event, Match};
use ume_engine::native::{compile_native, NativeChar};

const IDS: [&str; 6] = ["kaito", "shion", "akio", "suzume", "bruno", "sylene"];

fn load(id: &str) -> NativeChar {
    let p = format!(
        "{}/../../assets/chars/{id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn tag_match(per_side: usize) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_tag = true;
    for k in 0..per_side {
        for side in 0..2 {
            m.add_player(
                Arc::new(compile_native(
                    &load(IDS[(k * 2 + side) % IDS.len()]),
                    1,
                    false,
                )),
                side,
                0,
                1,
            );
        }
    }
    m.start_round();
    for _ in 0..140 {
        m.tick(&vec![0; per_side * 2]);
    }
    m
}

#[test]
fn only_the_first_member_of_each_side_starts_active() {
    let m = tag_match(2);
    assert_eq!(m.round_state, 2);
    let roots = m.root_indices();
    assert_eq!(
        (m.tag_active(0), m.tag_active(1)),
        (Some(roots[0]), Some(roots[1]))
    );
    assert!(m.fighters[roots[2]].ikemen.standby && m.fighters[roots[3]].ikemen.standby);
    assert_eq!(
        m.fighters[roots[2]].pos.x, m.fighters[roots[0]].pos.x,
        "standby fighters wait on top of the active one"
    );
}

#[test]
fn swap_preserves_life_and_variables_and_resets_the_cooldown() {
    let mut m = tag_match(2);
    let roots = m.root_indices();
    m.fighters[roots[2]].life = 321.0;
    m.fighters[roots[2]].vars[3] = 77;
    m.fighters[roots[0]].life = 555.0;
    for _ in 0..30 {
        m.tick(&[0, 0, 0, 0]);
    }
    assert!(m.fighters[roots[0]].ctrl, "neutral fighter has control");
    m.tick(&[TAG, 0, 0, 0]);
    assert_eq!(m.tag_active(0), Some(roots[2]));
    assert!(m.fighters[roots[0]].ikemen.standby);
    assert_eq!(m.fighters[roots[0]].life, 555.0, "outgoing life is kept");
    assert_eq!(m.fighters[roots[2]].life, 321.0, "incoming life is kept");
    assert_eq!(
        m.fighters[roots[2]].vars[3], 77,
        "variables survive the swap"
    );
    assert!(m.tag_cooldown[0] > 0);
    assert!(m
        .events
        .iter()
        .any(|e| matches!(e, Event::Tag { side: 0, .. })));
    m.tick(&[0, 0, TAG, 0]);
    assert_eq!(
        m.tag_active(0),
        Some(roots[2]),
        "cooldown blocks an immediate swap back"
    );
    for _ in 0..m.tag_cooldown_max {
        m.tick(&[0, 0, 0, 0]);
    }
    m.tick(&[0, 0, TAG, 0]);
    assert_eq!(
        m.tag_active(0),
        Some(roots[0]),
        "swap back once the cooldown is over"
    );
}

#[test]
fn standby_fighter_is_untargetable_unpushable_and_receives_no_input() {
    let mut m = tag_match(2);
    let roots = m.root_indices();
    for _ in 0..5 {
        m.tick(&[0, 0, 0, 0]);
    }
    assert_eq!(m.enemy(roots[0], 0), Some(roots[1]));
    assert_ne!(
        m.enemy(roots[0], 1),
        Some(roots[3]),
        "standby enemy is never targeted"
    );
    let hd = ume_engine::fighter::HitDefData {
        hitflag: "MAF".into(),
        affect_team: 'E',
        ..Default::default()
    };
    assert!(m.can_be_hit(roots[1], &hd, 0));
    assert!(
        !m.can_be_hit(roots[3], &hd, 0),
        "standby fighter cannot be hit"
    );
    let before = m.fighters[roots[2]].pos;
    for _ in 0..30 {
        m.tick(&[0, 0, R | X | A, 0]);
    }
    assert_eq!(
        m.fighters[roots[2]].pos.x, m.fighters[roots[0]].pos.x,
        "input never moves the standby fighter ({before:?})"
    );
    assert!(!m.fighters[roots[2]].player_push);
}

#[test]
fn swap_is_refused_while_the_active_fighter_is_being_hit() {
    let mut m = tag_match(2);
    let roots = m.root_indices();
    m.fighters[roots[0]].ctrl = false;
    m.fighters[roots[0]].move_type = 'H';
    m.tick(&[TAG, 0, 0, 0]);
    assert_eq!(m.tag_active(0), Some(roots[0]));
}

#[test]
fn round_ends_only_when_both_members_are_down_and_ko_brings_the_partner_in() {
    let mut m = tag_match(2);
    let roots = m.root_indices();
    m.fighters[roots[1]].life = 0.0;
    for _ in 0..5 {
        m.tick(&[0, 0, 0, 0]);
    }
    assert_eq!(m.round_state, 2, "partner still alive: the round goes on");
    for _ in 0..120 {
        m.tick(&[0, 0, 0, 0]);
    }
    assert_eq!(
        m.tag_active(1),
        Some(roots[3]),
        "the KO'd fighter is replaced automatically"
    );
    assert!(m.fighters[roots[1]].ikemen.standby);
    assert_eq!(m.round_state, 2);
    m.fighters[roots[3]].life = 0.0;
    for _ in 0..5 {
        m.tick(&[0, 0, 0, 0]);
    }
    assert_eq!((m.round_state, m.winner_side), (3, 0));
}

#[test]
fn tag_sctrls_swap_inside_the_mode() {
    use ume_engine::chardef::Controller;
    let mut m = tag_match(2);
    let roots = m.root_indices();
    let sc = |kind: &str| Controller {
        kind: kind.into(),
        label: kind.into(),
        trigger_all: vec![],
        triggers: vec![],
        params: vec![],
        persistent: 1,
        ignore_hitpause: false,
        uid: 1,
    };
    ume_engine::controllers::run(&mut m, roots[0], &sc("tagout"));
    assert_eq!(m.tag_active(0), Some(roots[2]));
    ume_engine::controllers::run(&mut m, roots[0], &sc("tagin"));
    assert_eq!(m.tag_active(0), Some(roots[0]));
}

#[test]
fn three_versus_three_tag_runs_for_a_minute_without_panicking() {
    let mut m = tag_match(3);
    let n = m.root_indices().len();
    assert_eq!(n, 6);
    for i in 0..3600u32 {
        let bits: Vec<u32> = (0..n as u32)
            .map(|k| {
                let mut b = if (i + k * 3) % 7 < 3 {
                    R | if (i / 5 + k) % 2 == 0 { X } else { A }
                } else {
                    0
                };
                if (i + k * 11) % 97 == 0 {
                    b |= TAG;
                }
                b
            })
            .collect();
        m.tick(&bits);
        for side in 0..2 {
            assert!(
                m.tag_members(side)
                    .iter()
                    .filter(|&&i| !m.fighters[i].ikemen.standby)
                    .count()
                    <= 1,
                "never two active fighters on a side"
            );
        }
        if m.match_over {
            break;
        }
        if m.round_state == 4 && m.round_time > 200 {
            break;
        }
    }
}
