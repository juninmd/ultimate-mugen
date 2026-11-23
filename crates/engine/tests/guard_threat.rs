mod support;
use std::sync::Arc;
use support::*;
use ume_engine::fighter::HitDefData;
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn striker() -> Match {
    let nc: NativeChar =
        serde_json::from_str(&std::fs::read_to_string("../../assets/chars/kaito.json").unwrap()).unwrap();
    let mut d = compile_native(&nc, 1, true);
    for fr in &mut d.anims.get_mut(&0).unwrap().frames {
        fr.clsn1 = vec![REACH];
        fr.clsn2 = vec![HURT];
    }
    let def = Arc::new(d);
    duel(def.clone(), def)
}

fn arm(m: &mut Match, wide: f32) {
    let f = &mut m.fighters[0];
    f.move_type = 'A';
    f.hitdef_active = true;
    let mut hd = HitDefData::default();
    hd.guardflag = "MA".into();
    hd.guard_dist = wide;
    f.hitdef = Some(hd);
}

#[test]
fn an_attack_within_guard_dist_but_out_of_reach_is_no_threat() {
    let mut m = striker();
    face_off(&mut m, 140.0);
    arm(&mut m, 300.0);
    assert!(m.in_guard_dist(1), "MUGEN guard.dist still reports the attack");
    assert!(!m.in_guard_threat(1), "box 80px short of the body is not a threat");
}

#[test]
fn an_attack_box_touching_the_body_is_a_threat() {
    let mut m = striker();
    face_off(&mut m, 70.0);
    arm(&mut m, 300.0);
    assert!(m.in_guard_threat(1));
}

#[test]
fn walking_back_is_not_pinned_in_guard_by_an_attack_that_cannot_reach() {
    let mut m = striker();
    face_off(&mut m, 140.0);
    for _ in 0..8 {
        arm(&mut m, 300.0);
        m.tick(&[0, k::R]);
    }
    assert!(!(120..=155).contains(&m.fighters[1].state_no), "state {}", m.fighters[1].state_no);
}

#[test]
fn a_jumping_fighter_enters_air_guard_when_the_attack_box_is_close() {
    let mut m = striker();
    face_off(&mut m, 70.0);
    m.fighters[1].state_type = 'A';
    m.fighters[1].state_no = 50;
    m.fighters[1].pos.y = -80.0;
    m.fighters[1].ctrl = true;
    m.fighters[1].guard_button = true;
    arm(&mut m, 200.0);
    m.tick(&[0, k::R]);
    assert!((120..=132).contains(&m.fighters[1].state_no), "air guard state {}", m.fighters[1].state_no);
}
