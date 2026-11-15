use std::sync::Arc;
use ume_engine::chardef::{CharDef, Source};
use ume_engine::cns::parse_states;
use ume_engine::matchs::{Match, StageInfo};

const CNS: &str = "
[Statedef 5050]
type = A
movetype = H
physics = N

[State 5050, grav]
type = VelAdd
trigger1 = 1
y = 0.44

[State 5050, land]
type = ChangeState
trigger1 = Pos Y >= 25
value = 5100

[Statedef 5100]
type = S
movetype = I
physics = S
ctrl = 1
";

fn dummy() -> Arc<CharDef> {
    let mut cd = CharDef::empty("dummy");
    cd.source = Source::Mugen;
    cd.local_w = 320.0;
    cd.local_h = 240.0;
    parse_states(CNS, &mut cd.states, true);
    Arc::new(cd)
}

#[test]
fn scripted_air_hit_state_lands_when_its_own_trigger_fires() {
    let cd = dummy();
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let b = m.add_player(cd.clone(), 0, 0, 1);
    m.add_player(cd.clone(), 1, 0, 2);
    m.round_state = 2;
    m.fighters[b].pos.y = -100.0;
    m.fighters[b].state_type = 'A';
    m.fighters[b].move_type = 'H';
    m.fighters[b].ctrl = false;
    m.change_state(b, 5050, cd.clone());
    for _ in 0..600 {
        m.tick(&[0, 0]);
        if m.fighters[b].state_no == 5100 { break; }
    }
    assert_eq!(m.fighters[b].state_no, 5100, "pos y {} vel y {}", m.fighters[b].pos.y, m.fighters[b].vel.y);
}
