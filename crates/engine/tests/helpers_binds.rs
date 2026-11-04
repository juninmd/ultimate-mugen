mod support;
use support::*;

const CNS: &str = "
[Statedef 800]
type = S
physics = S

[State 800, h]
type = Helper
trigger1 = Time = 0
id = 5
stateno = 810
pos = 30, 0
supermovetime = 20
preserve = 1
ownpal = 0

[Statedef 810]
type = S
physics = N
anim = 0

[State 810, v]
type = VelSet
trigger1 = 1
x = 2

[Statedef 820]
type = S
physics = S

[State 820, sp]
type = SuperPause
trigger1 = Time = 0
time = 30
anim = -1

[Statedef 830]
type = S
physics = N

[State 830, bind]
type = BindToRoot
trigger1 = 1
time = -1
pos = 25, 0
";

fn helper(m: &ume_engine::matchs::Match) -> usize { m.fighters.iter().position(|f| f.is_helper).expect("helper exists") }

#[test]
fn helper_supermovetime_moves_during_superpause() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", CNS));
    face_off(&mut m, 200.0);
    enter(&mut m, 0, 800);
    m.tick(&[0, 0]);
    enter(&mut m, 1, 820);
    m.tick(&[0, 0]);
    assert!(m.pause.as_ref().is_some_and(|p| p.is_super));
    let h = helper(&m);
    let x0 = m.fighters[h].pos.x;
    let p1x = m.fighters[0].pos.x;
    ticks(&mut m, 10);
    assert!(m.fighters[h].pos.x > x0, "helper kept moving");
    assert_eq!(m.fighters[0].pos.x, p1x, "its root stayed frozen");
}

#[test]
fn preserved_helper_survives_the_round_and_stays_out_of_the_lineup() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 800);
    m.tick(&[0, 0]);
    m.start_round();
    let h = helper(&m);
    assert_eq!(m.fighters[h].helper_id, 5);
    assert_eq!(m.root_indices().len(), 2);
}

#[test]
fn ownpal_zero_shares_the_parent_palfx() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 800);
    m.tick(&[0, 0]);
    m.fighters[0].palfx = Some(ume_engine::fighter::PalFx { time: 10, add: [50.0, 0.0, 0.0], mul: [256.0; 3] });
    let h = helper(&m);
    assert_eq!(m.palfx_of(h).map(|p| p.add[0]), Some(50.0));
    m.fighters[h].own_pal = true;
    assert!(m.palfx_of(h).is_none());
}

#[test]
fn bind_is_reapplied_after_push() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    face_off(&mut m, 30.0);
    m.fighters[0].pos.x = m.fighters[1].pos.x - 10.0;
    let to = m.fighters[0].id;
    m.fighters[1].bind = Some(ume_engine::fighter::Bind { to, time: -1, x: 10.0, y: 0.0, kind: ume_engine::fighter::BindKind::Target });
    for _ in 0..3 {
        m.tick(&[0, 0]);
        let (a, b) = (&m.fighters[0], &m.fighters[1]);
        assert_eq!(b.pos.x, a.pos.x + 10.0 * a.facing, "push must not leave the bound player off its anchor");
    }
}

#[test]
fn custom_state_from_a_destroyed_giver_falls_back_to_own_states() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 800);
    m.tick(&[0, 0]);
    let h = helper(&m);
    let (hid, def) = (m.fighters[h].id, m.fighters[0].def.clone());
    m.change_state(1, 810, def);
    m.fighters[1].state_giver = Some(hid);
    m.destroy_helper(h);
    m.tick(&[0, 0]);
    assert!(std::sync::Arc::ptr_eq(&m.fighters[1].st_def, &m.fighters[1].def));
    assert_ne!(m.fighters[1].state_no, 810);
}
