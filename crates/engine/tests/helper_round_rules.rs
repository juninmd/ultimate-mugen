mod support;
use support::*;
use ume_engine::matchs::Match;

const CNS: &str = "
[Statedef 220]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 220, always moving]
type = Helper
trigger1 = Time = 0
id = 7
stateno = 230
pausebool = 1
keyctrl = 1

[State 220, plain]
type = Helper
trigger1 = Time = 0
id = 8
stateno = 230
keyctrl = 1

[Statedef 230]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 230, drift]
type = PosAdd
trigger1 = 1
x = 1

[Statedef 240]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 240, pause]
type = Pause
trigger1 = Time = 0
time = 30

[Statedef 250]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 250, super]
type = SuperPause
trigger1 = Time = 0
time = 30
anim = -1
";

fn setup() -> (Match, usize, usize) {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", CNS));
    face_off(&mut m, 200.0);
    enter(&mut m, 0, 220);
    ticks(&mut m, 2);
    let find = |m: &Match, id: i32| (0..m.fighters.len()).find(|&i| m.fighters[i].is_helper && m.fighters[i].helper_id == id).unwrap();
    let (a, b) = (find(&m, 7), find(&m, 8));
    (m, a, b)
}

#[test]
fn pausebool_helper_ignores_pause_and_superpause() {
    for state in [240, 250] {
        let (mut m, always, plain) = setup();
        enter(&mut m, 1, state);
        m.tick(&[0, 0]);
        assert!(m.pause.is_some());
        let (ax, px, rx) = (m.fighters[always].pos.x, m.fighters[plain].pos.x, m.fighters[0].pos.x);
        ticks(&mut m, 8);
        assert_eq!(m.fighters[always].pos.x, ax + 8.0, "state {state}: pausebool helper keeps moving");
        assert_eq!(m.fighters[plain].pos.x, px, "state {state}: a regular helper freezes");
        assert_eq!(m.fighters[0].pos.x, rx, "state {state}: so does its root");
    }
}

#[test]
fn helpers_stop_accepting_commands_when_the_round_ends() {
    let (mut m, _, plain) = setup();
    let root = m.fighters[0].id;
    m.fighters[0].cmd.active.insert("x".into());
    assert!(m.has_command(plain, "x"));
    assert!(m.has_command(0, "x"));
    m.round_state = 3;
    assert!(!m.has_command(plain, "x"), "helpers lose their commands as soon as the round is decided");
    assert!(m.has_command(0, "x"), "the root keeps its own until it settles");
    m.round_state = 4;
    assert!(!m.has_command(plain, "x"));
    assert_eq!(m.fighters[plain].root, root);
}

#[test]
fn helpers_are_frozen_and_hidden_on_the_victory_screen() {
    let (mut m, always, plain) = setup();
    let invisible = |m: &Match, i: usize| m.fighters[i].has_assert(ume_engine::fighter::asf::INVISIBLE);
    ticks(&mut m, 1);
    assert!(!invisible(&m, plain) && !invisible(&m, always));
    let (px, ax) = (m.fighters[plain].pos.x, m.fighters[always].pos.x);
    m.round_state = 4;
    m.round_time = 0;
    ticks(&mut m, 5);
    assert_eq!((m.fighters[plain].pos.x, m.fighters[always].pos.x), (px, ax), "no helper moves, pausebool included");
    assert!(invisible(&m, plain) && invisible(&m, always));
}
