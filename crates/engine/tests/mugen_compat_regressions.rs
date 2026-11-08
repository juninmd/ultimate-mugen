mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::commands::parse_command;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;

const STOCK_COMMON: &str = "
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

fn char_with(cns: &str, extra_anims: Vec<ume_engine::chardef::Animation>) -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char("c", &format!("{STOCK_COMMON}{cns}")))
        .ok()
        .unwrap();
    for a in extra_anims {
        d.anims.insert(a.id, a);
    }
    d.commands.push(parse_command("holdfwd", "/$F", 1, 1));
    d.commands.push(parse_command("holdback", "/$B", 1, 1));
    Arc::new(d)
}

fn pair(a: Arc<CharDef>, b: Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a, 0, 0, 1);
    m.add_player(b, 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    m
}

#[test]
fn anim_time_reaches_zero_on_an_infinite_last_frame() {
    let a = anim(
        777,
        vec![frame(5, vec![], vec![HURT]), frame(-1, vec![], vec![HURT])],
    );
    let cns = "
[Statedef 777]
type=S
movetype=I
physics=S
anim=777

[State 777,end]
type=ChangeState
trigger1=AnimTime=0
value=0
ctrl=1
";
    let d = char_with(cns, vec![a]);
    let mut m = pair(d.clone(), d);
    enter(&mut m, 0, 777);
    let mut left = false;
    for _ in 0..20 {
        m.tick(&[0, 0]);
        if m.fighters[0].state_no == 0 {
            left = true;
            break;
        }
    }
    assert!(
        left,
        "a state ending on a -1 frame must see AnimTime = 0 and leave"
    );
}

#[test]
fn anim_time_is_negative_before_the_infinite_frame() {
    let a = anim(
        778,
        vec![frame(8, vec![], vec![HURT]), frame(-1, vec![], vec![HURT])],
    );
    let d = char_with("[Statedef 778]\ntype=S\nphysics=S\nanim=778\n", vec![a]);
    let mut m = pair(d.clone(), d);
    enter(&mut m, 0, 778);
    m.tick(&[0, 0]);
    assert!(m.fighters[0].anim_time_trig() < 0);
}

#[test]
fn walk_state_without_its_own_exit_returns_to_stand() {
    let d = char_with("", vec![]);
    let mut m = pair(d.clone(), d);
    for _ in 0..12 {
        m.tick(&[support_keys::R, 0]);
    }
    assert_eq!(m.fighters[0].state_no, 20);
    for _ in 0..6 {
        m.tick(&[0, 0]);
    }
    assert_eq!(
        m.fighters[0].state_no, 0,
        "walk must hand back to stand when the direction is released"
    );
}

#[test]
fn guard_state_without_its_own_exit_ends_when_back_is_released() {
    let d = char_with("", vec![]);
    let mut m = pair(d.clone(), d);
    enter(&mut m, 0, 130);
    m.fighters[0].ctrl = true;
    for _ in 0..40 {
        m.tick(&[0, 0]);
    }
    assert_ne!(m.fighters[0].state_no, 130);
    assert!(m.fighters[0].ctrl);
}

mod support_keys {
    pub const R: u32 = 8;
}

#[test]
fn own_intro_state_is_not_replaced_by_the_face_off_and_is_awaited() {
    let intro = anim(190, vec![frame(-1, vec![], vec![HURT])]);
    let cns = "
[Statedef 5900]
type=S
ctrl=0

[State 5900,go]
type=ChangeState
trigger1=1
value=190

[Statedef 190]
type=S
ctrl=0
anim=190

[State 190,assert]
type=AssertSpecial
trigger1=Time<150
flag=intro

[State 190,end]
type=ChangeState
trigger1=Time=150
value=0
ctrl=1
";
    let d = char_with(cns, vec![intro]);
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d, 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    let mut in_own_intro_after_faceoff = false;
    let mut fight_at = None;
    for t in 0..700 {
        m.tick(&[0, 0]);
        if m.round_state == 1 && m.round_time > 60 && m.fighters[0].state_no == 190 {
            in_own_intro_after_faceoff = true;
        }
        if m.round_state == 2 && fight_at.is_none() {
            fight_at = Some(t);
        }
    }
    assert!(
        in_own_intro_after_faceoff,
        "the face-off must not cut the character's own intro"
    );
    assert_eq!(m.fighters[0].state_no, 0);
    let fight_at = fight_at.expect("the round must start");
    assert!(
        fight_at > 40 + 150,
        "FIGHT must wait for the intro to finish, started at {fight_at}"
    );
}
