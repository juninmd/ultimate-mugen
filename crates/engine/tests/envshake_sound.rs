mod support;
use support::*;
use ume_engine::matchs::{Event, Match};

const CNS: &str = "
[Statedef 400]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 400, shake]
type = EnvShake
trigger1 = Time = 0
time = 20
ampl = 8
phase = 90
dir = 90

[Statedef 410]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 410, shake]
type = EnvShake
trigger1 = Time = 0
time = 20
ampl = 8
phase = 90
dir = 45

[Statedef 420]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 420, shake]
type = EnvShake
trigger1 = Time = 0
time = 20
ampl = 8
phase = 90
dir = 450

[Statedef 430]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 430, shake]
type = EnvShake
trigger1 = Time = 0
time = 20
ampl = 8
phase = 90

[Statedef 500]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 500, hit sound]
type = PlaySnd
trigger1 = Time = 0
value = 5, 0
channel = 3
stopongethit = 1

[State 500, state sound]
type = PlaySnd
trigger1 = Time = 0
value = 5, 1
stoponchangestate = 1

[State 500, plain]
type = PlaySnd
trigger1 = Time = 0
value = 5, 2
channel = 4

[Statedef 300]
type = S
movetype = A
physics = N
anim = 200
ctrl = 0

[State 300, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NA
damage = 30
hitflag = MAFD
guardflag = MA
pausetime = 0, 0
ground.velocity = -4
ground.hittime = 20
";

fn setup() -> Match {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", CNS));
    face_off(&mut m, 40.0);
    m
}

fn shake_with(state: i32) -> (f32, f32, f32) {
    let mut m = setup();
    enter(&mut m, 0, state);
    ticks(&mut m, 1);
    assert!(m.shake_time > 0);
    let v = m.shake_vector();
    (m.shake_offset(), v.x, v.y)
}

#[test]
fn envshake_dir_90_shakes_horizontally() {
    let (off, x, y) = shake_with(400);
    assert!(off.abs() > 1.0);
    assert!((x - off).abs() < 1e-4 && y.abs() < 1e-4);
}

#[test]
fn envshake_dir_45_shakes_on_both_axes() {
    let (off, x, y) = shake_with(410);
    assert!((x - y).abs() < 1e-4 && x.abs() > off.abs() * 0.7 - 1e-3 && x.abs() < off.abs());
}

#[test]
fn envshake_dir_is_clamped_to_one_turn() {
    let (off, x, y) = shake_with(420);
    assert!(x.abs() < 1e-3 && (y - off).abs() < 1e-3, "450 degrees clamps to 360, which is the vertical shake");
}

#[test]
fn envshake_without_dir_keeps_the_default_vertical_shake() {
    let mut m = setup();
    enter(&mut m, 0, 430);
    ticks(&mut m, 1);
    assert_eq!(m.shake_dir, None);
    let off = m.shake_offset();
    let v = m.shake_vector();
    assert_eq!((v.x, v.y), (0.0, off));
}

fn stopped_channels(m: &Match, owner: u32) -> Vec<i32> {
    m.events.iter().filter_map(|e| match e { Event::StopSound { owner: o, channel } if *o == owner => Some(*channel), _ => None }).collect()
}

fn played_channels(m: &Match) -> Vec<i32> {
    m.events.iter().filter_map(|e| match e { Event::Sound { channel, .. } => Some(*channel), _ => None }).collect()
}

#[test]
fn sound_marked_stop_on_get_hit_is_stopped_when_its_owner_is_hit() {
    let mut m = setup();
    enter(&mut m, 0, 500);
    ticks(&mut m, 1);
    let played = played_channels(&m);
    assert!(played.contains(&3) && played.contains(&4));
    let auto = *played.iter().find(|&&c| c >= 1000).expect("flagged sound without a channel gets one assigned");
    let owner = m.fighters[0].id;
    m.events.clear();
    enter(&mut m, 1, 300);
    for _ in 0..12 { m.tick(&[0, 0]); if m.fighters[0].move_type == 'H' { break; } }
    assert_eq!(m.fighters[0].move_type, 'H');
    let stopped = stopped_channels(&m, owner);
    assert!(stopped.contains(&3), "channel 3 stops on the hit");
    assert!(!stopped.contains(&4), "an unmarked channel keeps playing");
    assert!(stopped.contains(&auto), "being hit changes the state, which also releases stoponchangestate sounds");
}

#[test]
fn sound_marked_stop_on_change_state_is_stopped_by_the_next_state_change() {
    let mut m = setup();
    enter(&mut m, 0, 500);
    ticks(&mut m, 1);
    let owner = m.fighters[0].id;
    let auto = *played_channels(&m).iter().find(|&&c| c >= 1000).unwrap();
    m.events.clear();
    enter(&mut m, 0, 400);
    let stopped = stopped_channels(&m, owner);
    assert!(stopped.contains(&auto));
    assert!(!stopped.contains(&3), "stopongethit alone does not react to state changes");
    assert!(!stopped.contains(&4));
    m.events.clear();
    enter(&mut m, 0, 500);
    assert!(stopped_channels(&m, owner).is_empty(), "each mark fires once");
}
