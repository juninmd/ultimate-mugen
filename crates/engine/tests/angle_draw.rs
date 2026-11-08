mod support;
use support::*;

#[test]
fn angle_draw_lasts_only_the_tick_that_asked_for_it() {
    let cns = "
[Statedef 510]
type = S
physics = S

[State 510, tilt]
type = AngleDraw
trigger1 = Time < 2
value = 40

[Statedef 511]
type = S
physics = S
";
    let mut m = duel(mugen_char("a", cns), mugen_char("b", ""));
    enter(&mut m, 0, 510);
    m.tick(&[0, 0]);
    assert!(m.fighters[0].angle_draw, "AngleDraw tilts the tick it runs");
    enter(&mut m, 0, 511);
    m.tick(&[0, 0]);
    assert!(
        !m.fighters[0].angle_draw,
        "leaving the state must not leave the fighter tilted"
    );
}
