use std::sync::Arc;

use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::fighter::{asf, Fighter, HitDefData, V2};
use ume_engine::matchs::{AnimCursor, Match, Projectile, ARENA_P1_SPAWN_X, ARENA_P2_SPAWN_X};
use ume_native::{compile_native, NativeChar};

fn native(id: &str) -> Arc<CharDef> {
    let path = format!(
        "{}/../../assets/chars/{id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

type RootSnap = (V2, V2, f32, f32, i32, bool, bool, u64, [i32; 5], [f64; 5]);

#[derive(Debug, PartialEq)]
struct Snap {
    roots: Vec<RootSnap>,
    camera: V2,
    zoom: f32,
    view: (f32, f32),
    projectiles: usize,
    helpers: usize,
}

fn snap(m: &Match) -> Snap {
    let roots = m
        .root_indices()
        .into_iter()
        .map(|i| {
            let f = &m.fighters[i];
            (
                f.pos,
                f.vel,
                f.facing,
                f.ground_level,
                f.state_no,
                f.ctrl,
                f.pos_freeze,
                f.asserts,
                f.sysvars,
                f.sysfvars,
            )
        })
        .collect();
    Snap {
        roots,
        camera: m.camera,
        zoom: m.zoom,
        view: (m.view_w, m.view_h),
        projectiles: m.projectiles.len(),
        helpers: m.fighters.iter().filter(|f| f.is_helper).count(),
    }
}

#[rustfmt::skip]
fn loop_projectile(f: &Fighter) -> Projectile {
    Projectile {
        owner: f.id, side: f.side, teamside: f.side, root: f.root, tally: f.root, id: 9000,
        cur: AnimCursor::new(f.def.clone(), 0), hit_anim: -1, rem_anim: -1, cancel_anim: -1,
        pos: V2 { x: 0.0, y: -300.0 }, vel: V2::default(), accel: V2::default(),
        vel_mul: V2 { x: 1.0, y: 1.0 }, rem_vel: V2::default(), facing: f.facing,
        hits: 1, miss_time: 0, miss_cd: 0, priority: 1, spr_priority: 3,
        edge_bound: 1000.0, height_bound: [-1000.0, 1000.0], remove_time: -1, time: 0,
        hitdef: HitDefData::default(), dying: false, guard_dist: 0.0, scale: [1.0, 1.0],
        remove: false, lscl: f.lscl, remove_on_hit: false, stage_bound: 1000.0, super_move: 0, pause_move: 0,
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
    m.rounds_to_win = 3;
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
    for i in roots.iter().copied() {
        let def = m.fighters[i].def.clone();
        m.change_state(i, 50, def);
    }
    for (side, i) in roots.iter().enumerate() {
        let f = &mut m.fighters[*i];
        f.vel = V2 {
            x: if side == 0 { 4.0 } else { -3.0 },
            y: -2.0,
        };
        f.facing = if side == 0 { -1.0 } else { 1.0 };
        f.pos_freeze = true;
        f.asserts = asf::INVISIBLE;
        f.sysvars = [7; 5];
        f.sysfvars = [2.5; 5];
    }
    let projectile = loop_projectile(&m.fighters[roots[0]]);
    m.projectiles.push(projectile);
    let mut helper = m.fighters[roots[0]].clone();
    helper.id = m.next_id;
    m.next_id += 1;
    helper.is_helper = true;
    helper.root = m.fighters[roots[0]].id;
    helper.parent = Some(helper.root);
    helper.preserve = false;
    m.fighters.push(helper);
    assert!(
        !m.projectiles.is_empty() && m.fighters.iter().any(|f| f.is_helper),
        "leftovers are live"
    );
    m.fighters[roots[1]].life = 0.0;
}

fn into_round(m: &mut Match, round: i32) {
    let mut ticks = 0;
    while !(m.round_no == round && m.round_state == 0) {
        if m.round_state >= 3 {
            m.zoom = m.zoom_max;
        }
        m.tick(&[0, 0]);
        ticks += 1;
        assert!(ticks < 3000, "round {round} never started");
    }
}

fn assert_reset(m: &Match) {
    for i in m.root_indices() {
        let f = &m.fighters[i];
        assert!(!f.pos_freeze && f.asserts == 0, "root {i} kept a flag");
        assert_eq!(
            (f.sysvars, f.sysfvars),
            ([0; 5], [0.0; 5]),
            "root {i} kept vars"
        );
    }
    assert!(
        m.projectiles.is_empty() && !m.fighters.iter().any(|f| f.is_helper),
        "round kept leftovers"
    );
}

fn rounds_open_like_round_one(arena: bool, p1: V2, p2: V2, spawn: (f32, f32)) {
    let (mut m, first) = open(arena);
    let label = if arena { "arena" } else { "flat" };
    let (a, b) = (m.stage.p1x, m.stage.p2x);
    assert!(
        (a - spawn.0).abs() <= 0.01 && (b - spawn.1).abs() <= 0.01,
        "{label}: spawn"
    );

    for round in [2, 3] {
        move_far_and_ko(&mut m, p1, p2);
        into_round(&mut m, round);
        let opened = snap(&m);
        assert_eq!(
            first, opened,
            "{label}: round {round} does not open like round 1"
        );
        assert!(
            (opened.roots[0].0.x - spawn.0).abs() <= 0.5,
            "{label}: p1 not at spawn"
        );
        assert!(
            (opened.roots[1].0.x - spawn.1).abs() <= 0.5,
            "{label}: p2 not at spawn"
        );
        assert_reset(&m);
    }
}

#[test]
fn flat_rounds_open_like_round_one() {
    rounds_open_like_round_one(
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
fn power_arena_rounds_open_like_round_one() {
    rounds_open_like_round_one(
        true,
        V2 { x: 8.0, y: -102.0 },
        V2 { x: 480.0, y: 0.0 },
        (ARENA_P1_SPAWN_X, ARENA_P2_SPAWN_X),
    );
}
