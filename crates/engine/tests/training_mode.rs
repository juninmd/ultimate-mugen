use std::sync::Arc;
use ume_engine::chardef::{CharDef, Param, StateDef};
use ume_engine::fighter::HitDefData;
use ume_engine::matchs::{Match, StageInfo};

fn state(id: i32, ty: &str, mv: &str, phys: &str) -> StateDef {
    StateDef { id, params: vec![("type".into(), Param::new(ty)), ("movetype".into(), Param::new(mv)), ("physics".into(), Param::new(phys))], controllers: vec![] }
}

fn dummy() -> Arc<CharDef> {
    let mut cd = CharDef::empty("dummy");
    cd.local_w = 320.0; cd.local_h = 240.0;
    for s in [state(0, "S", "I", "S"), state(5000, "S", "H", "S"), state(5150, "L", "H", "N")] { cd.states.insert(s.id, s); }
    Arc::new(cd)
}

fn killing_hit() -> HitDefData { HitDefData { damage: 5000.0, kill: true, p2stateno: -1, hitflag: "MAF".into(), attr_state: "S".into(), attr_kind: vec!["NA".into()], airjuggle: 15, ..Default::default() } }

#[test]
fn lethal_hit_never_kills_the_dummy_in_training() {
    let cd = dummy();
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let a = m.add_player(cd.clone(), 0, 0, 1);
    let b = m.add_player(cd, 1, 0, 2);
    m.training = true;
    m.round_state = 2;
    m.apply_hit(a, b, &killing_hit(), (0.0, 0.0), None);
    assert!(m.fighters[b].alive);
    assert!(m.fighters[b].life >= 1.0);
    for _ in 0..400 { m.tick(&[0, 0]); }
    assert!(m.fighters[b].alive);
    assert_ne!(m.fighters[b].state_no, 5150);
}

#[test]
fn dummy_already_stuck_in_the_dead_state_is_revived() {
    let cd = dummy();
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let _a = m.add_player(cd.clone(), 0, 0, 1);
    let b = m.add_player(cd.clone(), 1, 0, 2);
    m.training = true;
    m.round_state = 2;
    m.fighters[b].alive = false;
    m.fighters[b].life = 0.0;
    m.change_state(b, 5150, cd);
    m.tick(&[0, 0]);
    assert!(m.fighters[b].alive);
    assert_eq!(m.fighters[b].state_no, 0);
    assert_eq!(m.fighters[b].life, m.fighters[b].life_max);
}
