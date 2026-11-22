mod support;

use support::{mugen_char, ticks};
use ume_engine::matchs::{arena_tiles, Match, ARENA_HALF_WIDTH, SWARM_MAX_TOTAL};

fn swarm_match(team: usize, slots: usize, total: u32) -> Match {
    let fighter = mugen_char("swarm fighter", "");
    let mut m = Match::new(ume_engine::common::common_def(), None, ume_engine::common::flat_stage());
    for k in 0..team.max(slots) {
        if k < team {
            m.add_player(fighter.clone(), 0, 0, 1);
        }
        if k < slots {
            m.add_player(fighter.clone(), 1, 0, 2);
        }
    }
    m.team_simul = true;
    m.start_round();
    m.enable_power_arena();
    m.enable_swarm(total);
    while m.round_state != 2 {
        m.tick(&vec![0; team + slots]);
    }
    m
}

fn knock_out(m: &mut Match, side: usize, count: usize) {
    for f in m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == side && f.alive).take(count) {
        f.life = 0.0;
    }
}

#[test]
fn map_grows_with_the_crowd_and_stays_a_whole_number_of_tiles() {
    assert_eq!(arena_tiles(2), 1);
    assert_eq!(arena_tiles(7), 3);
    assert_eq!(arena_tiles(60), 7);
    for n in 1..200 {
        assert_eq!(arena_tiles(n) % 2, 1);
    }
    let m = swarm_match(3, 10, 100);
    assert_eq!(m.arena_half, ARENA_HALF_WIDTH * arena_tiles(13) as f32);
    assert_eq!(m.stage.bound_right, m.arena_half);
}

#[test]
fn foes_spawn_apart_from_the_squad_inside_the_map() {
    let m = swarm_match(5, 12, 100);
    let team: Vec<f32> = m.roots().filter(|f| f.side == 0).map(|f| f.pos.x).collect();
    assert!(team.iter().all(|x| x.abs() < 130.0));
    for foe in m.roots().filter(|f| f.side == 1) {
        assert!(foe.pos.x.abs() < m.arena_half);
    }
    let far = m.roots().filter(|f| f.side == 1 && f.pos.x.abs() > 150.0).count();
    assert!(far >= 8, "most foes start away from the squad, got {far}");
}

#[test]
fn fallen_foes_are_replaced_until_the_total_is_spent_then_the_squad_wins() {
    let mut m = swarm_match(1, 3, 7);
    let mut guard = 0;
    while m.round_state == 2 && guard < 6000 {
        knock_out(&mut m, 1, 1);
        ticks(&mut m, 1);
        guard += 1;
        let alive = m.roots().filter(|f| f.side == 1 && f.alive && f.life > 0.0).count();
        assert!(alive <= 3);
    }
    let swarm = m.swarm.as_ref().unwrap();
    assert_eq!(swarm.kills, 7);
    assert_eq!(swarm.spawned, 7);
    assert_eq!(m.winner_side, 0);
}

#[test]
fn round_does_not_end_while_reserves_remain() {
    let mut m = swarm_match(1, 2, 10);
    knock_out(&mut m, 1, 2);
    ticks(&mut m, 20);
    assert_eq!(m.round_state, 2);
    assert!(m.swarm_pending());
}

#[test]
fn camera_follows_the_squad_not_the_distant_crowd() {
    let mut m = swarm_match(2, 6, 100);
    for _ in 0..120 {
        m.update_camera();
    }
    let team_x: Vec<f32> = m.roots().filter(|f| f.side == 0).map(|f| f.pos.x).collect();
    let (lo, hi) = (team_x.iter().cloned().fold(f32::MAX, f32::min), team_x.iter().cloned().fold(f32::MIN, f32::max));
    assert!(m.camera.x - m.view_w / 2.0 <= lo && m.camera.x + m.view_w / 2.0 >= hi);
    assert!(m.view_w < 1000.0, "zoom must not stretch to show every foe");
}

#[test]
fn squad_wipe_loses_the_round_even_with_reserves() {
    let mut m = swarm_match(2, 3, 100);
    knock_out(&mut m, 0, 2);
    ticks(&mut m, 5);
    assert_eq!(m.winner_side, 1);
}

#[test]
fn total_is_capped() {
    let m = swarm_match(1, 2, 5000);
    assert_eq!(m.swarm.as_ref().unwrap().total, SWARM_MAX_TOTAL);
}

#[test]
fn mass_knockout_respawns_every_slot_without_double_counting() {
    let mut m = swarm_match(1, 16, 100);
    knock_out(&mut m, 1, 16);
    ticks(&mut m, 200);
    let swarm = m.swarm.as_ref().unwrap();
    assert_eq!(swarm.kills, 16);
    assert_eq!(swarm.spawned, 32);
    let alive = m.roots().filter(|f| f.side == 1 && f.alive && f.life > 0.0).count();
    assert_eq!(alive, 16);
}

#[test]
fn horde_foes_cannot_steal_the_elixir_but_the_squad_can() {
    let mut m = swarm_match(1, 3, 100);
    let (x, y) = {
        let e = m.elixir.as_ref().unwrap();
        (e.x, e.y)
    };
    let foe = m.roots().position(|f| f.side == 1).unwrap();
    let foe = m.roots().nth(foe).unwrap().id;
    let idx = m.fighters.iter().position(|f| f.id == foe).unwrap();
    m.fighters[idx].pos.x = x;
    m.fighters[idx].pos.y = y;
    ticks(&mut m, 2);
    assert!(!m.elixir.as_ref().unwrap().taken());
    m.fighters[0].pos.x = x;
    m.fighters[0].pos.y = y;
    ticks(&mut m, 2);
    assert!(m.elixir.as_ref().unwrap().taken());
}
