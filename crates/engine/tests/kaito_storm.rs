use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::{Event, Match};
use ume_engine::native::{compile_native, NativeChar};

fn fighter(name: &str) -> Arc<ume_engine::chardef::CharDef> {
    let path = format!(
        "{}/../../assets/chars/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let character: NativeChar =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    Arc::new(compile_native(&character, 1, true))
}

fn match_with_close_opponent() -> Match {
    let mut game = Match::new(common_def(), None, flat_stage());
    game.add_player(fighter("kaito"), 0, 0, 1);
    game.add_player(fighter("shion"), 1, 0, 2);
    game.start_round();
    for _ in 0..600 {
        if game.round_state == 2 && game.fighters[0].ctrl {
            break;
        }
        game.tick(&[0, 0]);
    }
    game.fighters[0].pos.x = -32.0;
    game.fighters[1].pos.x = 32.0;
    game.fighters[0].facing = 1.0;
    game.fighters[1].facing = -1.0;
    game
}

#[test]
fn kaito_walk_visits_six_frames_and_stays_on_the_floor() {
    let mut game = match_with_close_opponent();
    game.fighters[1].pos.x = 400.0;
    let start = game.fighters[0].pos.x;
    let mut frames = std::collections::BTreeSet::new();
    let mut max_ground_error = 0.0_f32;
    for _ in 0..48 {
        game.tick(&[k::R, 0]);
        let fighter = &game.fighters[0];
        if fighter.state_no == 20 {
            frames.insert(fighter.elem);
            max_ground_error = max_ground_error.max(fighter.pos.y.abs());
        }
    }
    assert_eq!(
        frames.len(),
        6,
        "walk animation did not visit its six frames"
    );
    assert!(
        game.fighters[0].pos.x > start,
        "Kaito did not advance while walking"
    );
    assert!(max_ground_error < 0.01, "walking left the floor");
}

#[test]
fn gale_fist_motion_creates_a_projectile_and_hits() {
    let mut game = match_with_close_opponent();
    let before = game.fighters[1].life;
    let inputs = [k::D, k::D | k::R, k::R, k::R | k::X];
    let mut projectile = false;
    for tick in 0..100 {
        game.tick(&[inputs.get(tick).copied().unwrap_or(0), 0]);
        projectile |= !game.projectiles.is_empty();
    }
    assert!(projectile, "Gale Fist did not create its projectile");
    assert!(game.fighters[1].life < before, "Gale Fist did not connect");
}

#[test]
fn rising_tempest_motion_connects_from_close_range() {
    let mut game = match_with_close_opponent();
    let inputs = [k::R, k::D, k::D | k::R, k::R | k::Y];
    for tick in 0..100 {
        game.tick(&[inputs.get(tick).copied().unwrap_or(0), 0]);
    }
    assert!(
        game.fighters[1].received_hits >= 1,
        "Rising Tempest failed to connect"
    );
}

#[test]
fn thunderstep_motion_connects_from_close_range() {
    let mut game = match_with_close_opponent();
    let inputs = [k::D, k::D | k::L, k::L, k::L | k::A];
    for tick in 0..100 {
        game.tick(&[inputs.get(tick).copied().unwrap_or(0), 0]);
    }
    assert!(
        game.fighters[1].received_hits >= 1,
        "Thunderstep failed to connect"
    );
}

#[test]
fn typhoon_barrage_spends_meter_and_connects() {
    let mut game = match_with_close_opponent();
    game.fighters[0].power = 3000.0;
    let before = game.fighters[1].life;
    let inputs = [
        k::D,
        k::D | k::R,
        k::R,
        k::D,
        k::D | k::R,
        k::R,
        k::R | k::X,
    ];
    for tick in 0..120 {
        game.tick(&[inputs.get(tick).copied().unwrap_or(0), 0]);
    }
    assert!(
        game.fighters[0].power < 3000.0,
        "Typhoon Barrage did not spend power"
    );
    assert!(
        game.fighters[1].life < before,
        "Typhoon Barrage did not connect"
    );
}

#[test]
fn awakened_native_sprites_recolor_only_the_hair_region() {
    let path = format!(
        "{}/../../assets/chars/kaito.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let character: NativeChar =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let definition =
        ume_engine::native::compile_native_with(&character, 1, true, Some(&[0, 20, 196, 1012]));
    let metadata_path = format!(
        "{}/../../assets/chars/sprites/kaito/meta.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let metadata: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(metadata_path).unwrap()).unwrap();
    for (animation, frame_index, pose) in [
        (0, 0, "idle0"),
        (20, 1, "walk1"),
        (196, 0, "powerup0"),
        (196, 1, "powerup1"),
        (1012, 0, "rise0"),
        (1012, 1, "inline_0172e9bbe547734f"),
        (1012, 2, "rise1"),
        (1012, 3, "rise2"),
    ] {
        let frame = &definition.anims[&animation].frames[frame_index];
        let normal = definition.sprites[&(frame.group, frame.index)]
            .image
            .as_ref()
            .unwrap();
        let awakened = definition.sprites[&(2, frame.index)]
            .image
            .as_ref()
            .unwrap();
        let region: [u32; 4] =
            serde_json::from_value(metadata["frames"][pose]["awakeningHair"].clone()).unwrap();
        let (mut changed_inside, mut changed_outside) = (0, 0);
        for (index, (a, b)) in normal
            .rgba
            .chunks_exact(4)
            .zip(awakened.rgba.chunks_exact(4))
            .enumerate()
        {
            if a == b {
                continue;
            }
            let (x, y) = (index as u32 % normal.w, index as u32 / normal.w);
            if (region[0]..region[2]).contains(&x) && (region[1]..region[3]).contains(&y) {
                changed_inside += 1;
            } else {
                changed_outside += 1;
            }
        }
        assert!(changed_inside > 100, "{pose} hair should change visibly");
        assert_eq!(
            changed_outside, 0,
            "{pose} awakening should not recolor pixels outside the hair region"
        );
    }
}

#[test]
fn dragon_command_emits_voice_and_hits_from_either_facing() {
    for facing in [1.0, -1.0] {
        let mut game = Match::new(common_def(), None, flat_stage());
        game.add_player(fighter("kaito"), 0, 0, 1);
        game.add_player(fighter("shion"), 1, 0, 2);
        game.start_round();
        for _ in 0..600 {
            if game.round_state == 2 && game.fighters[0].ctrl {
                break;
            }
            game.tick(&[0, 0]);
        }
        game.fighters[0].pos.x = -100.0 * facing;
        game.fighters[1].pos.x = 100.0 * facing;
        game.fighters[0].facing = facing;
        game.fighters[1].facing = -facing;
        let forward = if facing > 0.0 { k::R } else { k::L };
        let inputs = [k::D, k::D | forward, forward, forward | k::Y];
        let life = game.fighters[1].life;
        game.events.clear();
        let mut projectile = false;
        let mut voice = false;
        for tick in 0..100 {
            game.tick(&[inputs.get(tick).copied().unwrap_or(0), 0]);
            projectile |= !game.projectiles.is_empty();
            voice |= game.events.iter().any(|event| {
                matches!(
                    event,
                    Event::Sound {
                        group: 1000,
                        index: 0,
                        own: true,
                        ..
                    }
                )
            });
            game.events.clear();
        }
        assert!(projectile, "dragon projectile missing when facing {facing}");
        assert!(voice, "dragon voice event missing when facing {facing}");
        assert!(
            game.fighters[1].life < life,
            "dragon did not hit when facing {facing}"
        );
    }
}

#[test]
fn storm_rush_connects_three_hits() {
    let mut game = Match::new(common_def(), None, flat_stage());
    game.add_player(fighter("kaito"), 0, 0, 1);
    game.add_player(fighter("shion"), 1, 0, 2);
    game.start_round();
    for _ in 0..600 {
        if game.round_state == 2 && game.fighters[0].ctrl {
            break;
        }
        game.tick(&[0, 0]);
    }
    game.fighters[0].pos.x = -32.0;
    game.fighters[1].pos.x = 32.0;
    game.fighters[0].facing = 1.0;
    game.fighters[1].facing = -1.0;
    let inputs = [k::D, k::D | k::L, k::L, k::L | k::X];
    for tick in 0..100 {
        game.tick(&[inputs.get(tick).copied().unwrap_or(0), 0]);
    }
    assert_eq!(game.fighters[1].received_hits, 3);
}
