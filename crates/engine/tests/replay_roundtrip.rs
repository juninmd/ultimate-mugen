mod support;
use support::*;
use std::sync::Arc;
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
        parse_command("a", "a", 1, 1),
    ];
    Arc::new(d)
}

fn fresh() -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(pilot("a"), 0, 0, 1);
    m.add_player(pilot("b"), 1, 0, 2);
    m.start_round();
    m
}

fn input(i: usize) -> [u32; 2] {
    let a = match (i / 7) % 5 { 0 => k::L, 1 => k::R, 2 => k::X, 3 => k::D, _ => 0 };
    let b = match (i / 11) % 6 { 0 => k::R, 1 => k::Y, 2 => k::L, 3 => k::U, 4 => k::A, _ => 0 };
    [a, b]
}

#[test]
fn a_recorded_input_stream_reproduces_the_same_checksum() {
    let mut m = fresh();
    let mut rec = Vec::new();
    for i in 0..1200 {
        let bits = input(i);
        rec.push(bits);
        m.tick(&bits);
    }
    let expected = m.checksum();

    let mut replayed = fresh();
    for bits in &rec {
        replayed.tick(bits);
    }
    assert_eq!(replayed.checksum(), expected, "1200 ticks reproduce the same state");

    let mut different = fresh();
    for _ in 0..1200 { different.tick(&[0, 0]); }
    assert_ne!(different.checksum(), expected, "the checksum notices different inputs");
}

#[test]
fn the_checksum_tracks_single_field_changes() {
    let m = fresh();
    let base = m.checksum();
    let mut moved = fresh();
    moved.fighters[0].pos.x += 1.0;
    assert_ne!(moved.checksum(), base, "position changes the checksum");
    let mut hurt = fresh();
    hurt.fighters[1].life -= 10.0;
    assert_ne!(hurt.checksum(), base, "life changes the checksum");
}
