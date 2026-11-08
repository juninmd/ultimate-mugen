use std::sync::Arc;
use std::time::Instant;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_dir, add_zip, find_char_defs, Vfs};

fn percentile(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p) as usize]
}

fn main() {
    let src = std::env::args().nth(1).unwrap_or_else(|| {
        format!(
            "{}/../../assets/mugen/elecbyte.zip",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    let seconds: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let per_side: usize = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mut vfs = Vfs::new();
    let path = std::path::Path::new(&src);
    if path.is_dir() {
        add_dir(&mut vfs, path, "c").unwrap();
    } else {
        add_zip(&mut vfs, &std::fs::read(path).unwrap(), "").unwrap();
    }
    for def in find_char_defs(&vfs) {
        let Ok(d) = load_mugen_char(&vfs, &def, 1, true) else {
            continue;
        };
        let d = Arc::new(d);
        let mut m = Match::new(common_def(), None, flat_stage());
        let mut ais = Vec::new();
        for side in 0..2 {
            for n in 0..per_side {
                m.add_player(d.clone(), side, 6, 1 + n as i32);
                ais.push(Ai::new(6, 7 + 6 * (side * per_side + n) as u32));
            }
        }
        m.start_round();
        let mut lat = Vec::with_capacity(seconds * 60);
        let mut peak_fighters = 0;
        let total = Instant::now();
        for _ in 0..seconds * 60 {
            let t = Instant::now();
            let roots = m.root_indices();
            let inputs: Vec<u32> = roots
                .iter()
                .zip(ais.iter_mut())
                .map(|(&r, a)| a.poll(&m, r))
                .collect();
            m.tick(&inputs);
            m.events.clear();
            peak_fighters = peak_fighters.max(m.fighters.len());
            lat.push(t.elapsed().as_secs_f64() * 1000.0);
            if m.match_over {
                break;
            }
        }
        let wall = total.elapsed().as_secs_f64();
        lat.sort_by(|a, b| a.total_cmp(b));
        println!(
            "{def:40} {:8.0} ticks/s  p50 {:6.3} ms  p99 {:6.3} ms  max {:6.3} ms  budget(16.6ms) {:.1}%  peak_fighters {}",
            lat.len() as f64 / wall, percentile(&lat, 0.5), percentile(&lat, 0.99), lat[lat.len() - 1],
            100.0 * percentile(&lat, 0.99) / 16.667, peak_fighters,
        );
    }
}
