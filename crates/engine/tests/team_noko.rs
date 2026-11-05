mod support;
use support::mugen_char;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::fighter::{asf, HitDefData};
use ume_engine::matchs::Match;

fn simul_match() -> Match {
    let mut game = Match::new(common_def(), None, flat_stage());
    game.team_simul = true;
    for (slot, side) in [0, 1, 0, 1].into_iter().enumerate() {
        game.add_player(mugen_char(&format!("fighter{slot}"), ""), side, 0, 1);
    }
    game.start_round();
    while game.round_state != 2 {
        game.tick(&[0; 4]);
    }
    game
}

#[test]
fn a_team_member_with_noko_does_not_count_as_knocked_out() {
    let mut game = simul_match();
    game.fighters[0].asserts |= asf::NOKO;
    let fatal = HitDefData {
        damage: 2000.0,
        kill: true,
        hitflag: "MAFD".into(),
        attr_state: "S".into(),
        ..Default::default()
    };
    for defender in 0..4 {
        let attacker = if defender % 2 == 0 { 1 } else { 0 };
        game.apply_hit(attacker, defender, &fatal, (0.0, 0.0), None);
    }

    game.tick(&[0; 4]);

    assert_eq!(game.fighters[0].life, 0.0);
    assert!(game.fighters[0].alive);
    assert_eq!(game.round_state, 3);
    assert_eq!(game.winner_side, 0);
}
