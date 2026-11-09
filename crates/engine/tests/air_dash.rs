mod support;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

fn setup_air_match() -> Match {
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
fn air_dash_propulsion_and_state_recovery() {
    let mut m = setup_air_match();

    let f0 = &mut m.fighters[0];
    f0.pos.y = -80.0;
    f0.vel.y = 2.0;
    f0.state_type = 'A';
    f0.state_no = 50;
    f0.ctrl = true;
    f0.air_dashes = 0;

    use ume_engine::input::k;
    m.tick(&[k::R, 0]);
    m.tick(&[0, 0]);
    m.tick(&[k::R, 0]);

    assert_eq!(m.fighters[0].state_no, 110);
    assert_eq!(m.fighters[0].air_dashes, 1);
    assert!(m.fighters[0].vel.x > 4.0);
    assert_eq!(m.fighters[0].vel.y, 0.0);

    for _ in 0..14 {
        m.tick(&[0, 0]);
    }

    assert_eq!(m.fighters[0].state_no, 50);
    assert!(m.fighters[0].ctrl);
    assert_eq!(m.fighters[0].air_dashes, 1);
}

#[test]
fn air_retreat_and_landing_reset() {
    let mut m = setup_air_match();

    let f0 = &mut m.fighters[0];
    f0.pos.y = -70.0;
    f0.vel.y = 1.0;
    f0.state_type = 'A';
    f0.state_no = 50;
    f0.ctrl = true;
    f0.air_dashes = 0;

    use ume_engine::input::k;
    m.tick(&[k::L, 0]);
    m.tick(&[0, 0]);
    m.tick(&[k::L, 0]);

    assert_eq!(m.fighters[0].state_no, 115);
    assert_eq!(m.fighters[0].air_dashes, 1);
    assert!(m.fighters[0].vel.x < -3.0);
    assert_eq!(m.fighters[0].vel.y, 0.0);

    for _ in 0..40 {
        if m.fighters[0].state_type != 'A' {
            break;
        }
        m.tick(&[0, 0]);
    }

    assert_eq!(m.fighters[0].air_dashes, 0);
}
