use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const ROUND_TICKS: usize = 60 * 99;
const SEEDS: [(u32, u32); 3] = [(7, 13), (21, 5), (99, 3)];

fn load(dir: &str) -> Vec<(String, Arc<CharDef>)> {
    let mut chars: Vec<NativeChar> = std::fs::read_dir(dir).unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        (path.extension()? == "json" && stem != "index").then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
    }).collect();
    chars.sort_by(|a, b| a.id.cmp(&b.id));
    chars.iter().map(|nc| (nc.id.clone(), Arc::new(compile_native(nc, 1, true)))).collect()
}

fn mirror(def: &Arc<CharDef>, levels: [i32; 2], seeds: (u32, u32)) -> Option<usize> {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    let (mut a, mut b) = (Ai::new(levels[0], seeds.0), Ai::new(levels[1], seeds.1));
    let roots = m.root_indices();
    let mut fighting = 0;
    while fighting < ROUND_TICKS && m.round_state <= 2 {
        let inputs = [a.poll(&m, roots[0]), b.poll(&m, roots[1])];
        m.tick(&inputs);
        if m.round_state == 2 { fighting += 1; }
    }
    let lives = [m.fighters[roots[0]].life, m.fighters[roots[1]].life];
    if lives[0] > lives[1] { Some(0) } else if lives[1] > lives[0] { Some(1) } else { None }
}

fn main() {
    let roster = load(&std::env::args().nth(1).unwrap_or_else(|| "assets/chars".into()));
    let pairs: [(i32, i32); 3] = [(8, 3), (6, 3), (8, 6)];
    for (high, low) in pairs {
        let (mut wins, mut games) = (0.0f32, 0.0f32);
        for (_, def) in &roster {
            for seeds in SEEDS {
                for high_side in 0..2 {
                    let levels = if high_side == 0 { [high, low] } else { [low, high] };
                    games += 1.0;
                    match mirror(def, levels, seeds) {
                        Some(side) if side == high_side => wins += 1.0,
                        None => wins += 0.5,
                        _ => {}
                    }
                }
            }
        }
        println!("level {high} vs level {low}: higher level wins {:.1}% of {games} mirror rounds", wins / games * 100.0);
    }
}
