use std::sync::Arc;

use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::{k, InputSource};
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn native(id: &str) -> Arc<CharDef> {
    let path = format!(
        "{}/../../assets/chars/{id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

fn arena() -> (Match, [usize; 2]) {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(native("kaito"), 0, 0, 1);
    m.add_player(native("shion"), 1, 0, 2);
    m.enable_power_arena();
    m.enable_swarm(1);
    m.rounds_to_win = 3;
    m.start_round();
    let roots = m.root_indices();
    (m, [roots[0], roots[1]])
}

fn ai_until(m: &mut Match, cpu: &mut Ai, me: usize, done: impl Fn(&Match) -> bool) {
    while !done(m) {
        let foe = cpu.poll(m, me);
        m.tick(&[0, foe]);
    }
}

fn on_platform(f: &ume_engine::fighter::Fighter) -> bool {
    f.ground_level <= -50.0 && (f.pos.y - f.ground_level).abs() <= 0.5
}

#[test]
fn power_arena_ai_restarts_its_opening_climb_in_round_two() {
    let (mut m, roots) = arena();
    let mut cpu = Ai::new(4, 99);
    ai_until(&mut m, &mut cpu, roots[1], |m| m.round_state == 2);
    let mut landed = false;
    for _ in 0..360 {
        let foe = cpu.poll(&m, roots[1]);
        m.tick(&[k::L | k::U, foe]);
        if on_platform(&m.fighters[roots[0]]) {
            landed = true;
            break;
        }
    }
    assert!(landed, "P1 did not reach the first tier in round 1");
    m.fighters[roots[1]].life = 0.0;
    ai_until(&mut m, &mut cpu, roots[1], |m| {
        m.round_no == 2 && m.round_state == 0
    });
    ai_until(&mut m, &mut cpu, roots[1], |m| m.round_state == 2);
    let mut climbed = false;
    for _ in 0..240 {
        let foe = cpu.poll(&m, roots[1]);
        m.tick(&[0, foe]);
        if on_platform(&m.fighters[roots[1]]) {
            climbed = true;
            break;
        }
    }
    assert!(climbed, "CPU did not climb the first tier at round 2");
    assert!(
        m.fighters[roots[0]].ground_level > -1.0,
        "P1 is on the ground"
    );
}
