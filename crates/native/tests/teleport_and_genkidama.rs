use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_native::{compile_native, parse_char};

#[test]
fn converted_goku_teleports_and_genkidama_execute_accurately() {
    let Ok(path) = std::env::var("UME_TEST_UMF_JSON") else { return };
    let nc = parse_char(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }

    m.fighters[0].pos.x = -80.0;
    m.fighters[1].pos.x = 60.0;
    m.fighters[0].facing = 1.0;
    m.fighters[0].power = 3000.0;
    m.change_state(0, 3910, def.clone());
    for _ in 0..80 {
        m.tick(&[0, 0]);
    }
    println!(
        "ABILITIES.TELEPORT: x={:.1} y={:.1} facing={} state={} ctrl={}",
        m.fighters[0].pos.x, m.fighters[0].pos.y, m.fighters[0].facing, m.fighters[0].state_no, m.fighters[0].ctrl
    );
    assert!(m.fighters[0].pos.x > 60.0, "expected blink behind opponent");
    assert_eq!(m.fighters[0].facing, -1.0, "expected Turn after blink");
    assert_eq!(m.fighters[0].state_no, 0, "expected clean return to idle");
    assert!(m.fighters[0].ctrl);

    let genki_sid = def.movelist.iter().find(|mv| mv.command == "m_supergenkidama").unwrap().state;
    m.fighters[0].pos.x = 0.0;
    m.fighters[0].pos.y = 0.0;
    m.fighters[0].facing = 1.0;
    m.fighters[0].power = 3000.0;
    m.change_state(0, genki_sid, def.clone());
    let (mut min_y, mut min_x, mut saw_proj) = (0.0_f32, 0.0_f32, false);
    for _ in 0..420 {
        m.tick(&[0, 0]);
        min_y = min_y.min(m.fighters[0].pos.y);
        min_x = min_x.min(m.fighters[0].pos.x);
        saw_proj |= !m.projectiles.is_empty();
    }
    println!(
        "SUPERGENKIDAMA: min_x={:.1} min_y={:.1} saw_proj={} final=({:.1},{:.1}) state={} ctrl={}",
        min_x, min_y, saw_proj, m.fighters[0].pos.x, m.fighters[0].pos.y, m.fighters[0].state_no, m.fighters[0].ctrl
    );
    assert!(min_y <= -90.0, "expected upward teleport into sky");
    assert!(min_x <= -100.0, "expected backward teleport");
    assert!(saw_proj, "expected Super Genkidama projectile");
    assert!((m.fighters[0].pos.y).abs() < 1.0, "expected return teleport back to floor");
    assert_eq!(m.fighters[0].state_no, 0);
    assert!(m.fighters[0].ctrl);

    let air_tp_sid = def.movelist.iter().find(|mv| mv.command == "m_airteleport_a").unwrap().state;
    m.fighters[0].pos.x = 0.0;
    m.fighters[0].pos.y = 0.0;
    m.fighters[0].facing = 1.0;
    m.change_state(0, air_tp_sid, def.clone());
    let mut air_min_y = 0.0_f32;
    for _ in 0..160 {
        m.tick(&[0, 0]);
        air_min_y = air_min_y.min(m.fighters[0].pos.y);
    }
    println!(
        "AIRTELEPORT_A: min_y={:.1} final=({:.1},{:.1}) state={} ctrl={}",
        air_min_y, m.fighters[0].pos.x, m.fighters[0].pos.y, m.fighters[0].state_no, m.fighters[0].ctrl
    );
    assert!(air_min_y <= -70.0, "expected aerial teleport upward");
    assert!((m.fighters[0].pos.y).abs() < 1.0, "expected natural landing back on floor");
    assert_eq!(m.fighters[0].state_no, 0);
    assert!(m.fighters[0].ctrl);
}
