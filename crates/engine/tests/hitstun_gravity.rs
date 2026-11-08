//! Airborne hit states keep falling even when their script says `physics = N`, while attack states keep their scripted height.

use std::sync::Arc;
use ume_engine::chardef::{CharDef, Param, StateDef};
use ume_engine::matchs::{Match, StageInfo};

fn state(id: i32, ty: &str, mv: &str, phys: &str) -> StateDef {
    StateDef { id, params: vec![("type".into(), Param::new(ty)), ("movetype".into(), Param::new(mv)), ("physics".into(), Param::new(phys))], controllers: vec![] }
}

fn duel() -> (Match, usize) {
    let mut cd = CharDef::empty("dummy");
    cd.local_w = 320.0;
    cd.local_h = 240.0;
    for s in [state(0, "S", "I", "S"), state(5020, "A", "H", "N"), state(700, "A", "I", "N")] {
        cd.states.insert(s.id, s);
    }
    let cd = Arc::new(cd);
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let a = m.add_player(cd.clone(), 0, 0, 1);
    m.add_player(cd, 1, 0, 2);
    m.round_state = 2;
    (m, a)
}

#[test]
fn airborne_hit_state_falls_when_its_script_has_no_physics() {
    let (mut m, b) = duel();
    m.fighters[b].pos.y = -40.0;
    m.fighters[b].vel.y = 0.0;
    let d = m.fighters[b].def.clone();
    m.change_state(b, 5020, d);
    m.tick(&[0, 0]);
    assert!(m.fighters[b].vel.y > 0.0, "gravity must act on a hit state left in mid-air");
    for _ in 0..200 {
        m.tick(&[0, 0]);
    }
    assert_eq!(m.fighters[b].pos.y, 0.0, "the hit fighter lands");
}

#[test]
fn attack_state_keeps_its_scripted_height_without_physics() {
    let (mut m, b) = duel();
    m.fighters[b].pos.y = -40.0;
    m.fighters[b].vel.y = 0.0;
    let d = m.fighters[b].def.clone();
    m.change_state(b, 700, d);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[b].vel.y, 0.0, "a hover move must not be pulled down");
}
