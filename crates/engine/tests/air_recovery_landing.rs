use crate::support::*;

fn falling_in(state: i32) -> ume_engine::matchs::Match {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    let f = &mut m.fighters[1];
    f.state_no = state;
    f.physics = 'A';
    f.move_type = 'H';
    f.ctrl = false;
    f.pos.y = f.ground_level - 2.0;
    f.vel.y = 6.0;
    m
}

#[test]
fn air_recovery_state_lands_into_standing_with_control() {
    let mut m = falling_in(5201);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[1].state_no, 52);
    assert!(m.fighters[1].ctrl);
}

#[test]
fn other_hit_states_keep_falling_through_their_own_landing_rules() {
    let mut m = falling_in(5030);
    m.tick(&[0, 0]);
    assert_ne!(m.fighters[1].state_no, 52);
}
