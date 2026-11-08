use std::sync::Arc;
use std::time::Instant;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const ROUND_TICKS: usize = 60 * 20;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/chars".into());
    let mut chars: Vec<NativeChar> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let stem = path.file_stem()?.to_string_lossy().to_string();
            (path.extension()? == "json" && stem != "index")
                .then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
        })
        .collect();
    chars.sort_by(|a, b| a.id.cmp(&b.id));
    let roster: Vec<_> = chars
        .iter()
        .map(|nc| Arc::new(compile_native(nc, 1, true)))
        .collect();
    let (mut ticks, mut nanos, mut worst) = (0u64, 0u128, 0u128);
    for i in 0..roster.len() {
        for j in (i + 1)..roster.len() {
            let mut m = Match::new(common_def(), None, flat_stage());
            m.add_player(roster[i].clone(), 0, 0, 1);
            m.add_player(roster[j].clone(), 1, 0, 2);
            m.start_round();
            let (mut a, mut b) = (Ai::new(6, 7), Ai::new(6, 13));
            let roots = m.root_indices();
            for _ in 0..ROUND_TICKS {
                let inputs = [a.poll(&m, roots[0]), b.poll(&m, roots[1])];
                let started = Instant::now();
                m.tick(&inputs);
                let spent = started.elapsed().as_nanos();
                nanos += spent;
                worst = worst.max(spent);
                ticks += 1;
            }
        }
    }
    println!(
        "native_tick_bench {} fighters, {} ticks, {:.0} ticks/s, mean {:.4} ms, worst {:.3} ms",
        roster.len(),
        ticks,
        ticks as f64 / (nanos as f64 / 1e9),
        nanos as f64 / ticks as f64 / 1e6,
        worst as f64 / 1e6
    );
}
