use crate::support;

use support::{duel, mugen_char, ticks};
use ume_engine::matchs::{ARENA_HALF_WIDTH, ARENA_PLATFORMS, ENGINE_STATE_ARENA_EXIT};

#[test]
fn descending_fighter_lands_on_platform() {
    let fighter = mugen_char("arena fighter", "");
    let mut battle = duel(fighter.clone(), fighter);
    battle.enable_power_arena();
    let (left, right, top) = ARENA_PLATFORMS[0];
    let player = &mut battle.fighters[0];
    player.pos.x = (left + right) * 0.5;
    player.pos.y = top - 1.0;
    player.vel.y = 4.0;
    player.physics = 'A';
    player.state_type = 'A';

    ticks(&mut battle, 1);
    assert_eq!(battle.fighters[0].pos.y, top);
    assert_eq!(battle.fighters[0].ground_level, top);

    assert_eq!(battle.round_state, 2);
}

#[test]
fn arena_exit_waits_for_a_visible_fall_outside_the_base() {
    let fighter = mugen_char("arena fighter", "");
    let mut battle = duel(fighter.clone(), fighter);
    battle.enable_power_arena();
    let player = &mut battle.fighters[0];
    player.pos.x = -ARENA_HALF_WIDTH - 1.0;
    player.pos.y = -28.0;
    player.vel.y = 1.0;
    player.physics = 'A';
    player.state_type = 'A';

    ticks(&mut battle, 2);
    assert!(battle.fighters[0].alive);
    assert_ne!(battle.fighters[0].state_no, ENGINE_STATE_ARENA_EXIT);
    assert_eq!(battle.winner_side, -1);

    for _ in 0..60 {
        if battle.fighters[0].state_no == ENGINE_STATE_ARENA_EXIT { break; }
        ticks(&mut battle, 1);
    }
    assert_eq!(battle.fighters[0].state_no, ENGINE_STATE_ARENA_EXIT);
    ticks(&mut battle, 3);
    assert!(battle.fighters[0].alive);
    assert!(battle.fighters[0].angle_draw);
    assert!(!battle.fighters[0].ctrl);
    assert_eq!(battle.winner_side, -1);

    ticks(&mut battle, 60);
    let player = &battle.fighters[0];
    assert!(!player.alive);
    assert!(player.pos.x.abs() > ARENA_HALF_WIDTH + player.front_width().max(player.back_width()));
    assert!(player.pos.y >= 184.0);
    assert_eq!(battle.winner_side, 1);
    assert_eq!(battle.announce.as_ref().unwrap().text, "RING OUT");
}

#[test]
fn arena_fighter_can_return_after_crossing_the_edge_in_midair() {
    let fighter = mugen_char("arena fighter", "");
    let mut battle = duel(fighter.clone(), fighter);
    battle.enable_power_arena();
    let player = &mut battle.fighters[0];
    player.pos.x = -ARENA_HALF_WIDTH - 4.0;
    player.pos.y = -28.0;
    player.vel.x = 4.0;
    player.vel.y = 1.0;
    player.physics = 'A';
    player.state_type = 'A';

    ticks(&mut battle, 30);
    let player = &battle.fighters[0];
    assert!(player.pos.x > -ARENA_HALF_WIDTH);
    assert_eq!(player.pos.y, 0.0);
    assert!(player.alive);
    assert_ne!(player.state_no, ENGINE_STATE_ARENA_EXIT);
    assert_eq!(battle.winner_side, -1);
}

#[test]
fn arena_camera_reveals_both_edges_when_fighters_separate() {
    assert!(ARENA_HALF_WIDTH * 2.0 >= 852.0 * 1.30);
    assert_eq!(ARENA_PLATFORMS.len(), 40);
    let fighter = mugen_char("arena fighter", "");
    let mut battle = duel(fighter.clone(), fighter);
    battle.enable_power_arena();
    battle.fighters[0].pos.x = -ARENA_HALF_WIDTH + 6.0;
    battle.fighters[1].pos.x = ARENA_HALF_WIDTH - 6.0;

    for _ in 0..90 {
        battle.update_camera();
    }
    let roof = ARENA_PLATFORMS.last().unwrap().2 + battle.stage.zoffset - battle.camera.y;
    assert!(roof >= 17.0);
    assert!(battle.camera_scale() < 0.5);
    assert!(battle.camera.x - battle.view_w * 0.5 < -ARENA_HALF_WIDTH);
    assert!(battle.camera.x + battle.view_w * 0.5 > ARENA_HALF_WIDTH);
}

#[test]
fn imported_fighter_can_climb_the_eight_platform_tiers() {
    use std::sync::Arc;
    use ume_engine::input::k;
    use ume_engine::loader::load_mugen_char;
    use ume_formats::vfs::{add_zip, find_char_defs, Vfs};

    let mut vfs = Vfs::new();
    add_zip(&mut vfs, include_bytes!("../../../assets/mugen/elecbyte.zip"), "").unwrap();
    let path = find_char_defs(&vfs).into_iter().next().unwrap();
    let fighter = Arc::new(load_mugen_char(&vfs, &path, 1, false).unwrap());
    let mut battle = duel(fighter.clone(), fighter);
    battle.enable_power_arena();
    battle.fighters[0].pos.x = -410.0;
    for target in [-50.0, -102.0, -154.0, -206.0, -258.0, -310.0, -362.0, -414.0] {
        battle.tick(&[k::U, 0]);
        ticks(&mut battle, 75);
        assert_eq!(battle.fighters[0].ground_level, target);
        assert_eq!(battle.fighters[0].pos.y, target);
        assert!(battle.fighters[0].ctrl);
    }
}

#[test]
fn arena_camera_frames_the_base_and_top_tier_together() {
    let fighter = mugen_char("arena fighter", "");
    let mut battle = duel(fighter.clone(), fighter);
    battle.enable_power_arena();
    battle.fighters[0].pos.y = ARENA_PLATFORMS.last().unwrap().2;
    battle.fighters[1].pos.y = 0.0;
    for _ in 0..90 {
        battle.update_camera();
    }

    let screen_y = |side: usize| battle.fighters[side].pos.y + battle.stage.zoffset - battle.camera.y;
    assert!(screen_y(0) >= 133.0, "upper fighter needs headroom");
    assert!(screen_y(1) <= battle.view_h - 33.0, "base must remain visible");
}
