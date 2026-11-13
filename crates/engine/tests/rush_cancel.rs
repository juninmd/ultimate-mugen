mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::{Event, Match};
use ume_engine::native::{compile_native, NativeChar};
use ume_engine::rush::{COST, COOLDOWN_TICKS, OVERDRIVE_ATTACK_GAIN};

const JAB_STATE: i32 = 200;
const CHARGE_STATE: i32 = 3920;
const CLOSE_GAP: f32 = 34.0;
const WATCH_TICKS: usize = 90;
const FIGHTERS: [&str; 4] = ["akio", "bruno", "jin", "kaito"];

fn fighter(id: &str) -> Arc<CharDef> {
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, true))
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    face_off(&mut m, CLOSE_GAP);
    m.fighters[0].power = 1000.0;
    m
}

fn connect_jab(m: &mut Match) -> bool {
    enter(m, 0, JAB_STATE);
    for _ in 0..WATCH_TICKS {
        m.tick(&[0, 0]);
        if m.fighters[0].move_hit > 0 && m.fighters[0].move_contact > 0 { return true; }
    }
    false
}

fn rush_events(m: &Match) -> usize {
    m.events.iter().filter(|e| matches!(e, Event::Rush { .. })).count()
}

#[test]
fn start_during_a_connected_move_cancels_it_into_neutral_with_control() {
    for id in FIGHTERS {
        let mut m = arena(&fighter(id));
        assert!(connect_jab(&mut m), "{id} jab never connected");
        let before = m.fighters[0].power;
        m.events.clear();
        m.tick(&[k::S, 0]);
        let f = &m.fighters[0];
        assert_eq!(rush_events(&m), 1, "{id} emitted no rush event");
        assert!(f.ctrl && f.move_type != 'A', "{id} did not get control back (state {})", f.state_no);
        assert!((f.power - (before - COST)).abs() < 2.0, "{id} paid the wrong meter ({} to {})", before, f.power);
        assert!(f.rush.overdrive > 0 && f.after_image.is_some() && f.palfx.is_some(), "{id} missing overdrive feedback");
        assert!(m.slow_mo > 0, "{id} no slow motion");
    }
}

#[test]
fn holding_the_button_after_a_cancel_does_not_start_charging() {
    let mut m = arena(&fighter("akio"));
    assert!(connect_jab(&mut m));
    for _ in 0..40 {
        m.tick(&[k::S, 0]);
        assert_ne!(m.fighters[0].state_no, CHARGE_STATE, "held button leaked into the charge");
    }
}

#[test]
fn the_cancel_needs_contact_and_meter() {
    let mut m = arena(&fighter("akio"));
    enter(&mut m, 0, JAB_STATE);
    m.tick(&[k::S, 0]);
    assert!(m.fighters[0].rush.cancels == 0, "cancelled a move that had not connected");

    let mut broke = arena(&fighter("akio"));
    assert!(connect_jab(&mut broke));
    broke.fighters[0].power = COST - 1.0;
    broke.tick(&[k::S, 0]);
    assert_eq!(broke.fighters[0].rush.cancels, 0, "cancelled without enough meter");
}

#[test]
fn cooldown_blocks_back_to_back_cancels_and_overdrive_raises_damage() {
    let mut m = arena(&fighter("akio"));
    assert!(connect_jab(&mut m));
    m.tick(&[k::S, 0]);
    assert_eq!(m.fighters[0].rush.cancels, 1);
    assert!(m.fighters[0].rush.cooldown > 0 && m.fighters[0].rush.cooldown <= COOLDOWN_TICKS);
    assert!((m.fighters[0].effective_attack() - m.fighters[0].attack_mul * OVERDRIVE_ATTACK_GAIN).abs() < 1e-4);
    for _ in 0..200 { m.tick(&[0, 0]); }
    assert_eq!(m.fighters[0].effective_attack(), m.fighters[0].attack_mul, "overdrive never faded");
}

#[test]
fn a_rush_cancel_replays_identically() {
    let run = || {
        let mut m = arena(&fighter("jin"));
        assert!(connect_jab(&mut m));
        m.tick(&[k::S, 0]);
        for _ in 0..30 { m.tick(&[0, 0]); }
        (m.fighters[0].pos.x, m.fighters[0].power, m.fighters[1].life, m.fighters[0].rush.overdrive)
    };
    assert_eq!(run(), run());
}
