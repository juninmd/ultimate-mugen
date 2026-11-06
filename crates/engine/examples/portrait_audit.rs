//! Audits M.U.G.E.N. select-screen portraits: which sprite a pack would use, whether it is a
//! solid dark placeholder, and how long decoding it takes.
//! Usage: cargo run --release -p ume-engine --example portrait_audit -- <char_dir>...
use std::time::Instant;
use ume_engine::loader::load_mugen_portrait;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn stats(rgba: &[u8]) -> (usize, f64, u8) {
    let (mut visible, mut sum, mut max) = (0usize, 0u64, 0u8);
    for p in rgba.chunks_exact(4).filter(|p| p[3] > 16) {
        let l = ((p[0] as u16 + p[1] as u16 + p[2] as u16) / 3) as u8;
        visible += 1;
        sum += l as u64;
        max = max.max(l);
    }
    (
        visible,
        if visible == 0 {
            0.0
        } else {
            sum as f64 / visible as f64
        },
        max,
    )
}

fn main() {
    let repeat: usize = std::env::args()
        .position(|a| a == "--repeat")
        .and_then(|i| std::env::args().nth(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    for dir in std::env::args().skip(1).filter(|a| !a.starts_with("--")) {
        let mut vfs = Vfs::new();
        if add_dir(&mut vfs, std::path::Path::new(&dir), "c").is_err() {
            continue;
        }
        for def in find_char_defs(&vfs) {
            let t = Instant::now();
            let mut portrait = None;
            for _ in 0..repeat {
                portrait = load_mugen_portrait(&vfs, &def);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / repeat as f64;
            match portrait {
                Some(p) => {
                    let (visible, avg, max) = p
                        .image
                        .as_ref()
                        .map(|i| stats(&i.rgba))
                        .unwrap_or((0, 0.0, 0));
                    println!(
                        "{def:<32} {:>5.0}x{:<5.0} visible={visible:<6} lum avg={avg:>6.1} max={max:<3} decode={ms:5.2} ms",
                        p.width, p.height
                    );
                }
                None => println!("{def:<32} no usable portrait (decode={ms:5.2} ms)"),
            }
        }
    }
}
