//! Rust rig port vs the TypeScript reference (`tests/data/rig_boxes.json`, exported from
//! `src/native/rig.ts` through `compileNative` with a null canvas factory): every frame's
//! collision boxes and every sprite's size/axis must match.
use serde_json::Value;
use ume_engine::native::{compile_native, NativeChar};

const IDS: [&str; 10] = ["kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian", "garan", "marlow"];
const EPS: f64 = 0.01;

fn load(id: &str) -> NativeChar {
    let p = format!("{}/../../assets/chars/{id}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn boxes(v: &Value) -> Vec<[f64; 4]> {
    v.as_array().unwrap().iter().map(|b| { let a = b.as_array().unwrap(); [0, 1, 2, 3].map(|i| a[i].as_f64().unwrap()) }).collect()
}
fn same(a: &[[f32; 4]], b: &[[f64; 4]]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (0..4).all(|i| (x[i] as f64 - y[i]).abs() <= EPS))
}

#[test]
fn rig_boxes_match_typescript_reference() {
    let path = format!("{}/tests/data/rig_boxes.json", env!("CARGO_MANIFEST_DIR"));
    let reference: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut checked = 0;
    let mut errors = vec![];
    for id in IDS {
        let nc = load(id);
        for pal in [1, 2] {
            let r = &reference[id][pal.to_string()];
            let d = compile_native(&nc, pal, false);
            let ranims = r["anims"].as_object().unwrap();
            // the engine adds shared taunt / power-up / defeat animations (195..=200) that the TypeScript reference does not know
            assert_eq!(ranims.len(), d.anims.keys().filter(|k| !(195..=200).contains(*k)).count(), "{id}: anim count");
            for (no, frames) in ranims {
                let a = d.anims.get(&no.parse::<i32>().unwrap()).unwrap_or_else(|| panic!("{id}: anim {no} missing"));
                let frames = frames.as_array().unwrap();
                assert_eq!(frames.len(), a.frames.len(), "{id} anim {no}: frame count");
                for (i, (rf, f)) in frames.iter().zip(&a.frames).enumerate() {
                    checked += 1;
                    if rf["g"].as_i64() != Some(f.group as i64) || rf["i"].as_i64() != Some(f.index as i64) { errors.push(format!("{id}/{pal} anim {no} #{i}: sprite {},{} vs {},{}", f.group, f.index, rf["g"], rf["i"])); }
                    if !same(&f.clsn1, &boxes(&rf["c1"])) { errors.push(format!("{id}/{pal} anim {no} #{i}: clsn1 {:?} vs {}", f.clsn1, rf["c1"])); }
                    if !same(&f.clsn2, &boxes(&rf["c2"])) { errors.push(format!("{id}/{pal} anim {no} #{i}: clsn2 {:?} vs {}", f.clsn2, rf["c2"])); }
                }
            }
            for (k, g) in r["sprites"].as_object().unwrap() {
                let mut it = k.split(',').map(|x| x.parse::<i32>().unwrap());
                let key = (it.next().unwrap(), it.next().unwrap());
                let s = d.sprites.get(&key).unwrap_or_else(|| panic!("{id}: sprite {k} missing"));
                let g: Vec<f64> = g.as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect();
                let got = [s.width as f64, s.height as f64, s.axis_x as f64, s.axis_y as f64, s.scale as f64];
                if (0..5).any(|i| (got[i] - g[i]).abs() > EPS) { errors.push(format!("{id}/{pal} sprite {k}: {got:?} vs {g:?}")); }
            }
        }
    }
    assert!(errors.is_empty(), "{} mismatches (of {checked} frames):\n{}", errors.len(), errors.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
    assert!(checked > 1000);
}

#[test]
fn rig_renders_images() {
    let nc = load("shion");
    let t = std::time::Instant::now();
    let d = compile_native(&nc, 1, true);
    let n = d.sprites.values().filter(|s| s.image.is_some()).count();
    assert_eq!(n, d.sprites.len());
    let idle = &d.sprites[&(1, 0)];
    let img = idle.image.as_ref().unwrap();
    assert_eq!((img.w as f32, img.h as f32), (idle.width, idle.height));
    // something opaque was drawn
    assert!(img.rgba.chunks_exact(4).filter(|p| p[3] == 255).count() > 1000);
    eprintln!("shion with images: {} sprites in {:?}", n, t.elapsed());
}
