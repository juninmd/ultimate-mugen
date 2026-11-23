use crate::support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};
use ume_engine::stage_objects::{
    Breakable, Goal, ObjectEvent, PickupSpec, PICKUP_ARM_TICKS, PICKUP_LIFETIME,
};

const JAB_STATE: i32 = 201;
const MEAL: PickupSpec = PickupSpec {
    kind: 3,
    heal: 0.25,
    power: 0.1,
    score: 300.0,
};

fn street(foe: bool) -> Match {
    let path = format!(
        "{}/../../assets/chars/kaito.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let def = Arc::new(compile_native(&nc, 1, false));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    if foe {
        m.add_player(def, 1, 0, 2);
    }
    m.lanes.active = true;
    m.lanes.far = -40.0;
    m.lanes.near = 40.0;
    m.lanes.show_foe_hud = false;
    m.rounds_to_win = 1;
    m.start_round();
    let idle = [0u32; 2];
    while m.round_state != 2 {
        m.tick(&idle[..m.fighters.len()]);
    }
    for _ in 0..240 {
        if m.fighters.iter().all(|f| f.ctrl) {
            break;
        }
        m.tick(&idle[..m.fighters.len()]);
    }
    m
}

fn run(m: &mut Match, n: usize) {
    let idle = [0u32; 8];
    for _ in 0..n {
        let roots = m.fighters.iter().filter(|f| !f.is_helper).count();
        m.tick(&idle[..roots]);
    }
}

fn crate_ahead(m: &Match, life: f32) -> Breakable {
    let hero = &m.fighters[0];
    let mut b = Breakable::new(
        1,
        hero.pos.x + hero.facing * 44.0,
        hero.lane,
        14.0,
        30.0,
        life,
    );
    b.score = 100.0;
    b.drop = Some(MEAL);
    b
}

fn jab_until_broken(m: &mut Match, limit: usize) -> usize {
    let mut jabs = 0;
    while !m.objects.breakables[0].broken && jabs < limit {
        enter(m, 0, JAB_STATE);
        run(m, 40);
        jabs += 1;
    }
    jabs
}

#[test]
fn a_solid_breakable_blocks_the_floor_until_you_jump() {
    let mut m = street(true);
    let mut car = crate_ahead(&m, 5_000.0);
    car.half_width = 50.0;
    car.height = 40.0;
    car.solid = true;
    let x = car.x;
    m.objects.add(car);
    m.fighters[0].pos.x = x + 5.0;
    run(&mut m, 2);
    assert!(
        (m.fighters[0].pos.x - x).abs() >= 50.0,
        "pushed out of the car body"
    );
    m.fighters[0].pos.x = x;
    m.fighters[0].pos.y = -80.0;
    m.fighters[0].vel.y = -6.0;
    run(&mut m, 1);
    assert!(
        (m.fighters[0].pos.x - x).abs() < 50.0,
        "airborne above the roof is free"
    );
}

#[test]
fn a_break_all_goal_ends_the_round_for_the_hero_when_everything_is_wrecked() {
    let mut m = street(false);
    let b = crate_ahead(&m, 40.0);
    m.objects.add(b);
    m.objects.start_break_all_goal(30 * 60);
    run(&mut m, 5);
    assert!(matches!(m.objects.goal, Goal::BreakAll { .. }));
    assert_eq!(m.round_state, 2, "a round with no villains keeps running");
    jab_until_broken(&mut m, 12);
    run(&mut m, 4);
    assert!(matches!(m.objects.goal, Goal::Cleared { ticks_left } if ticks_left > 0));
    run(&mut m, 300);
    assert_eq!(m.winner_side, 0);
    assert!(m.match_over, "the bonus round hands the stage back");
}

#[test]
fn a_break_all_goal_times_out_without_a_loss() {
    let mut m = street(false);
    let b = crate_ahead(&m, 99_999.0);
    m.objects.add(b);
    m.objects.start_break_all_goal(90);
    run(&mut m, 400);
    assert_eq!(m.objects.goal, Goal::TimedOut);
    assert_eq!(m.winner_side, 0);
    assert!(m.match_over);
}
