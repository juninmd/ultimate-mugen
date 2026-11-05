use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::{Event, Match};
use ume_engine::native::{compile_native, NativeChar};

fn fighter(name: &str) -> Arc<ume_engine::chardef::CharDef> {
    let path = format!("{}/../../assets/chars/{name}.json", env!("CARGO_MANIFEST_DIR"));
    let character: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    Arc::new(compile_native(&character, 1, true))
}

#[test]
fn awakened_idle_preserves_pixels_outside_the_hair_region() {
    let path = format!("{}/../../assets/chars/kaito.json", env!("CARGO_MANIFEST_DIR"));
    let character: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let definition = ume_engine::native::compile_native_with(&character, 1, true, Some(&[0]));
    let frame = &definition.anims[&0].frames[0];
    let normal = definition.sprites[&(frame.group, frame.index)].image.as_ref().unwrap();
    let awakened = definition.sprites[&(2, frame.index)].image.as_ref().unwrap();
    let metadata_path = format!("{}/../../assets/chars/sprites/kaito/meta.json", env!("CARGO_MANIFEST_DIR"));
    let metadata: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(metadata_path).unwrap()).unwrap();
    let region: [u32; 4] = serde_json::from_value(metadata["frames"]["idle0"]["awakeningHair"].clone()).unwrap();
    let (mut changed_inside, mut changed_outside) = (0, 0);
    for (index, (a, b)) in normal.rgba.chunks_exact(4).zip(awakened.rgba.chunks_exact(4)).enumerate() {
        if a == b { continue; }
        let (x, y) = (index as u32 % normal.w, index as u32 / normal.w);
        if (region[0]..region[2]).contains(&x) && (region[1]..region[3]).contains(&y) { changed_inside += 1; } else { changed_outside += 1; }
    }
    assert!(changed_inside > 100, "the hair should change visibly");
    assert_eq!(changed_outside, 0, "awakening should not recolor the rest of the sprite");
}

#[test]
fn dragon_command_emits_voice_and_hits_from_either_facing() {
    for facing in [1.0, -1.0] {
        let mut game = Match::new(common_def(), None, flat_stage());
        game.add_player(fighter("kaito"), 0, 0, 1);
        game.add_player(fighter("shion"), 1, 0, 2);
        game.start_round();
        for _ in 0..600 {
            if game.round_state == 2 && game.fighters[0].ctrl { break; }
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
            voice |= game.events.iter().any(|event| matches!(event, Event::Sound { group: 1000, index: 0, own: true, .. }));
            game.events.clear();
        }
        assert!(projectile, "dragon projectile missing when facing {facing}");
        assert!(voice, "dragon voice event missing when facing {facing}");
        assert!(game.fighters[1].life < life, "dragon did not hit when facing {facing}");
    }
}

#[test]
fn storm_rush_connects_three_hits() {
    let mut game = Match::new(common_def(), None, flat_stage());
    game.add_player(fighter("kaito"), 0, 0, 1);
    game.add_player(fighter("shion"), 1, 0, 2);
    game.start_round();
    for _ in 0..600 {
        if game.round_state == 2 && game.fighters[0].ctrl { break; }
        game.tick(&[0, 0]);
    }
    game.fighters[0].pos.x = -32.0;
    game.fighters[1].pos.x = 32.0;
    game.fighters[0].facing = 1.0;
    game.fighters[1].facing = -1.0;
    let inputs = [k::D, k::D | k::L, k::L, k::L | k::X];
    for tick in 0..100 { game.tick(&[inputs.get(tick).copied().unwrap_or(0), 0]); }
    assert_eq!(game.fighters[1].received_hits, 3);
}
