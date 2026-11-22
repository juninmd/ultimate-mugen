//! Sums the RGBA bytes every native fighter would push into the atlas (`cargo run --release -p ume-engine --example atlas_bytes`).
use std::collections::HashSet;
use ume_native::{compile_native, NativeChar};
fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| format!("{}/../../assets/chars", env!("CARGO_MANIFEST_DIR")));
    let (mut total, mut count) = (0u64, 0u64);
    for e in std::fs::read_dir(&dir).unwrap().flatten() {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "json") || p.file_stem().unwrap() == "index" { continue; }
        let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let def = compile_native(&nc, 1, true);
        let mut seen = HashSet::new();
        let mut dup = HashSet::new();
        let mut dup_bytes = 0u64;
        let (mut bytes, mut n) = (0u64, 0u64);
        for s in def.sprites.values() {
            let Some(im) = s.image.as_ref() else { continue };
            if seen.insert(im.id) { if !dup.insert((im.w, im.h, im.rgba.iter().fold(0u64, |h, b| h.wrapping_mul(1099511628211).wrapping_add(*b as u64 + 1)))) { dup_bytes += im.w as u64 * im.h as u64 * 4; } bytes += im.w as u64 * im.h as u64 * 4; n += 1; }
        }
        println!("{:10} {:5} images {:8.1} MB  avg {:6.1} KB  duplicate {:6.1} MB", nc.id, n, bytes as f64 / 1e6, bytes as f64 / n.max(1) as f64 / 1e3, dup_bytes as f64 / 1e6);
        total += bytes; count += n;
    }
    println!("total {count} images {:.1} MB avg {:.1} KB", total as f64 / 1e6, total as f64 / count.max(1) as f64 / 1e3);
}
