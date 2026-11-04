mod support;
use support::*;
use ume_engine::matchs::Event;
use ume_engine::sound_loop::{LoopChange, LoopPlan, LoopSpec};

const CNS: &str = "
[Statedef 700]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 700, points]
type = PlaySnd
trigger1 = Time = 0
value = 1, 0
channel = 2
loopstart = 100
loopend = 400
loopcount = 3

[State 700, plain loop]
type = PlaySnd
trigger1 = Time = 0
value = 1, 1
channel = 3
loop = 1

[State 700, one shot]
type = PlaySnd
trigger1 = Time = 0
value = 1, 2
channel = 4

[Statedef 710]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 710, modify]
type = ModifySnd
trigger1 = Time = 0
channel = 2
loopstart = 50
loopcount = -1

[State 710, stop looping]
type = ModifySnd
trigger1 = Time = 0
channel = 3
loop = 0
";

#[test]
fn infinite_loop_with_points_returns_to_the_loop_start() {
    let plan = LoopPlan::new(1000, 0, &LoopSpec { start: 100, end: 400, count: -1 });
    assert_eq!(plan.source_frame(0), Some(0));
    assert_eq!(plan.source_frame(399), Some(399));
    assert_eq!(plan.source_frame(400), Some(100), "reaching loopend restarts at loopstart");
    assert_eq!(plan.source_frame(699), Some(399));
    assert_eq!(plan.source_frame(700), Some(100));
    assert_eq!(plan.source_frame(10_000_000).map(|f| (100..400).contains(&f)), Some(true));
}

#[test]
fn finite_loop_count_repeats_the_body_then_plays_the_tail() {
    let plan = LoopPlan::new(500, 0, &LoopSpec { start: 100, end: 400, count: 2 });
    assert_eq!(plan.finite_len(), 400 + 2 * 300 + 100);
    assert_eq!(plan.source_frame(400), Some(100));
    assert_eq!(plan.source_frame(700), Some(100));
    assert_eq!(plan.source_frame(1000), Some(400), "after the last repeat the sound runs on to its end");
    assert_eq!(plan.source_frame(1099), Some(499));
    assert_eq!(plan.source_frame(1100), None, "and then it is over");
}

#[test]
fn whole_sample_loops_and_invalid_points_degrade_gracefully() {
    let whole = LoopPlan::new(200, 0, &LoopSpec { start: 0, end: 0, count: -1 });
    assert_eq!(whole.source_frame(250), Some(50));
    let inverted = LoopPlan::new(200, 0, &LoopSpec { start: 150, end: 100, count: -1 });
    assert_eq!(inverted.source_frame(250), None, "an empty loop body plays once");
    let clamped = LoopPlan::new(200, 0, &LoopSpec { start: 10, end: 5000, count: -1 });
    assert_eq!(clamped.source_frame(200), Some(10), "loopend past the sample is clamped to its length");
    let off = LoopPlan::new(200, 0, &LoopSpec { start: 10, end: 100, count: 0 });
    assert_eq!(off.source_frame(150), Some(150), "count 0 disables the loop");
}

#[test]
fn replanning_from_the_current_position_keeps_playing_from_there() {
    let before = LoopPlan::new(1000, 0, &LoopSpec { start: 100, end: 400, count: -1 });
    let at = before.source_frame(550).unwrap();
    assert_eq!(at, 250);
    let after = LoopPlan::new(1000, at, &LoopSpec { start: 0, end: 300, count: -1 }.apply(LoopChange { start: Some(50), ..Default::default() }));
    assert_eq!(after.source_frame(0), Some(250));
    assert_eq!(after.source_frame(50), Some(50), "the new end point (300) is honoured immediately");
}

#[test]
fn unroll_builds_the_head_and_the_repeating_body() {
    let frames: Vec<usize> = (0..10).collect();
    let (head, body) = LoopPlan::new(10, 0, &LoopSpec { start: 2, end: 5, count: -1 }).unroll(&frames);
    assert_eq!(head, vec![0, 1, 2, 3, 4]);
    assert_eq!(body, Some(vec![2, 3, 4]));
    let (head, body) = LoopPlan::new(10, 0, &LoopSpec { start: 2, end: 5, count: 1 }).unroll(&frames);
    assert_eq!(head, vec![0, 1, 2, 3, 4, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(body, None);
}

#[test]
fn playsnd_carries_loop_points_and_count_to_the_channel() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 700);
    ticks(&mut m, 1);
    let spec = |channel: i32| m.events.iter().find_map(|e| match e { Event::Sound { channel: c, loop_spec, looping, .. } if *c == channel => Some((*loop_spec, *looping)), _ => None }).expect("sound event");
    assert_eq!(spec(2), (LoopSpec { start: 100, end: 400, count: 3 }, true));
    assert_eq!(spec(3), (LoopSpec { start: 0, end: 0, count: -1 }, true));
    assert_eq!(spec(4), (LoopSpec::default(), false));
}

#[test]
fn modifysnd_changes_the_loop_of_a_live_channel() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 710);
    ticks(&mut m, 1);
    let owner = m.fighters[0].id;
    let change = |channel: i32| m.events.iter().find_map(|e| match e { Event::SoundLoop { owner: o, channel: c, change } if *o == owner && *c == channel => Some(*change), _ => None }).expect("loop change");
    assert_eq!(change(2), LoopChange { start: Some(50), end: None, count: Some(-1) });
    assert_eq!(change(3), LoopChange { start: None, end: None, count: Some(0) });
}
