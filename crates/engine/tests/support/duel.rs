use std::path::PathBuf;
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_dir, Vfs};
pub fn load_spec(spec: &str) -> Arc<CharDef> {
    let (kind, rest) = spec
        .split_once(':')
        .expect("spec is mugen:<dir>[:def] or native:<json>");
    match kind {
        "native" => {
            let text = std::fs::read_to_string(rest).unwrap();
            let nc = ume_native::parse_char(&text).unwrap();
            let sprites = std::path::Path::new(rest).with_file_name("sprites");
            if let Ok(dir) = std::fs::read_dir(&sprites) {
                let files: Vec<(String, Vec<u8>)> = dir
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().extension().is_some_and(|x| x == "png")
                            || e.file_name() == "meta.json"
                    })
                    .map(|e| {
                        (
                            e.file_name().to_string_lossy().to_string(),
                            std::fs::read(e.path()).unwrap(),
                        )
                    })
                    .collect();
                let pack = ume_native::encode_sprite_pack(nc.sprite_dir(), &files).unwrap();
                ume_native::insert_sprite_pack(&nc.id, &pack).unwrap();
            }
            Arc::new(ume_native::compile_native(&nc, 1, true))
        }
        _ => {
            let (dir, def) = match rest.split_once(':') {
                Some((d, f)) => (PathBuf::from(d), f.to_string()),
                None => {
                    let d = PathBuf::from(rest);
                    let mut n: Vec<String> = std::fs::read_dir(&d)
                        .unwrap()
                        .filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .filter(|n| n.to_lowercase().ends_with(".def"))
                        .collect();
                    n.sort();
                    (d, n.remove(0))
                }
            };
            let mut v = Vfs::new();
            add_dir(&mut v, &dir, "g").unwrap();
            Arc::new(load_mugen_char(&v, &format!("g/{def}"), 1, true).unwrap())
        }
    }
}

pub fn wilson(score: f64, n: f64) -> [f64; 2] {
    let z = 1.96;
    let d = 1.0 + z * z / n;
    let c = score + z * z / (2.0 * n);
    let h = z * (score * (1.0 - score) / n + z * z / (4.0 * n * n)).sqrt();
    [(c - h) / d, (c + h) / d]
}

pub fn mean_ci(v: &[f64]) -> [f64; 3] {
    let n = v.len() as f64;
    let mean = v.iter().sum::<f64>() / n;
    let sd = (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
    let h = 1.96 * sd / n.sqrt();
    [mean, mean - h, mean + h]
}
