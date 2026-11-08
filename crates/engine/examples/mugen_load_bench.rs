use std::time::Instant;
use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_zip, find_char_defs, Vfs};

fn main() {
    let zip = std::env::args().nth(1).unwrap_or_else(|| format!("{}/../../assets/mugen/elecbyte.zip", env!("CARGO_MANIFEST_DIR")));
    let runs: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(5);
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &std::fs::read(&zip).unwrap(), "").unwrap();
    for def in find_char_defs(&vfs) {
        let mut best = f64::MAX;
        let mut sprites = 0;
        for _ in 0..runs {
            let t = Instant::now();
            let d = load_mugen_char(&vfs, &def, 1, std::env::var("NOIMG").is_err()).unwrap();
            best = best.min(t.elapsed().as_secs_f64() * 1000.0);
            sprites = d.sprites.len();
        }
        let sff = def.replace(".def", ".sff");
        let t = Instant::now();
        if let Some(b) = vfs.get(&sff) { let _ = ume_formats::sff::decode_sff(&b); }
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let _ = ume_engine::loader::load_mugen_portrait(&vfs, &def);
        let portrait = t.elapsed().as_secs_f64() * 1000.0;
        println!("{def:40} {best:7.1} ms ({sprites} sprites, sff {ms:.1} ms, portrait {portrait:.1} ms)");
    }
}
