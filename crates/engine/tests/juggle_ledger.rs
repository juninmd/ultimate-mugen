//! Juggle points are tracked per attacking source: a helper with `inheritjuggle` shares its parent's
//! (1) or root's (2) budget against a target, while the default (0) keeps the helper on its own.

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
    for s in [state(0, "S", "I", "S"), state(5000, "S", "H", "S"), state(5020, "A", "H", "N"), state(5110, "L", "H", "N"), state(5120, "L", "I", "N"), state(5050, "A", "H", "N")] { cd.states.insert(s.id, s); }
    Arc::new(cd)
}

fn duel() -> (Match, usize, usize) {
    let cd = dummy();
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let a = m.add_player(cd.clone(), 0, 0, 1);
    let b = m.add_player(cd, 1, 0, 2);
    m.round_state = 2;
    (m, a, b)
}

fn hit(juggle: i32) -> HitDefData { HitDefData { damage: 10.0, p2stateno: -1, hitflag: "MAF".into(), attr_state: "S".into(), attr_kind: vec!["NA".into()], airjuggle: 15, juggle, ..Default::default() } }

fn make_falling(m: &mut Match, b: usize) {
    let f = &mut m.fighters[b];
    f.state_type = 'A'; f.move_type = 'H'; f.ghv.fall = true; f.pos.y = -50.0;
}

#[test]
fn a_landed_hit_charges_the_attacker_ledger_entry() {
    let (mut m, a, b) = duel();
    make_falling(&mut m, b);
    let key = m.fighters[a].id;
    m.apply_hit(a, b, &hit(10), (0.0, 0.0), None);
    assert_eq!(m.fighters[b].juggle_spent(key), 10);
}

#[test]
fn a_different_attacker_does_not_share_the_budget() {
    let (mut m, a, b) = duel();
    make_falling(&mut m, b);
    let key = m.fighters[a].id;
    m.fighters[b].add_juggle(key, 10);
    let other = m.fighters[a].id + 1000;
    assert_eq!(m.fighters[b].juggle_spent(other), 0);
    assert!(m.can_be_hit(b, &hit(10), 0, other), "a fresh source has its full 15-point budget");
}

fn add_helper(m: &mut Match, parent: usize, inherit: u8) -> usize {
    let mut h = m.fighters[parent].clone();
    h.id = 900 + inherit as u32;
    h.is_helper = true;
    h.parent = Some(m.fighters[parent].id);
    h.root = m.fighters[parent].id;
    h.inherit_juggle = inherit;
    m.fighters.push(h);
    m.fighters.len() - 1
}

#[test]
fn inherit_parent_helper_spends_the_parents_points() {
    let (mut m, a, b) = duel();
    make_falling(&mut m, b);
    let h = add_helper(&mut m, a, 1);
    let pid = m.fighters[a].id;
    m.fighters[b].add_juggle(pid, 10);
    assert_eq!(m.juggle_source(h), m.fighters[a].id);
    assert!(!m.can_be_hit(b, &hit(6), 0, m.juggle_source(h)), "10 spent by the parent + 6 > 15");
}

#[test]
fn default_helper_keeps_its_own_points() {
    let (mut m, a, b) = duel();
    make_falling(&mut m, b);
    let h = add_helper(&mut m, a, 0);
    let pid = m.fighters[a].id;
    m.fighters[b].add_juggle(pid, 10);
    assert_eq!(m.juggle_source(h), m.fighters[h].id);
    assert!(m.can_be_hit(b, &hit(6), 0, m.juggle_source(h)));
}

#[test]
fn inherit_root_helper_uses_the_root_source() {
    let (mut m, a, b) = duel();
    let h = add_helper(&mut m, a, 2);
    assert_eq!(m.juggle_source(h), m.fighters[a].id);
}
