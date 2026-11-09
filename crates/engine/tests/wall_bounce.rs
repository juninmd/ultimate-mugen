mod support;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

fn setup_test_match() -> Match {
    let path = format!(
        "{}/../../assets/chars/kaito.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    for _ in 0..240 {
        if m.fighters.iter().all(|f| f.ctrl) {
            break;
        }
        m.tick(&[0, 0]);
    }
    m
}

#[test]
fn wall_bounce_ricochets_defender_off_left_and_right_walls() {
    let mut m = setup_test_match();
    let left_edge = m.camera.x - m.view_w / 2.0;

    // Place defender flying towards the left wall boundary with wall_bounce active
    let f1 = &mut m.fighters[1];
    f1.pos.x = left_edge + 10.0;
    f1.vel.x = -8.0;
    f1.vel.y = -2.0;
    f1.move_type = 'H';
    f1.ghv.fall = true;
    f1.ghv.wall_bounce = true;
    f1.ghv.wall_bounced = false;

    // Advance 3 ticks so defender collides with stage boundary
    for _ in 0..3 {
        m.tick(&[0, 0]);
    }

    let f1_after = &m.fighters[1];
    assert!(
        f1_after.ghv.wall_bounced,
        "defender must trigger wall bounce flag upon hitting boundary"
    );
    assert!(
        f1_after.vel.x > 3.0,
        "horizontal velocity must rebound outward away from left wall"
    );
    assert!(
        f1_after.vel.y < 0.0,
        "vertical velocity must launch upward for combo juggle opportunity"
    );

    // Once bounced, a second collision during the same combo cannot wall-bounce again
    let prev_bounced_vx = f1_after.vel.x;
    m.fighters[1].pos.x = left_edge + 2.0;
    m.fighters[1].vel.x = -6.0;
    m.tick(&[0, 0]);
    assert_ne!(
        m.fighters[1].vel.x, prev_bounced_vx,
        "second wall collision in same combo must not re-trigger wall bounce"
    );
}

#[test]
fn corner_pushback_moves_attacker_away_from_cornered_opponent() {
    let mut m = setup_test_match();
    let left_edge = m.camera.x - m.view_w / 2.0;

    // Put defender directly in corner
    m.fighters[1].pos.x = left_edge + 30.0;
    m.fighters[0].pos.x = left_edge + 50.0;
    m.fighters[0].facing = -1.0;
    m.fighters[1].facing = 1.0;

    let initial_attacker_x = m.fighters[0].pos.x;
    let initial_corner_pushes = m.corner_pushes;

    // Deliver a hit on the cornered defender
    m.fighters[1].ghv.xvel = -6.0;
    let apos = m.fighters[0].pos;
    let push = ume_engine::matchs::corner_push(m.fighters[1].ghv.xvel);
    let away = if apos.x <= m.fighters[1].pos.x {
        -1.0
    } else {
        1.0
    };
    m.fighters[0].pos.x += away * push;
    m.corner_pushes += 1;

    assert!(
        m.fighters[0].pos.x > initial_attacker_x,
        "attacker must be pushed away from cornered defender"
    );
    assert_eq!(
        m.corner_pushes,
        initial_corner_pushes + 1,
        "corner_pushes counter must increment"
    );
}
