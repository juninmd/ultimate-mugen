use std::collections::BTreeSet;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("expected character JSON")?;
    let character: NativeChar = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let definition = Arc::new(compile_native(&character, 1, true));
    let mut game = Match::new(common_def(), None, flat_stage());
    game.add_player(definition.clone(), 0, 0, 1);
    game.add_player(definition.clone(), 1, 0, 2);
    game.start_round();
    for _ in 0..600 {
        game.tick(&[0, 0]);
        if game.round_state == 2 && game.fighters[0].ctrl {
            break;
        }
    }
    game.fighters[0].pos.x = -120.0;
    game.fighters[1].pos.x = 120.0;
    game.change_state(0, 100, definition);
    let mut frames = BTreeSet::new();
    let mut states = BTreeSet::new();
    let start = game.fighters[0].pos.x;
    let mut max_ground_error = 0.0_f32;
    for tick in 0..180 {
        game.tick(&[if tick < 60 { k::R } else { 0 }, 0]);
        let fighter = &game.fighters[0];
        assert!(fighter.pos.x.is_finite() && fighter.pos.y.is_finite());
        assert!(fighter.vel.x.is_finite() && fighter.vel.y.is_finite());
        states.insert(fighter.state_no);
        if fighter.state_no == 100 {
            frames.insert(fighter.elem);
            max_ground_error = max_ground_error.max(fighter.pos.y.abs());
        }
    }
    let fighter = &game.fighters[0];
    println!("states={states:?} run_frames={frames:?} displacement={} ground_error={max_ground_error} final_state={} final_y={} final_vx={}", fighter.pos.x - start, fighter.state_no, fighter.pos.y, fighter.vel.x);
    assert!(
        frames.len() >= 4,
        "running animation did not visit four frames"
    );
    assert!(fighter.pos.x > start, "runner did not move forward");
    assert!(max_ground_error < 0.01, "running left the floor");
    assert_eq!(
        fighter.state_no, 0,
        "runner did not return to neutral after release"
    );
    assert!(fighter.pos.y.abs() < 0.01 && fighter.vel.x.abs() < 0.01);
    Ok(())
}
