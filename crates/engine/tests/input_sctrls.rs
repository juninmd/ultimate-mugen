mod support;
use support::*;
use std::sync::Arc;
use ume_engine::commands::parse_command;
use ume_engine::input::{k, InputHistory};

fn with_punch(cns: &str) -> Arc<ume_engine::chardef::CharDef> {
    let mut def = Arc::try_unwrap(mugen_char("p1", cns)).ok().unwrap();
    def.commands.push(parse_command("punch", "x", 1, 1));
    Arc::new(def)
}

fn history_of(bits: &[u32]) -> InputHistory {
    let mut h = InputHistory::default();
    for b in bits {
        h.push(*b, 1);
    }
    h
}

fn window(h: &InputHistory, len: i64) -> Vec<u32> {
    (h.tick - len + 1..=h.tick).map(|t| h.at(t)).collect()
}

#[test]
fn shift_delays_and_advances_the_history() {
    let mut h = history_of(&[k::A, k::B, k::C, k::X]);
    h.shift(1);
    assert_eq!(window(&h, 4), vec![0, k::A, k::B, k::C]);
    let mut h = history_of(&[k::A, k::B, k::C, k::X]);
    h.shift(-1);
    assert_eq!(window(&h, 4), vec![k::B, k::C, k::X, 0]);
    let mut h = history_of(&[k::A, k::B]);
    h.shift(0);
    assert_eq!(window(&h, 2), vec![k::A, k::B]);
    assert_eq!(h.tick, 1, "shifting never moves the tick counter");
    let mut h = history_of(&[k::A, k::B]);
    h.shift(40);
    assert_eq!(window(&h, 2), vec![0, 0]);
}

const ASSERT_COMMAND: &str = "
[Statedef 600]
type = S
physics = S
ctrl = 0

[State 600, assert]
type = AssertCommand
trigger1 = Time = 0
name = \"punch\"
buffertime = 3

[State 600, count]
type = VarAdd
trigger1 = Command = \"punch\"
v = 0
value = 1
";

#[test]
fn assertcommand_triggers_the_command_for_buffertime_ticks() {
    let mut m = duel(with_punch(ASSERT_COMMAND), mugen_char("p2", ""));
    enter(&mut m, 0, 600);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].vars[0], 1, "the assert is visible to later controllers on the same tick");
    ticks(&mut m, 2);
    assert_eq!(m.fighters[0].vars[0], 3);
    ticks(&mut m, 3);
    assert_eq!(m.fighters[0].vars[0], 3, "the command stops after buffertime ticks");
}

#[test]
fn assertcommand_can_target_another_player() {
    let probe = duel(with_punch(""), with_punch(""));
    let other = probe.fighters[1].id;
    let cns = format!("
[Statedef 601]
type = S
physics = S

[State 601, assert]
type = AssertCommand
trigger1 = Time = 0
name = \"punch\"
redirectid = {other}
");
    let mut m = duel(with_punch(&cns), with_punch(""));
    assert_eq!(m.fighters[1].id, other);
    enter(&mut m, 0, 601);
    m.tick(&[0, 0]);
    assert!(m.fighters[1].cmd.active.contains("punch"));
    assert!(!m.fighters[0].cmd.active.contains("punch"));
}

const ASSERT_INPUT: &str = "
[Statedef 610]
type = S
physics = S
ctrl = 0

[State 610, assert]
type = AssertInput
trigger1 = Time = 0
flag = R, x
time = 2

[State 610, relative]
type = AssertInput
trigger1 = Time = 5
flag = F
";

#[test]
fn assertinput_injects_bits_for_n_ticks() {
    let mut m = duel(with_punch(ASSERT_INPUT), mugen_char("p2", ""));
    enter(&mut m, 0, 610);
    m.tick(&[0, 0]);
    let h = &m.fighters[0].history;
    assert_eq!(h.at(h.tick), k::R | k::X, "the current tick already carries the injected bits");
    assert!(m.fighters[0].cmd.active.contains("punch"), "the injected press is matched on the tick it is injected");
    m.tick(&[0, 0]);
    let h = &m.fighters[0].history;
    assert_eq!(h.at(h.tick), k::R | k::X, "second tick of time = 2");
    assert!(!m.fighters[0].cmd.active.contains("punch"), "a held button is not a new press");
    m.tick(&[0, 0]);
    let h = &m.fighters[0].history;
    assert_eq!(h.at(h.tick), 0, "injection ends after time ticks");
    assert!(!m.fighters[0].cmd.active.contains("punch"));
    m.fighters[0].facing = -1.0;
    ticks(&mut m, 3);
    let h = &m.fighters[0].history;
    assert_eq!(h.at(h.tick), k::L, "F is relative to facing");
}

#[test]
fn assertinput_ors_with_real_input() {
    let mut m = duel(with_punch(ASSERT_INPUT), mugen_char("p2", ""));
    enter(&mut m, 0, 610);
    m.tick(&[k::U, 0]);
    let h = &m.fighters[0].history;
    assert_eq!(h.at(h.tick), k::U | k::R | k::X);
    m.tick(&[k::D, 0]);
    let h = &m.fighters[0].history;
    assert_eq!(h.at(h.tick), k::D | k::R | k::X);
}

#[test]
fn shiftinput_rewrites_history_through_the_controller() {
    let cns = "
[Statedef 620]
type = S
physics = S

[State 620, shift]
type = ShiftInput
trigger1 = Time = 2
value = 1

[State 620, back]
type = ShiftInput
trigger1 = Time = 4
value = -1
";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    enter(&mut m, 0, 620);
    m.tick(&[k::A, 0]);
    m.tick(&[k::B, 0]);
    m.tick(&[k::C, 0]);
    assert_eq!(window(&m.fighters[0].history, 3), vec![0, k::A, k::B], "a positive shift delays every input by one tick");
    m.tick(&[k::Y, 0]);
    m.tick(&[k::Z, 0]);
    assert_eq!(window(&m.fighters[0].history, 5), vec![k::A, k::B, k::Y, k::Z, 0], "a negative shift moves them one tick earlier");
}

const ASSISTED: &str = "
[Statedef -1]

[State -1, command]
type = AssertCommand
trigger1 = GameTime % 7 = 0
name = \"punch\"
buffertime = 2

[State -1, input]
type = AssertInput
trigger1 = GameTime % 11 = 0
flag = R, y
time = 3

[State -1, shift]
type = ShiftInput
trigger1 = GameTime % 13 = 0
value = 1

[State -1, jab]
type = ChangeState
trigger1 = Command = \"punch\" && StateType = S && Ctrl
value = 200
ctrl = 0

[Statedef 200]
type = S
movetype = A
physics = S
anim = 200
ctrl = 0

[State 200, end]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
";

fn state_hash(m: &ume_engine::matchs::Match) -> u64 {
    let mut h: u64 = 1469598103934665603;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(1099511628211);
    };
    for f in &m.fighters {
        mix(f.id as u64);
        mix(f.life.to_bits() as u64);
        mix(f.pos.x.to_bits() as u64);
        mix(f.state_no as u64);
        mix(f.state_time as u64);
        mix(f.history.at(f.history.tick) as u64);
        mix(f.vars[0] as u64);
    }
    h
}

fn scripted(i: usize) -> u32 {
    [0, k::R, k::D, k::X, 0, k::L | k::A][(i.wrapping_mul(2654435761) >> 7) % 6]
}

fn run_assisted(ticks_to_run: usize) -> (Vec<u64>, usize) {
    let mut m = duel(with_punch(ASSISTED), mugen_char("p2", ""));
    face_off(&mut m, 60.0);
    let (mut hashes, mut jabs) = (vec![], 0);
    for i in 0..ticks_to_run {
        m.tick(&[scripted(i), 0]);
        if m.fighters[0].state_no == 200 && m.fighters[0].state_time == 1 {
            jabs += 1;
        }
        hashes.push(state_hash(&m));
    }
    (hashes, jabs)
}

#[test]
fn asserted_inputs_replay_deterministically() {
    let (first, jabs) = run_assisted(600);
    let (second, _) = run_assisted(600);
    assert_eq!(first, second, "same input stream and same asserts give identical state");
    assert!(jabs > 5, "the asserted command actually drives the character ({jabs} jabs)");
}
