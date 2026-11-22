mod support;

use support::{duel, mugen_char, ticks};
use ume_engine::matchs::{Event, Match, ARENA_PLATFORMS, ELIXIR_LIFE_FACTOR};

fn arena() -> Match {
    let fighter = mugen_char("elixir fighter", "");
    let mut m = duel(fighter.clone(), fighter);
    m.enable_power_arena();
    m
}

fn stand_at_top(m: &mut Match, i: usize) {
    let (x, y) = {
        let e = m.elixir.as_ref().unwrap();
        (e.x, e.y)
    };
    m.fighters[i].pos.x = x;
    m.fighters[i].pos.y = y;
    m.fighters[i].vel.y = 0.0;
}

fn drank(m: &mut Match) -> usize {
    m.events.iter().filter(|e| matches!(e, Event::Elixir { .. })).count()
}

#[test]
fn elixir_waits_on_the_highest_platform() {
    let m = arena();
    let e = m.elixir.as_ref().unwrap();
    assert!(!e.taken());
    assert_eq!(e.y, ARENA_PLATFORMS.last().unwrap().2);
    assert!(ARENA_PLATFORMS.iter().all(|p| p.2 >= e.y));
}

#[test]
fn first_to_arrive_doubles_max_life_and_fills_it() {
    let mut m = arena();
    let base = m.fighters[0].life_max;
    m.fighters[0].life = base * 0.3;
    stand_at_top(&mut m, 0);
    ticks(&mut m, 1);
    assert_eq!(m.fighters[0].life_max, base * ELIXIR_LIFE_FACTOR);
    assert_eq!(m.fighters[0].life, m.fighters[0].life_max);
    assert!(m.elixir.as_ref().unwrap().taken());
    assert_eq!(drank(&mut m), 1);
}

#[test]
fn only_one_fighter_gets_it_even_when_both_touch_at_once() {
    let mut m = arena();
    let base = m.fighters[1].life_max;
    stand_at_top(&mut m, 0);
    stand_at_top(&mut m, 1);
    ticks(&mut m, 3);
    assert_eq!(m.fighters[0].life_max, base * ELIXIR_LIFE_FACTOR);
    assert_eq!(m.fighters[1].life_max, base);
    assert_eq!(drank(&mut m), 1);
}

#[test]
fn an_enemy_can_win_the_race() {
    let mut m = arena();
    let base = m.fighters[0].life_max;
    stand_at_top(&mut m, 1);
    ticks(&mut m, 1);
    assert_eq!(m.fighters[1].life_max, base * ELIXIR_LIFE_FACTOR);
    assert_eq!(m.fighters[0].life_max, base);
}

#[test]
fn a_lower_tier_does_not_count() {
    let mut m = arena();
    let base = m.fighters[0].life_max;
    let e = m.elixir.as_ref().unwrap().clone();
    m.fighters[0].pos.x = e.x;
    m.fighters[0].pos.y = e.y + 52.0;
    ticks(&mut m, 1);
    assert_eq!(m.fighters[0].life_max, base);
    assert!(!m.elixir.as_ref().unwrap().taken());
}

#[test]
fn doubled_life_is_given_back_when_the_next_round_starts() {
    let mut m = arena();
    let base = m.fighters[0].life_max;
    stand_at_top(&mut m, 0);
    ticks(&mut m, 1);
    m.start_round();
    ticks(&mut m, 2);
    assert_eq!(m.fighters[0].life_max, base);
    assert!(!m.elixir.as_ref().unwrap().taken());
}
