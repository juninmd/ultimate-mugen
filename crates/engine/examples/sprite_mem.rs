//! Reports how much sprite data a MUGUEN character needs in the render atlases, which is what the
//! fight warm-up has to copy. Used to size `App::WARM_BUDGET`.
//! Usage: cargo run --release -p ume-engine --example sprite_mem -- <char_dir>...
use std::sync::Arc;
use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn main() {
    for dir in std::env::args().skip(1) {
        let mut vfs = Vfs::new();
        if let Err(e) = add_dir(&mut vfs, std::path::Path::new(&dir), "c") {
            println!("{dir}: io error {e}");
            continue;
        }
        for def in find_char_defs(&vfs) {
            let Ok(d) = load_mugen_char(&vfs, &def, 1, true) else {
                println!("FAIL load {def}");
                continue;
            };
            let d = Arc::new(d);
            let indexed: u64 = d.indexed.values().map(|s| s.w as u64 * s.h as u64).sum();
            let rgba: u64 = d
                .sprites
                .values()
                .filter_map(|s| s.image.as_ref())
                .map(|i| i.w as u64 * i.h as u64 * 4)
                .sum();
            let largest = d
                .sprites
                .values()
                .filter_map(|s| s.image.as_ref())
                .map(|i| i.w as u64 * i.h as u64)
                .max()
                .unwrap_or(0);
            let biggest = d
                .indexed
                .values()
                .map(|s| s.w as u64 * s.h as u64)
                .max()
                .unwrap_or(0);
            println!(
                "{:<26} sprites={:<5} indexed={:<5} rgba={:<5} | index atlas {:>7.2} MB | rgba atlas {:>7.2} MB | total {:>7.2} MB | largest spr {:>6.0} KB",
                d.id,
                d.sprites.len(),
                d.indexed.len(),
                d.sprites.values().filter(|s| s.image.is_some()).count(),
                mb(indexed),
                mb(rgba),
                mb(indexed + rgba),
                (largest.max(biggest) as f64 * 4.0) / 1024.0
            );
        }
    }
}
