mod support;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::expr::{parse, truthy, Expr};
use ume_engine::matchs::{Match, TeamMode};

fn ev(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &Expr::new(parse(src).unwrap()))
}

fn match_of(sides: &[usize], simul: bool, turns: bool) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = simul;
    m.team_turns = turns;
    for (slot, side) in sides.iter().enumerate() {
        m.add_player(mugen_char(&format!("fighter{}", slot + 1), ""), *side, 0, 1);
    }
    m.start_round();
    while m.round_state != 2 {
        m.tick(&vec![0; sides.len()]);
    }
    m
}

fn check(m: &Match, number: f64, name: &str) {
    assert_eq!(ev(m, 0, "TeamMode"), number);
    assert_eq!(ev(m, 0, &format!("TeamMode = {name}")), 1.0);
    assert_eq!(
        ev(m, 0, &format!("TeamMode = {}", name.to_uppercase())),
        1.0
    );
    assert_eq!(ev(m, 0, &format!("TeamMode = {}", number as i32)), 1.0);
    assert_eq!(ev(m, 0, &format!("TeamMode != {name}")), 0.0);
    for other in ["single", "simul", "turns", "tag"] {
        if other != name {
            assert_eq!(
                ev(m, 0, &format!("TeamMode = {other}")),
                0.0,
                "{name} is not {other}"
            );
        }
    }
}

#[test]
fn single_mode_is_zero() {
    let m = match_of(&[0, 1], false, false);
    assert_eq!(m.team_mode(), TeamMode::Single);
    check(&m, 0.0, "single");
    assert!(truthy(ev(&m, 0, "TeamMode = 0 && TeamMode < 1")));
}

#[test]
fn parallel_mode_is_simul() {
    let m = match_of(&[0, 1, 0, 1], true, false);
    assert_eq!(m.team_mode(), TeamMode::Simul);
    check(&m, 1.0, "simul");
    assert!(truthy(ev(&m, 0, "TeamMode > 0")));
}

#[test]
fn sequence_mode_is_turns() {
    let m = match_of(&[0, 1], false, true);
    assert_eq!(m.team_mode(), TeamMode::Turns);
    check(&m, 2.0, "turns");
}

#[test]
fn tag_value_is_reserved() {
    assert_eq!(TeamMode::Tag.number(), 3);
    assert_eq!(TeamMode::parse("Tag"), Some(TeamMode::Tag));
    assert_eq!(TeamMode::Simul.name(), "simul");
    assert_eq!(TeamMode::parse("bogus"), None);
}

#[test]
fn bare_form_formats_as_a_name_in_text() {
    let cns = "
[Statedef 900]
type = S

[State 900, t]
type = Text
trigger1 = Time = 0
text = \"mode %s\"
params = TeamMode
removetime = -1
";
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = true;
    m.add_player(mugen_char("a", cns), 0, 0, 1);
    m.add_player(mugen_char("b", ""), 1, 0, 1);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    enter(&mut m, 0, 900);
    ticks(&mut m, 2);
    assert!(
        m.texts.iter().any(|t| t.text == "mode simul"),
        "texts: {:?}",
        m.texts.iter().map(|t| &t.text).collect::<Vec<_>>()
    );
}
