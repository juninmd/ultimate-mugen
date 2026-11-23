use crate::support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};
use ume_engine::stage_objects::{
    Breakable, ObjectEvent, PickupSpec, PICKUP_ARM_TICKS, PICKUP_LIFETIME,
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
fn a_strike_damages_a_breakable_once_per_swing() {
    let mut m = street(true);
    m.fighters[1].pos.x = m.fighters[0].pos.x - 150.0;
    let b = crate_ahead(&m, 10_000.0);
    m.objects.add(b);
    enter(&mut m, 0, JAB_STATE);
    run(&mut m, 40);
    let after_one = m.objects.breakables[0].damage_ratio();
    assert!(after_one > 0.0, "the jab reaches the crate");
    run(&mut m, 40);
    assert_eq!(
        m.objects.breakables[0].damage_ratio(),
        after_one,
        "no damage without a new swing"
    );
    assert!(m
        .objects
        .events
        .iter()
        .any(|e| matches!(e, ObjectEvent::Struck { index: 0, .. })));
}

#[test]
fn a_far_lane_keeps_the_breakable_out_of_reach() {
    let mut m = street(true);
    let mut b = crate_ahead(&m, 50.0);
    b.lane = m.fighters[0].lane + 30.0;
    b.depth = 2.0;
    m.objects.add(b);
    enter(&mut m, 0, JAB_STATE);
    run(&mut m, 40);
    assert_eq!(m.objects.breakables[0].damage_ratio(), 0.0);
}

#[test]
fn breaking_drops_a_pickup_that_heals_the_hero_and_scores() {
    let mut m = street(true);
    m.fighters[1].pos.x = m.fighters[0].pos.x - 150.0;
    let b = crate_ahead(&m, 60.0);
    m.objects.add(b);
    m.fighters[0].life = m.fighters[0].life_max * 0.4;
    assert!(jab_until_broken(&mut m, 12) < 12, "the crate breaks");
    assert!(m
        .objects
        .events
        .iter()
        .any(|e| matches!(e, ObjectEvent::Dropped { kind: 3, .. })));
    assert!(
        m.fighters[0].score_total >= 100.0,
        "breaking pays the crate score"
    );
    let before = m.fighters[0].life;
    let drop_x = m
        .objects
        .pickups
        .first()
        .map(|p| p.pos.x)
        .unwrap_or(m.fighters[0].pos.x);
    run(&mut m, PICKUP_ARM_TICKS as usize + 40);
    m.fighters[0].pos.x = drop_x;
    run(&mut m, 4);
    assert!(
        m.objects.pickups.is_empty(),
        "walking over the meal collects it"
    );
    let gained = m.fighters[0].life - before;
    assert!(
        (gained - m.fighters[0].life_max * MEAL.heal).abs() < 1.0,
        "healed {gained}"
    );
    assert!(m.fighters[0].score_total >= 400.0);
    assert!(m
        .objects
        .events
        .iter()
        .any(|e| matches!(e, ObjectEvent::Collected { .. })));
}

#[test]
fn healing_never_exceeds_the_life_bar_and_villains_cannot_eat() {
    let mut m = street(true);
    let b = crate_ahead(&m, 1.0);
    m.objects.add(b);
    jab_until_broken(&mut m, 4);
    let x = m.objects.pickups[0].pos.x;
    m.fighters[0].pos.x = x - 120.0;
    m.fighters[1].pos.x = x;
    m.fighters[1].life *= 0.5;
    let villain_life = m.fighters[1].life;
    run(&mut m, PICKUP_ARM_TICKS as usize + 30);
    assert_eq!(
        m.objects.pickups.len(),
        1,
        "the villain walks over it and nothing happens"
    );
    assert_eq!(m.fighters[1].life, villain_life);
    m.fighters[1].pos.x = x + 150.0;
    m.fighters[0].pos.x = x;
    run(&mut m, 3);
    assert!(m.objects.pickups.is_empty());
    assert!(m.fighters[0].life <= m.fighters[0].life_max);
}

#[test]
fn uncollected_pickups_expire() {
    let mut m = street(true);
    let b = crate_ahead(&m, 1.0);
    m.objects.add(b);
    jab_until_broken(&mut m, 4);
    let x = m.objects.pickups[0].pos.x;
    m.fighters[0].pos.x = x - 150.0;
    m.fighters[1].pos.x = x - 100.0;
    m.training = true;
    run(&mut m, PICKUP_LIFETIME as usize + 5);
    assert!(m.objects.pickups.is_empty());
}

#[test]
fn stage_objects_are_deterministic_and_idle_matches_are_untouched() {
    let play = || {
        let mut m = street(true);
        let b = crate_ahead(&m, 60.0);
        m.objects.add(b);
        jab_until_broken(&mut m, 12);
        run(&mut m, 120);
        (
            m.fighters[0].life,
            m.fighters[0].pos.x,
            m.objects.pickups.first().map(|p| (p.pos.x, p.pos.y)),
            m.rng,
        )
    };
    assert_eq!(play(), play());
    let mut plain = street(true);
    run(&mut plain, 60);
    assert!(plain.objects.is_idle());
    assert!(plain.objects.events.is_empty());
}

#[test]
fn breaking_a_crate_while_the_arena_scrolls_does_not_abort_the_tick() {
    let mut m = street(true);
    m.stage.bound_left = 539.0;
    m.stage.bound_right = 100.0;
    run(&mut m, 5);
    m.fighters[1].pos.x = m.fighters[0].pos.x - 150.0;
    let b = crate_ahead(&m, 1.0);
    m.objects.add(b);
    enter(&mut m, 0, JAB_STATE);
    run(&mut m, 40);
    assert!(m.objects.breakables[0].broken, "the crate breaks");
    let x = m
        .objects
        .events
        .iter()
        .find_map(|e| match e {
            ObjectEvent::Dropped { x, .. } => Some(*x),
            _ => None,
        })
        .expect("the pickup is dropped");
    assert!(x.is_finite(), "the pickup lands somewhere real, got {x}");
}
