mod support;
use support::*;
use ume_native::{compile_native, parse_char};

const KAITO: &str = include_str!("../../../assets/chars/kaito.json");

fn fighter(moves: &str) -> ume_native::NativeChar {
    parse_char(KAITO).unwrap();
    let text = format!(
        r##"{{"format":"umf/2","id":"blocks_tester","name":"T","series":"t","author":"t","color":"#808080","extends":["kaito","abilities"],"moves":[{moves}]}}"##
    );
    parse_char(&text).unwrap()
}

fn start(moves: &str, name: &str) -> ume_engine::matchs::Match {
    let def = std::sync::Arc::new(compile_native(&fighter(moves), 1, false));
    let mut m = duel(def, mugen_char("dummy", ""));
    face_off(&mut m, 200.0);
    let sid = m.fighters[0]
        .def
        .movelist
        .iter()
        .find(|e| e.name == name)
        .unwrap()
        .state;
    enter(&mut m, 0, sid as i32);
    m
}

#[test]
fn instant_transmission_reappears_behind_the_rival_and_turns() {
    let mut m = start(
        r#"{"id":"it","use":"instant_transmission"}"#,
        "Instant Transmission",
    );
    ticks(&mut m, 6);
    let (a, b) = (m.fighters[0].pos.x, m.fighters[1].pos.x);
    assert!((a - b).abs() < 80.0, "{a} {b}");
    assert!(m.fighters[0].facing < 0.0, "turned toward the rival");
}

#[test]
fn blink_strike_lands_in_front_of_the_rival() {
    let mut m = start(r#"{"id":"bs","use":"blink_strike"}"#, "Blink Strike");
    ticks(&mut m, 6);
    let gap = m.fighters[1].pos.x - m.fighters[0].pos.x;
    assert!(gap > 0.0 && gap < 90.0, "{gap}");
}

#[test]
fn ki_barrage_fires_the_whole_volley_from_one_move() {
    let mut m = start(
        r#"{"id":"kb","use":"ki_barrage","projectile":{"anim":"galeOrb"}}"#,
        "Ki Barrage",
    );
    let mut most = 0;
    for _ in 0..60 {
        ticks(&mut m, 1);
        most = most.max(m.projectiles.len());
    }
    assert!(most >= 5, "{most}");
}

#[test]
fn beam_spawns_the_charge_effect_and_shakes_the_screen() {
    let mut m = start(
        r#"{"id":"bm","use":"beam","projectile":{"anim":"galeOrb"}}"#,
        "Energy Beam",
    );
    ticks(&mut m, 8);
    assert!(!m.explods.is_empty(), "charge effect");
    assert!(m.shake_time > 0 || m.shake_elapsed > 0, "quake");
}
