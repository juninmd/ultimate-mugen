use std::{hint::black_box, time::Instant};
use ume_formats::vfs::{norm_path, resolve, Vfs};
fn main() {
    let paths = [
        ".\\Chars\\\\KFM//Kfm.DEF",
        "chars/kfm/kfm.def",
        "DATA/COMMON1.CNS",
    ];
    for _ in 0..5 {
        let start = Instant::now();
        for i in 0..1_000_000 {
            black_box(norm_path(black_box(paths[i % paths.len()])));
        }
        println!(
            "normalize: {:.1} ns/op",
            start.elapsed().as_nanos() as f64 / 1_000_000.0
        );
    }
    let mut vfs = Vfs::new();
    for i in 0..50_000 {
        vfs.insert(format!("chars/{i:05}/sprite.sff"), vec![0]);
    }
    vfs.insert("chars/target/nested/target.air".into(), vec![42]);
    for _ in 0..5 {
        let start = Instant::now();
        for _ in 0..1_000 {
            black_box(resolve(black_box(&vfs), "chars/target", "target.air")).unwrap();
        }
        println!(
            "folder fallback (50k files): {:.1} us/op",
            start.elapsed().as_secs_f64() * 1e6 / 1_000.0
        );
    }
}
