use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_native::{compile_native, parse_char};

fn apex_after(taps: &[u32]) -> f32 {
    parse_char(include_str!("../../../assets/chars/kaito.json")).expect("kaito parses");
    let ichigo =
        parse_char(include_str!("../../../assets/chars3d/ichigo3d.json")).expect("ichigo3d parses");
    let def = Arc::new(compile_native(&ichigo, 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    for &bits in taps {
        for _ in 0..3 {
            m.tick(&[bits, 0]);
        }
    }
    let mut apex = 0.0f32;
    for _ in 0..90 {
        m.tick(&[0, 0]);
        apex = apex.min(m.fighters[0].pos.y);
    }
    apex
}

#[test]
fn double_tap_up_and_down_up_launch_a_super_jump_above_a_normal_jump() {
    let normal = apex_after(&[k::U]);
    for taps in [[k::D, 0, k::U], [k::U, 0, k::U]] {
        let high = apex_after(&taps);
        assert!(
            high < normal * 1.4,
            "taps {taps:?}: apex {high} vs normal {normal}"
        );
    }
}

#[test]
fn shunpo_double_taps_follow_screen_direction_and_leave_a_trail() {
    parse_char(include_str!("../../../assets/chars/kaito.json")).expect("kaito parses");
    let ichigo = parse_char(include_str!("../../../assets/chars3d/ichigo3d.json")).unwrap();
    let def = Arc::new(compile_native(&ichigo, 1, false));

    for facing in [-1.0, 1.0] {
        for (key, direction) in [(k::L, -1.0), (k::R, 1.0)] {
            let mut m = Match::new(common_def(), None, flat_stage());
            m.add_player(def.clone(), 0, 0, 1);
            m.add_player(def.clone(), 1, 0, 2);
            m.start_round();
            for _ in 0..240 {
                if m.round_state == 2 { break; }
                m.tick(&[0, 0]);
            }
            assert_eq!(m.round_state, 2);
            m.fighters[0].pos.x = 0.0;
            m.fighters[1].pos.x = facing * 145.0;
            m.fighters[0].facing = facing;
            m.tick(&[key, 0]);
            m.tick(&[0, 0]);
            let mut previous = m.fighters[0].pos.x;
            assert!(m.fighters[0].after_image.is_none());
            let mut blinked = false;
            let mut trail = false;
            for bits in [key, 0, 0, 0, 0, 0, 0, 0] {
                m.tick(&[bits, 0]);
                let fighter = &m.fighters[0];
                blinked |= (fighter.pos.x - previous) * direction > 75.0;
                trail |= fighter.after_image.is_some();
                previous = fighter.pos.x;
            }
            assert!(blinked && trail, "facing={facing}, direction={direction}");
        }
    }
}
