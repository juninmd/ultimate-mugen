mod support;
use support::*;
use ume_engine::input::k;
use ume_native::{compile_native, parse_char};

const KAITO: &str = include_str!("../../../assets/chars/kaito.json");
const MOVES: &str = r#"
{"id":"charge","name":"Charge","input":"c+z","kind":"special","anim":"triad","hits":[],
 "holdRelease":{"button":"y","tiers":[{"at":0,"to":"quick"},{"at":40,"to":"full"}]}},
{"id":"quick","name":"Quick","input":"a+c+z","kind":"special","anim":"gale","hits":[]},
{"id":"full","name":"Full","input":"b+c+z","kind":"special","anim":"gale","hits":[]}"#;

fn sid(m: &ume_engine::matchs::Match, name: &str) -> i32 {
    m.fighters[0].def.movelist.iter().find(|e| e.name == name).unwrap().state
}

fn held_for(hold: usize) -> (i32, i32, i32) {
    parse_char(KAITO).unwrap();
    let text = format!(
        r##"{{"format":"umf/2","id":"hold_tester","name":"T","series":"t","author":"t","color":"#808080","extends":"kaito","moves":[{MOVES}]}}"##
    );
    let def = std::sync::Arc::new(compile_native(&parse_char(&text).unwrap(), 1, false));
    let mut m = duel(def, mugen_char("dummy", ""));
    face_off(&mut m, 260.0);
    let (charge, quick, full) = (sid(&m, "Charge"), sid(&m, "Quick"), sid(&m, "Full"));
    enter(&mut m, 0, charge);
    for _ in 0..hold {
        m.tick(&[k::Y, 0]);
    }
    assert_eq!(m.fighters[0].state_no, charge, "still charging while the button is held");
    m.tick(&[0, 0]);
    m.tick(&[0, 0]);
    (m.fighters[0].state_no, quick, full)
}

#[test]
fn tapping_releases_the_quick_tier() {
    let (now, quick, _) = held_for(6);
    assert_eq!(now, quick);
}

#[test]
fn holding_past_the_threshold_releases_the_full_tier() {
    let (now, _, full) = held_for(50);
    assert_eq!(now, full);
}
