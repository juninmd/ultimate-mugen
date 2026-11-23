use crate::support;
use support::*;

fn cns(extra: &str) -> String {
    format!("
[Statedef 700]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 700, fx]
type = Explod
trigger1 = Time = 0
anim = 0
id = 5
{extra}
")
}

fn on_top(extra: &str) -> bool {
    let mut m = duel(mugen_char("a", &cns(extra)), mugen_char("b", ""));
    enter(&mut m, 0, 700);
    ticks(&mut m, 2);
    m.explods.iter().find(|e| e.id == 5).expect("explod").on_top
}

#[test]
fn layerno_one_matches_ontop() {
    assert!(on_top("layerno = 1"));
    assert!(on_top("ontop = 1"));
    assert!(!on_top("layerno = 0"));
    assert!(!on_top(""));
}
