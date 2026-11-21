//! Times PNG decode over a sprite directory (`cargo run --release -p ume-formats --example png_bench -- assets/chars/sprites/marlow`).
use std::time::Instant;
use ume_formats::png::decode_png;

fn main() {
    let dir = std::env::args().nth(1).expect("usage: png_bench <dir>");
    let files: Vec<Vec<u8>> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .map(|e| std::fs::read(e.path()).unwrap())
        .collect();
    let mut runs: Vec<f64> = (0..7)
        .map(|_| {
            let t = Instant::now();
            for f in &files {
                std::hint::black_box(decode_png(f).unwrap());
            }
            t.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{} pngs: min {:.1} ms  median {:.1} ms  max {:.1} ms", files.len(), runs[0], runs[3], runs[6]);
}
