mod support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::fighter::HitDefData;
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn duel() -> Match {
    let nc: NativeChar =
        serde_json::from_str(&std::fs::read_to_string("../../assets/chars/kaito.json").unwrap())
            .unwrap();
    let def = Arc::new(compile_native(&nc, 1, true));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    face_off(&mut m, 60.0);
    m
}

fn mid_hit() -> HitDefData {
    let mut hd = HitDefData::default();
    hd.guardflag = "MA".into();
    hd
}

#[test]
fn a_defender_without_guard_input_is_hit_not_blocked() {
    let mut m = duel();
    for _ in 0..10 {
        m.tick(&[0, 0]);
    }
    m.apply_hit(0, 1, &mid_hit(), (0.0, 0.0), None);
    assert!(!m.fighters[1].ghv.guarded, "idle defender must take the hit");
}

#[test]
fn a_defender_holding_back_blocks_the_same_hit() {
    let mut m = duel();
    for _ in 0..10 {
        m.tick(&[0, k::R]);
    }
    m.apply_hit(0, 1, &mid_hit(), (0.0, 0.0), None);
    assert!(m.fighters[1].ghv.guarded, "holding back must block");
}
