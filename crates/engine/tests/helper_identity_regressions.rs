mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;

const STOCK: &str = "
[Statedef 0]
type=S
physics=S
ctrl=1
anim=0
";

fn with_anims(cns: &str, extra: Vec<ume_engine::chardef::Animation>) -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char("c", &format!("{STOCK}{cns}"))).ok().unwrap();
    for a in extra {
        d.anims.insert(a.id, a);
    }
    Arc::new(d)
}

fn start(d: Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d, 1, 0, 2);
    m.start_round();
    let mut w = 0;
    while m.round_state != 2 && w < 2000 {
        m.tick(&[0, 0]);
        w += 1;
    }
    m
}

fn helpers_with_id(m: &Match, id: i32) -> usize {
    m.fighters.iter().filter(|f| f.is_helper && f.helper_id == id).count()
}

#[test]
fn helper_ids_above_f32_mantissa_are_stored_exactly_and_not_respawned() {
    let cns = "
[State -2, spawn once]
type=Helper
trigger1=!NumHelper(33333333)
name=\"Detector\"
ID=33333333
StateNo=600
postype=p1
ownpal=1

[Statedef 600]
type=S
physics=N
movetype=I
ctrl=0
anim=0
";
    let mut m = start(with_anims(cns, vec![]));
    for _ in 0..30 {
        m.tick(&[0, 0]);
    }
    assert_eq!(helpers_with_id(&m, 33333333), 2, "each player spawns the detector once");
    assert_eq!(helpers_with_id(&m, 33333332), 0, "the id must not drift to the nearest f32");
    assert_eq!(
        m.fighters.iter().filter(|f| f.is_helper && f.helper_id == 33333333 && f.state_no == 600).count(),
        2,
        "one detector per player, both in their own state"
    );
}

#[test]
fn projectile_ids_above_f32_mantissa_are_stored_exactly() {
    let proj = anim(
        888,
        vec![frame(-1, vec![], vec![HURT])],
    );
    let cns = "
[State -2, fire]
type=Projectile
trigger1=1
projanim=888
projid=33333333
projhits=1
velocity=2,0
hitdef=
attr=S,NA
damage=1
";
    let mut m = start(with_anims(cns, vec![proj]));
    for _ in 0..3 {
        m.tick(&[0, 0]);
    }
    assert!(
        m.projectiles.iter().any(|p| p.id == 33333333),
        "ProjID above 2^24 must survive the spawn"
    );
}

#[test]
fn helper_first_state_sees_prevstateno_zero_so_entry_animation_applies() {
    let cns = "
[State -2, spawn]
type=Helper
trigger1=Time=0
name=\"Grenade\"
ID=5
StateNo=600
postype=p1
ownpal=1

[Statedef 600]
type=S
physics=N
movetype=I
ctrl=0

[State 600, entry anim]
type=ChangeAnim
trigger1=!time&&!prevstateno
value=777
";
    let a = anim(777, vec![frame(-1, vec![], vec![HURT])]);
    let mut m = start(with_anims(cns, vec![a]));
    for _ in 0..5 {
        m.tick(&[0, 0]);
    }
    let h = m.fighters.iter().find(|f| f.is_helper && f.helper_id == 5).expect("helper spawned");
    assert_eq!(h.anim_no, 777, "a helper entering its first state must see PrevStateNo = 0");
}
