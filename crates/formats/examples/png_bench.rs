use std::path::{Path, PathBuf};
use std::time::Instant;

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "png") {
            out.push(path);
        }
    }
}

fn main() {
    let dir = std::env::args().nth(1).expect("usage: png_bench <dir>");
    let rounds: usize = std::env::args().nth(2).and_then(|r| r.parse().ok()).unwrap_or(5);
    let mut paths = Vec::new();
    collect(Path::new(&dir), &mut paths);
    paths.sort();
    let files: Vec<Vec<u8>> = paths.iter().filter_map(|p| std::fs::read(p).ok()).collect();
    let bytes: usize = files.iter().map(Vec::len).sum();
    let mut samples = Vec::new();
    let mut checksum = 0u64;
    for _ in 0..rounds {
        let start = Instant::now();
        for file in &files {
            if let Ok(png) = ume_formats::png::decode_png(file) {
                let pixels = png.rgba.as_deref().or(png.indexed.as_deref()).unwrap_or(&[]);
                checksum = checksum.wrapping_mul(31).wrapping_add(pixels.iter().step_by(97).map(|&b| b as u64).sum::<u64>());
            }
        }
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(|a, b| a.total_cmp(b));
    println!("{} files {:.1} MB  min {:.1} ms  median {:.1} ms  max {:.1} ms  checksum {checksum:x}", files.len(), bytes as f64 / 1e6, samples[0], samples[samples.len() / 2], samples[samples.len() - 1]);
}
}
