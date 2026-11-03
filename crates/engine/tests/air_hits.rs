//! Regression tests for fighters that stayed in mid-air after taking damage:
//! the get-hit state follows the real height, ground physics drops idle fighters, and the juggle budget ends air combos.

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
    for s in [state(0, "S", "I", "S"), state(5000, "S", "H", "S"), state(5020, "A", "H", "N")] { cd.states.insert(s.id, s); }
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

fn hit() -> HitDefData { HitDefData { damage: 10.0, p2stateno: -1, hitflag: "MAF".into(), attr_state: "S".into(), attr_kind: vec!["NA".into()], airjuggle: 15, ..Default::default() } }

#[test]
fn defender_off_the_floor_gets_the_air_hit_state_even_if_its_state_type_says_ground() {
    let (mut m, a, b) = duel();
    m.fighters[b].pos.y = -40.0; // e.g. a custom move scripted as type S that hovers
    let d = m.fighters[b].def.clone();
    m.change_state(b, 0, d);
    m.apply_hit(a, b, &hit(), (0.0, 0.0), None);
    assert_eq!(m.fighters[b].state_no, 5020, "5000 has ground physics and would pin the fighter in mid-air");
}

#[test]
fn grounded_defender_still_gets_the_stand_hit_state() {
    let (mut m, a, b) = duel();
    m.apply_hit(a, b, &hit(), (0.0, 0.0), None);
    assert_eq!(m.fighters[b].state_no, 5000);
}

#[test]
fn idle_fighter_left_in_mid_air_drops_back_to_the_floor() {
    let (mut m, _a, b) = duel();
    m.fighters[b].pos.y = -60.0;
    for _ in 0..120 { m.tick(&[0, 0]); }
    assert_eq!(m.fighters[b].pos.y, 0.0);
}

#[test]
fn juggle_budget_stops_air_combos() {
    let (mut m, _a, b) = duel();
    let f = &mut m.fighters[b];
    f.state_type = 'A'; f.move_type = 'H'; f.ghv.fall = true; f.pos.y = -50.0;
    let mut hd = hit();
    hd.juggle = 10;
    f.juggle_used = 10;
    assert!(!m.can_be_hit(b, &hd, 0), "10 used + 10 > 15 budget");
    m.fighters[b].juggle_used = 5;
    assert!(m.can_be_hit(b, &hd, 0), "5 used + 10 fits the 15 budget");
    hd.juggle = 0;
    m.fighters[b].juggle_used = 15;
    assert!(m.can_be_hit(b, &hd, 0), "zero-point moves are never blocked");
}
