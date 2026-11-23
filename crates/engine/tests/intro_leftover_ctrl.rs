mod support;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;

const CHAR: &str = "
[Statedef 0]
type=S
physics=S
ctrl=1

[Statedef 190]
type=S
physics=S
ctrl=0
anim=190

[State 190,end]
type=ChangeState
trigger1=Time>=20
value=21

[Statedef 21]
type=S
physics=S
anim=0

[Statedef 20]
type=S
physics=S
";

#[test]
fn custom_idle_state_left_by_the_intro_gets_control_when_the_fight_starts() {
    let mut d = std::sync::Arc::try_unwrap(mugen_char("c", CHAR)).ok().unwrap();
    d.anims.insert(190, anim(190, vec![frame(-1, vec![], vec![])]));
    let d = std::sync::Arc::new(d);
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d, 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    for _ in 0..600 {
        m.tick(&[0, 0]);
        if m.round_state == 2 {
            break;
        }
    }
    assert_eq!(m.round_state, 2);
    for i in m.root_indices() {
        assert!(m.fighters[i].ctrl, "fighter {i} has no control in state {}", m.fighters[i].state_no);
        assert_eq!(m.fighters[i].state_no, 0);
    }
}
