mod support;
use support::*;

fn p2_id(cns_template: &str) -> String {
    let probe = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    cns_template.replace("P2ID", &probe.fighters[1].id.to_string())
}

const TARGET_ADD: &str = "
[Statedef 800]
type = S
physics = S
ctrl = 0

[State 800, add]
type = TargetAdd
trigger1 = Time = 0
id = P2ID
targetid = 7

[State 800, again]
type = TargetAdd
trigger1 = Time = 0
id = P2ID
targetid = 9

[State 800, self]
type = TargetAdd
trigger1 = Time = 0
id = ID

[Statedef 801]
type = S
physics = S
ctrl = 0

[State 801, add]
type = TargetAdd
trigger1 = Time = 5
id = P2ID

[Statedef 802]
type = S
physics = S
ctrl = 0

[State 802, add]
type = TargetAdd
trigger1 = Time = 0
id = P2ID
targetid = 3

[Statedef 803]
type = S
physics = S
ctrl = 0

[State 803, drop]
type = TargetDrop
trigger1 = Time = 0

[State 803, readd]
type = TargetAdd
trigger1 = Time = 0
id = P2ID
targetid = 4

[Statedef 804]
type = S
physics = S
ctrl = 0

[State 804, drop]
type = TargetDrop
trigger1 = Time = 0
";

fn target_match() -> ume_engine::matchs::Match {
    duel(mugen_char("p1", &p2_id(TARGET_ADD)), mugen_char("p2", ""))
}

#[test]
fn targetadd_registers_a_target_once_without_a_hit() {
    let mut m = target_match();
    let p2 = m.fighters[1].id;
    assert!(m.fighters[0].targets.is_empty());
    enter(&mut m, 0, 800);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].targets, vec![(p2, 7)], "no duplicate, first target id wins, never the player itself");
    assert_eq!(m.fighters[1].life, m.fighters[1].life_max, "no damage involved");
}

#[test]
fn targetadd_follows_the_normal_target_drop_rules() {
    let mut m = target_match();
    enter(&mut m, 0, 801);
    ticks(&mut m, 6);
    assert!(m.fighters[0].targets.is_empty(), "a target that was never hit is dropped once the state is old");
    let p2 = m.fighters[1].id;
    enter(&mut m, 0, 802);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].targets, vec![(p2, 3)]);
    enter(&mut m, 0, 803);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].targets, vec![(p2, 4)], "TargetDrop clears the list and the player can be added again");
    enter(&mut m, 0, 804);
    m.tick(&[0, 0]);
    assert!(m.fighters[0].targets.is_empty());
}

const TARGET_POINTS: &str = "
[Statedef 810]
type = S
physics = S
ctrl = 0

[State 810, red]
type = TargetRedLifeAdd
trigger1 = Time = 0
value = -100

[State 810, dizzy]
type = TargetDizzyPointsAdd
trigger1 = Time = 0
value = -300

[State 810, guard]
type = TargetGuardPointsAdd
trigger1 = Time = 0
value = -450

[State 810, tscore]
type = TargetScoreAdd
trigger1 = Time = 0
value = 50

[State 810, score]
type = ScoreAdd
trigger1 = Time = 0
value = 25

[State 810, read]
type = VarSet
trigger1 = Time = 0
v = 0
value = Score

[State 810, readtarget]
type = VarSet
trigger1 = Time = 0
v = 1
value = Target,Score
";

#[test]
fn target_point_controllers_change_only_the_targets() {
    let mut m = duel(mugen_char("p1", TARGET_POINTS), mugen_char("p2", ""));
    let p2 = m.fighters[1].id;
    m.fighters[0].targets.push((p2, 0));
    m.fighters[1].life = 500.0;
    m.fighters[1].ikemen.red_life = 900.0;
    enter(&mut m, 0, 810);
    m.tick(&[0, 0]);
    let (a, b) = (&m.fighters[0], &m.fighters[1]);
    assert_eq!(b.ikemen.red_life, 800.0);
    assert_eq!(b.ikemen.dizzy_points, b.ikemen.dizzy_max - 300.0);
    assert_eq!(b.ikemen.guard_points, b.ikemen.guard_max - 450.0);
    assert_eq!((a.ikemen.dizzy_points, a.ikemen.guard_points), (a.ikemen.dizzy_max, a.ikemen.guard_max));
    assert_eq!(b.ikemen.score, 50.0);
    assert_eq!(a.ikemen.score, 25.0);
    assert_eq!(a.vars[0], 25, "Score trigger");
}

#[test]
fn target_point_controllers_clamp_to_their_ranges() {
    let cns = "
[Statedef 811]
type = S
physics = S

[State 811, d]
type = TargetDizzyPointsAdd
trigger1 = Time = 0
value = -99999

[State 811, g]
type = TargetGuardPointsAdd
trigger1 = Time = 0
value = 99999

[State 811, r]
type = TargetRedLifeAdd
trigger1 = Time = 0
value = -99999
absolute = 1
";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    let p2 = m.fighters[1].id;
    m.fighters[0].targets.push((p2, 0));
    m.fighters[1].life = 400.0;
    enter(&mut m, 0, 811);
    m.tick(&[0, 0]);
    let b = &m.fighters[1];
    assert_eq!((b.ikemen.dizzy_points, b.ikemen.guard_points), (0.0, b.ikemen.guard_max));
    assert_eq!(b.ikemen.red_life, 400.0, "red life never drops below the real life");
}

const ROUND_TIME: &str = "
[Statedef 820]
type = S
physics = S

[State 820, set]
type = RoundTimeSet
trigger1 = Time = 0
value = 600

[State 820, add]
type = RoundTimeAdd
trigger1 = Time = 1
value = -100

[Statedef 821]
type = S
physics = S

[State 821, big]
type = RoundTimeSet
trigger1 = Time = 0
value = 999999

[State 821, low]
type = RoundTimeAdd
trigger1 = Time = 1
value = -999999

[State 821, neg]
type = RoundTimeSet
trigger1 = Time = 2
value = -5

[Statedef 822]
type = S
physics = S

[State 822, soon]
type = RoundTimeSet
trigger1 = Time = 0
value = 3
";

#[test]
fn roundtime_controllers_set_and_add_with_clamping() {
    let mut m = duel(mugen_char("p1", ROUND_TIME), mugen_char("p2", ""));
    enter(&mut m, 0, 820);
    m.tick(&[0, 0]);
    assert_eq!(m.timer, 600);
    m.tick(&[0, 0]);
    assert_eq!(m.timer, 500 - 1, "Add applies, then the normal countdown continues");
    let mut m = duel(mugen_char("p1", ROUND_TIME), mugen_char("p2", ""));
    enter(&mut m, 0, 821);
    m.tick(&[0, 0]);
    assert_eq!(m.timer, 100000);
    m.tick(&[0, 0]);
    assert_eq!(m.timer, 0, "clamped at zero");
}

#[test]
fn roundtimeset_still_ends_the_round_through_the_time_over_flow() {
    let mut m = duel(mugen_char("p1", ROUND_TIME), mugen_char("p2", ""));
    enter(&mut m, 0, 822);
    for _ in 0..12 {
        m.tick(&[0, 0]);
        if m.round_state == 3 {
            break;
        }
    }
    assert_eq!(m.round_state, 3, "the round is decided when the shortened timer runs out");
    assert!(!m.won_by_ko);
    assert!(m.timer <= 0);
}
