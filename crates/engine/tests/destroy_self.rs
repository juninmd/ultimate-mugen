mod support;
use support::*;

const CNS: &str = "
[Statedef 700]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 700, spawn]
type = Helper
trigger1 = Time = 0
id = 1
stateno = 710

[Statedef 710]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 710, child]
type = Helper
trigger1 = Time = 0
id = 2
stateno = 730

[State 710, fx]
type = Explod
trigger1 = Time = 0
anim = 0
id = 5

[State 710, destroy all]
type = DestroySelf
trigger1 = Time = 3
recursive = 1
removeexplods = 1

[Statedef 701]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 701, spawn]
type = Helper
trigger1 = Time = 0
id = 1
stateno = 720

[Statedef 720]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 720, child]
type = Helper
trigger1 = Time = 0
id = 2
stateno = 730

[State 720, fx]
type = Explod
trigger1 = Time = 0
anim = 0
id = 5

[State 720, destroy alone]
type = DestroySelf
trigger1 = Time = 3

[Statedef 730]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0
";

fn run(state: i32) -> (usize, usize) {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, state);
    ticks(&mut m, 10);
    (m.fighters.iter().filter(|f| f.is_helper).count(), m.explods.len())
}

#[test]
fn recursive_destroy_removes_children_and_explods() {
    assert_eq!(run(700), (0, 0));
}

#[test]
fn plain_destroy_leaves_children_and_explods() {
    let (helpers, explods) = run(701);
    assert_eq!(helpers, 1);
    assert!(explods >= 1);
}
