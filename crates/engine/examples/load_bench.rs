//! Times native-character compilation (`cargo run --release -p ume-engine --example load_bench`).
use std::time::Instant;
use ume_engine::native::{compile_native, NativeChar};
fn main() {
    let dir = format!("{}/../../assets/chars", env!("CARGO_MANIFEST_DIR"));
    let mut tot = 0.0;
    for e in std::fs::read_dir(&dir).unwrap().flatten() {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "json") || p.file_stem().unwrap() == "index" { continue; }
        let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let t = Instant::now();
        let d = compile_native(&nc, 1, true);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let pv = ume_engine::native::compile_native_preview(&nc, 1);
        let pms = t.elapsed().as_secs_f64() * 1000.0;
        tot += ms;
        println!("{:10} full {:6.0} ms ({} sprites) · preview {:5.0} ms ({} sprites)", nc.id, ms, d.sprites.len(), pms, pv.sprites.len());
    }
    println!("total {tot:.0} ms");
}
