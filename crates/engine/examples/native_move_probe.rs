use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().ok_or("expected character JSON and state number")?;
    let state: i32 = args.get(1).ok_or("expected state number")?.parse()?;
    let character: NativeChar = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let definition = Arc::new(compile_native(&character, 1, true));
    for distance in [24.0, 36.0, 48.0, 60.0] {
        let mut game = Match::new(common_def(), None, flat_stage());
        game.add_player(definition.clone(), 0, 0, 1);
        game.add_player(definition.clone(), 1, 0, 2);
        game.start_round();
        for _ in 0..600 {
            game.tick(&[0, 0]);
            if game.round_state == 2 && game.fighters[0].ctrl { break; }
        }
        game.fighters[0].pos.x = -distance / 2.0;
        game.fighters[1].pos.x = distance / 2.0;
        game.change_state(0, state, definition.clone());
        let initial = game.fighters[1].life;
        let mut previous = initial;
        let mut hits = 0;
        for tick in 0..180 {
            game.tick(&[0, 0]);
            for fighter in &game.fighters {
                assert!(fighter.pos.x.is_finite() && fighter.pos.y.is_finite());
                assert!(fighter.vel.x.is_finite() && fighter.vel.y.is_finite());
            }
            let victim = &game.fighters[1];
            if victim.life < previous {
                hits += 1;
                println!("distance={distance} tick={tick} damage={} victim_y={} attacker_y={}", previous - victim.life, victim.pos.y, game.fighters[0].pos.y);
            }
            previous = victim.life;
        }
        println!("distance={distance} hits={hits} total_damage={}", initial - previous);
        for (index, fighter) in game.fighters.iter().take(2).enumerate() {
            println!("distance={distance} player={index} final_state={} final_y={} final_vy={}", fighter.state_no, fighter.pos.y, fighter.vel.y);
        }
    }
    Ok(())
}
