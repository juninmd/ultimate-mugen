use serde_json::Value;
use std::time::Instant;
use ume_engine::native::rig::{render_effect, render_pose};
fn main() {
    let id = std::env::args().nth(1).unwrap_or("shion".into());
    let ch: Value = serde_json::from_str(&std::fs::read_to_string(format!("{}/../../assets/chars/{id}.json", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap();
    let style = &ch["art"]["style"];
    let mut v: Vec<(f64, String)> = vec![];
    let mut seen = std::collections::HashSet::new();
    let mut tot = 0.0;
    for (n, a) in ch["anims"].as_object().unwrap() {
        for (i, f) in a["frames"].as_array().unwrap().iter().enumerate() {
            let key = serde_json::to_string(&(&f["pose"], &f["effect"], &f["hit"])).unwrap();
            if !seen.insert(key) { continue; }
            let t = Instant::now();
            if !f["effect"].is_null() { render_effect(&f["effect"], true, None); } else {
                let pose = match &f["pose"] { Value::String(s) => ch["art"]["poses"][s].clone(), v => v.clone() };
                let hit = serde_json::from_value(f["hit"].clone()).ok();
                render_pose(style, &pose, hit.as_ref(), true, None);
            }
            let dt = t.elapsed().as_secs_f64() * 1000.0;
            tot += dt;
            v.push((dt, format!("{n}#{i} fx={}", !f["effect"].is_null())));
        }
    }
    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("total {tot:.1}ms n={}", v.len());
    for x in v.iter().take(12) { println!("{:.2} {}", x.0, x.1); }
    println!("median {:.2}", v[v.len() / 2].0);
}
