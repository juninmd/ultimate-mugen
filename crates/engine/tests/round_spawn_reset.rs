use std::sync::Arc;

use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::fighter::V2;
use ume_engine::matchs::{Match, ARENA_P1_SPAWN_X, ARENA_P2_SPAWN_X};
use ume_native::{compile_native, NativeChar};

fn native(id: &str) -> Arc<CharDef> {
    let path = format!(
        "{}/../../assets/chars/{id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

#[derive(Debug, PartialEq)]
struct Snap {
    roots: Vec<(V2, V2, f32, f32, i32, bool)>,
    camera: V2,
    zoom: f32,
    view: (f32, f32),
    projectiles: usize,
}

fn snap(m: &Match) -> Snap {
    Snap {
        roots: m
            .root_indices()
            .into_iter()
            .map(|i| {
                let f = &m.fighters[i];
                (f.pos, f.vel, f.facing, f.ground_level, f.state_no, f.ctrl)
            })
            .collect(),
        camera: m.camera,
        zoom: m.zoom,
        view: (m.view_w, m.view_h),
        projectiles: m.projectiles.len(),
    }
}

fn open(arena: bool) -> (Match, Snap) {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(native(if arena { "kaito" } else { "akio" }), 0, 0, 1);
    m.add_player(native(if arena { "shion" } else { "bruno" }), 1, 0, 2);
    if arena {
        m.enable_power_arena();
        m.enable_swarm(1);
    } else {
        m.zoom_max = 2.0;
    }
    m.start_round();
    m.tick(&[0, 0]);
    let first = snap(&m);
    (m, first)
}

fn move_far_and_ko(m: &mut Match, p1: V2, p2: V2) {
    let roots = m.root_indices();
    for (i, pos) in roots.iter().zip([p1, p2]) {
        let f = &mut m.fighters[*i];
        f.pos = pos;
        f.ground_level = pos.y;
    }
    m.fighters[roots[1]].life = 0.0;
}

fn into_round_two(m: &mut Match) {
    let mut ticks = 0;
    while !(m.round_no == 2 && m.round_state == 0) {
        if m.round_state >= 3 {
            m.zoom = m.zoom_max;
        }
        m.tick(&[0, 0]);
        ticks += 1;
        assert!(
            ticks < 3000,
            "round 2 never started: round {} state {}",
            m.round_no,
            m.round_state
        );
    }
}

fn round_two_opens_like_round_one(arena: bool, p1: V2, p2: V2, spawn: (f32, f32)) {
    let (mut m, first) = open(arena);
    let label = if arena { "arena" } else { "flat" };
    assert!((m.stage.p1x - spawn.0).abs() <= 0.01, "{label}: p1 spawn");
    assert!((m.stage.p2x - spawn.1).abs() <= 0.01, "{label}: p2 spawn");

    move_far_and_ko(&mut m, p1, p2);
    into_round_two(&mut m);
    let second = snap(&m);

    assert_eq!(first, second, "{label}: round 2 does not open like round 1");
    assert!(
        (second.roots[0].0.x - spawn.0).abs() <= 0.5,
        "{label}: p1 not at spawn"
    );
    assert!(
        (second.roots[1].0.x - spawn.1).abs() <= 0.5,
        "{label}: p2 not at spawn"
    );
    assert_eq!(
        second.projectiles, 0,
        "{label}: round 2 kept round 1 projectiles"
    );
}

#[test]
fn flat_round_two_opens_like_round_one() {
    round_two_opens_like_round_one(
        false,
        V2 {
            x: -190.0,
            y: -60.0,
        },
        V2 { x: 190.0, y: -20.0 },
        (-70.0, 70.0),
    );
}

#[test]
fn power_arena_round_two_opens_like_round_one() {
    round_two_opens_like_round_one(
        true,
        V2 { x: 8.0, y: -102.0 },
        V2 { x: 480.0, y: 0.0 },
        (ARENA_P1_SPAWN_X, ARENA_P2_SPAWN_X),
    );
}
