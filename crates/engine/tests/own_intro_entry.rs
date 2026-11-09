mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::diagnostics::audit_cns;
use ume_engine::matchs::Match;

const BASE: &str = "
[Statedef 0]
type=S
physics=S
ctrl=1

[Statedef 20]
type=S
physics=S

[Statedef 130]
type=S
physics=S
";

const OWN_INTRO: &str = "
[Statedef 190]
type=S
physics=S
ctrl=0
anim=190

[State 190,assert]
type=AssertSpecial
trigger1=Time<=300
flag=intro

[State 190,bubble]
type=Explod
trigger1=Time=0
anim=191
pos=0,-60
bindtime=-1
removetime=-1

[State 190,end]
type=ChangeState
trigger1=Time>=300
value=0
ctrl=1
";

fn fighter(extra: &str) -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char("c", &format!("{BASE}{extra}")))
        .ok()
        .unwrap();
    d.anims
        .insert(190, anim(190, vec![frame(-1, vec![], vec![])]));
    d.anims
        .insert(191, anim(191, vec![frame(-1, vec![], vec![])]));
    Arc::new(d)
}

fn run(d: Arc<CharDef>) -> (Vec<i32>, bool, usize, usize) {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d, 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    let mut states = vec![];
    let mut bubble_until_fight = false;
    let mut fight_at = 0;
    for t in 0..1200 {
        m.tick(&[0, 0]);
        let s = m.fighters[0].state_no;
        if states.last() != Some(&s) {
            states.push(s);
        }
        bubble_until_fight = m.explods.iter().any(|e| e.cur.anim == 191);
        if m.round_state == 2 {
            fight_at = t;
            break;
        }
    }
    (states, bubble_until_fight, fight_at, m.explods.len())
}

#[test]
fn engine_enters_the_characters_own_intro_state() {
    let (states, bubble, fight_at, _) = run(fighter(OWN_INTRO));
    assert!(states.contains(&190), "{states:?}");
    assert!(!states.contains(&19900), "{states:?}");
    assert!(
        bubble,
        "the character's own bubble must stay until the fight starts"
    );
    assert!(fight_at > 300, "FIGHT started at {fight_at}");
}

#[test]
fn characters_without_an_intro_state_keep_the_generic_pose() {
    let (states, _, _, _) = run(fighter(""));
    assert!(states.contains(&19900), "{states:?}");
}

#[test]
fn explod_parameters_are_recognised_by_the_audit() {
    let cns = "[Statedef 1]\n\n[State 1,a]\ntype=Explod\ntrigger1=1\nanim=5\nfacing=1\npos=0,0\npostype=left\nsprpriority=1\nbindtime=-1\nid=3\nremovetime=-1\nsupermove=1\n";
    let d = audit_cns("x.cns", cns);
    assert!(d.is_clean(), "{}", d.detail());
}
