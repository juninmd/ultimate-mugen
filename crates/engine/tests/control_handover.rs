mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::commands::parse_command;
use ume_engine::input::k::*;
use ume_engine::matchs::Match;

const OWN_STATES: &str = "\
[Statedef 0]
type = S
ctrl = 1
anim = 0

[Statedef 800]
type = S
physics = N
anim = 0
ctrl = 0

[Statedef -1]
[State -1, hit cancel]
type = VarSet
trigger1 = StateNo = 5000
v = 0
value = 42
[State -1, mark]
type = VarSet
trigger1 = 1
v = 1
value = 7
";

fn posing(name: &str) -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char(name, OWN_STATES)).ok().unwrap();
    for id in [190, 195] {
        d.anims
            .insert(id, anim(id, vec![frame(-1, vec![], vec![HURT])]));
    }
    d.commands = ["holdfwd", "holdback", "holdup", "holddown"]
        .iter()
        .map(|n| {
            parse_command(
                n,
                &format!(
                    "/${}",
                    n.trim_start_matches("hold")
                        .chars()
                        .next()
                        .unwrap()
                        .to_ascii_uppercase()
                ),
                1,
                1,
            )
        })
        .collect();
    Arc::new(d)
}

fn with_face_off() -> Match {
    let mut m = Match::new(
        ume_engine::common::common_def(),
        None,
        ume_engine::common::flat_stage(),
    );
    m.add_player(posing("p1"), 0, 0, 1);
    m.add_player(posing("p2"), 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    m
}

/// The engine poses the fighters for the face-off with a state from the shared common definition. When the round
/// starts they go back to state 0, and that hand-over used to keep the common definition as the state owner: the
/// character then looked alive (state 0, `ctrl`) but the engine treated it as a custom state, so its `-1` states and
/// the built-in walk/jump/crouch never ran again — no walking, no attacks, no specials in any mode with a face-off.
#[test]
fn the_face_off_hands_control_back_to_the_character() {
    let mut m = with_face_off();
    for _ in 0..400 {
        if m.round_state == 2 {
            break;
        }
        m.tick(&[0, 0]);
    }
    assert_eq!(m.round_state, 2, "the fight started");
    for f in &m.fighters {
        assert!(f.ctrl, "the round handed control over");
        assert!(
            Arc::ptr_eq(&f.st_def, &f.def),
            "state 0 belongs to the character again, not to the common definition"
        );
    }
    let x = m.fighters[0].pos.x;
    for _ in 0..20 {
        m.tick(&[R, 0]);
    }
    assert_eq!(m.fighters[0].state_no, 20, "holding forward walks");
    assert!(m.fighters[0].pos.x > x + 10.0, "the fighter moved forward");
}

/// M.U.G.E.N. keeps the character's own `-1` states running while it is in a common1 state (get hit, lying down,
/// the shared states): characters rely on that for hit cancels and recovery scripts. Only a custom state given by
/// the opponent pauses them.
#[test]
fn the_character_keeps_its_scripts_during_a_common_hit_state() {
    let mut m = duel(posing("a"), posing("b"));
    let common = m.common.clone();
    m.change_state(0, 5000, common);
    assert_eq!(m.fighters[0].state_no, 5000);
    for _ in 0..3 {
        m.tick(&[0, 0]);
    }
    assert_eq!(
        m.fighters[0].vars[0], 42,
        "the character's -1 state ran during the common get-hit state"
    );
    assert_eq!(m.fighters[0].vars[1], 7);
}

/// A custom state really is the opponent's: the victim's own `-1` states must not fight the throw script.
#[test]
fn a_state_given_by_the_opponent_still_owns_the_fighter() {
    let attacker = posing("a");
    let victim = mugen_char("b", OWN_STATES);
    let mut m = duel(attacker.clone(), victim);
    m.fighters[1].vars[1] = 0;
    m.change_state(1, 800, attacker.clone());
    for _ in 0..3 {
        m.tick(&[0, 0]);
    }
    assert_eq!(m.fighters[1].state_no, 800);
    assert!(
        Arc::ptr_eq(&m.fighters[1].st_def, &attacker),
        "the giver owns the custom state"
    );
    assert_eq!(
        m.fighters[1].vars[1], 0,
        "the victim's -1 state did not run while the attacker owns it"
    );
}
