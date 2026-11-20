mod support;

use support::{duel, mugen_char, ticks};
use ume_engine::matchs::{ARENA_HALF_WIDTH, ARENA_PLATFORMS};

#[test]
fn descending_fighter_lands_on_platform_and_leaving_the_base_loses() {
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

    battle.fighters[0].pos.x = -ARENA_HALF_WIDTH - 1.0;
    ticks(&mut battle, 2);
    assert!(!battle.fighters[0].alive);
    assert_eq!(battle.winner_side, 1);
}

#[test]
fn arena_camera_reveals_both_edges_when_fighters_separate() {
    let fighter = mugen_char("arena fighter", "");
    let mut battle = duel(fighter.clone(), fighter);
    battle.enable_power_arena();
    battle.fighters[0].pos.x = -ARENA_HALF_WIDTH + 6.0;
    battle.fighters[1].pos.x = ARENA_HALF_WIDTH - 6.0;

    for _ in 0..90 {
        battle.update_camera();
    }
    assert!(battle.camera_scale() < 0.5);
    assert!(battle.camera.x - battle.view_w * 0.5 < -ARENA_HALF_WIDTH);
    assert!(battle.camera.x + battle.view_w * 0.5 > ARENA_HALF_WIDTH);
}

#[test]
fn imported_fighter_stays_on_a_platform_after_its_landing_controller() {
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
    battle.fighters[0].pos.x = -200.0;
    battle.tick(&[k::U, 0]);
    ticks(&mut battle, 75);
    assert_eq!(battle.fighters[0].ground_level, -50.0);
    assert_eq!(battle.fighters[0].pos.y, -50.0);
    assert!(battle.fighters[0].ctrl);
}
