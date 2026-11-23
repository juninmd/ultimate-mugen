//! Times native-character compilation (`cargo run --release -p ume-engine --example load_bench`).
use std::time::Instant;
use ume_native::{compile_native, NativeChar};
fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| format!("{}/../../assets/chars", env!("CARGO_MANIFEST_DIR")));
    let checksum = std::env::var_os("UME_LOAD_CHECKSUM").is_some();
    let mut tot = 0.0;
    for e in std::fs::read_dir(&dir).unwrap().flatten() {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "json") || p.file_stem().unwrap() == "index" { continue; }
        let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let t = Instant::now();
        let d = compile_native(&nc, 1, true);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let pv = ume_native::compile_native_preview(&nc, 1);
        let pms = t.elapsed().as_secs_f64() * 1000.0;
        tot += ms;
        println!("{:10} full {:6.0} ms ({} sprites) · preview {:5.0} ms ({} sprites) · pixels {:016x} · resident {:.1} MB (variants {:.1} MB)", nc.id, ms, d.sprites.len(), pms, pv.sprites.len(), if checksum { pixel_checksum(&d) } else { 0 }, resident_mb(&d, |_| true), resident_mb(&d, |g| g >= 2 && g < 9000));
    }
    println!("total {tot:.0} ms");
}

fn pixel_checksum(def: &ume_engine::chardef::CharDef) -> u64 {
    let mut keys: Vec<_> = def.sprites.keys().copied().collect();
    keys.sort_unstable();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut mix = |byte: u8| hash = (hash ^ byte as u64).wrapping_mul(0x100_0000_01b3);
    for key in keys {
        let sprite = &def.sprites[&key];
        key.0.to_le_bytes().into_iter().chain(key.1.to_le_bytes()).for_each(&mut mix);
        if let Some(image) = &sprite.image {
            image.w.to_le_bytes().into_iter().chain(image.h.to_le_bytes()).for_each(&mut mix);
            image.rgba().iter().copied().for_each(&mut mix);
        }
        [sprite.axis_x, sprite.axis_y, sprite.width, sprite.height, sprite.scale].iter().flat_map(|v| v.to_bits().to_le_bytes()).for_each(&mut mix);
    }
    hash
}

fn resident_mb(def: &ume_engine::chardef::CharDef, keep: impl Fn(i32) -> bool) -> f64 {
    let mut seen = std::collections::HashSet::new();
    let bytes: usize = def.sprites.iter().filter(|(key, _)| keep(key.0)).filter_map(|(_, sprite)| sprite.image.as_ref()).filter(|image| !image.is_pending()).filter(|image| seen.insert(std::sync::Arc::as_ptr(image) as usize)).map(|image| image.rgba().len()).sum();
    bytes as f64 / 1_048_576.0
}
