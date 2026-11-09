mod support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const BUTTON_Z: u32 = 512;
const FINISH_WINDOW_TICKS: i32 = 360;

fn finisher_def(kind: &str) -> Arc<ume_engine::chardef::CharDef> {
    let mut nc: NativeChar = serde_json::from_str(&std::fs::read_to_string("../../assets/chars/kaito.json").unwrap()).unwrap();
    let mut finisher = nc.moves.iter().find(|m| m.id == "strong").unwrap().clone();
    finisher.id = "finisher_move".into();
    finisher.name = "Finisher".into();
    finisher.input = "z".into();
    finisher.kind = kind.into();
    finisher.cancels = None;
    nc.moves.push(finisher);
    Arc::new(compile_native(&nc, 1, true))
}

fn knockout(def: &Arc<ume_engine::chardef::CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.rounds_to_win = 1;
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    face_off(&mut m, 80.0);
    m.fighters[1].life = 0.0;
    m.tick(&[0, 0]);
    m
}

#[test]
fn final_round_ko_opens_the_finish_window() {
    let mut m = knockout(&finisher_def("finisher"));
    assert_eq!(m.round_state, 3);
    assert_eq!(m.finish_window, FINISH_WINDOW_TICKS);
    for _ in 0..600 { m.tick(&[0, 0]); }
    assert!(m.finish_window < FINISH_WINDOW_TICKS);
}

#[test]
fn rosters_without_a_finisher_never_wait() {
    let m = knockout(&finisher_def("special"));
    assert_eq!(m.finish_window, 0);
}

#[test]
fn performing_the_finisher_closes_the_window() {
    let mut m = knockout(&finisher_def("finisher"));
    for step in 0..300 {
        m.tick(&[if step > 100 && step < 104 { BUTTON_Z } else { 0 }, 0]);
        if m.finish_window == 0 { break; }
    }
    assert_eq!(m.finish_window, 0);
    assert!(m.fighters[0].state_no >= 200 || m.round_state >= 3);
}

#[test]
fn window_stays_open_without_the_finisher() {
    let mut m = knockout(&finisher_def("finisher"));
    for _ in 0..300 { m.tick(&[0, 0]); }
    assert!(m.finish_window > 0);
    assert_eq!(m.round_state, 3);
}
