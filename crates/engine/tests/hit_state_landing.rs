mod support;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;

const CNS: &str = "
[Statedef 0]
type=S
physics=S
ctrl=1

[Statedef 821]
type=A
movetype=H
physics=N
velset=3.5,-8
anim=0
ctrl=0

[State 821,pos]
type=PosAdd
trigger1=Time=0
y=-14

[State 821,gravity]
type=VelAdd
trigger1=Time>0
y=.4

[State 821,land]
type=SelfState
trigger1=Vel Y>0
trigger1=Pos Y>=10
value=5100

[Statedef 5100]
type=L
movetype=H
physics=N
ctrl=0
anim=0

[State 5100,up]
type=ChangeState
trigger1=Time>=20
value=0
ctrl=1
";

#[test]
fn a_physics_n_hit_state_that_lands_by_script_is_not_clamped_at_the_floor() {
    let d = mugen_char("thrown", CNS);
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    m.change_state(0, 821, d);
    let mut reached = false;
    for _ in 0..300 {
        m.tick(&[0, 0]);
        if m.fighters[0].state_no == 5100 || m.fighters[0].state_no == 0 {
            reached = true;
            break;
        }
    }
    assert!(reached, "stuck in state {}", m.fighters[0].state_no);
}
