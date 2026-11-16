use std::sync::Arc;
use ume_engine::chardef::{CharDef, Param, StateDef};
use ume_engine::matchs::{Match, StageInfo};
use ume_engine::mutators::Mutators;

fn state(id: i32, ty: &str, mv: &str, phys: &str) -> StateDef {
    StateDef { id, params: vec![("type".into(), Param::new(ty)), ("movetype".into(), Param::new(mv)), ("physics".into(), Param::new(phys))], controllers: vec![] }
}

fn duel(gravity: f32) -> (Match, usize) {
    let mut cd = CharDef::empty("dummy");
    cd.local_w = 320.0;
    cd.local_h = 240.0;
    for s in [state(0, "S", "I", "S"), state(5020, "A", "H", "N")] {
        cd.states.insert(s.id, s);
    }
    let cd = Arc::new(cd);
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let a = m.add_player(cd.clone(), 0, 0, 1);
    m.add_player(cd, 1, 0, 2);
    m.round_state = 2;
    m.mutators = Mutators::with_gravity(gravity).expect("gravity in range");
    (m, a)
}

fn ticks_to_land(gravity: f32) -> usize {
    let (mut m, a) = duel(gravity);
    m.fighters[a].pos.y = -120.0;
    m.fighters[a].vel.y = 0.0;
    let d = m.fighters[a].def.clone();
    m.change_state(a, 5020, d);
    for n in 1..=600 {
        m.tick(&[0, 0]);
        if m.fighters[a].pos.y >= 0.0 {
            return n;
        }
    }
    panic!("fighter never landed at gravity {gravity}");
}

#[test]
fn default_gravity_is_neutral() {
    assert!(Mutators::default().is_default());
    assert_eq!(Mutators::with_gravity(1.0), Some(Mutators::default()));
}

#[test]
fn out_of_range_or_non_finite_gravity_is_rejected() {
    for bad in [0.0, -1.0, 0.1, 4.5, f32::NAN, f32::INFINITY] {
        assert!(Mutators::with_gravity(bad).is_none(), "{bad} must not become a mutator");
    }
}

#[test]
fn stronger_gravity_lands_the_fighter_sooner() {
    let light = ticks_to_land(0.5);
    let normal = ticks_to_land(1.0);
    let heavy = ticks_to_land(2.0);
    assert!(light > normal && normal > heavy, "fall time must shrink as gravity grows: {light} {normal} {heavy}");
}
