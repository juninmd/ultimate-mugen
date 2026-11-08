use std::collections::BTreeMap;
use ume_engine::native::{compile_native, NativeChar};

const STANDING_UNITS: f32 = 96.0;
const MIN_HEIGHT: f32 = 0.355;
const MIN_FORWARD_REACH: f32 = 0.595;
const MAX_START: f32 = 0.105;
const HIGHEST_TOP: f32 = -0.951;
const MAX_NORMAL_REACH: f32 = 1.6;
const PROJECTILE_WIDTH: f32 = 180.0;

fn characters() -> Vec<NativeChar> {
    let dir = std::env::var("UME_CHARS_DIR").unwrap_or_else(|_| "../../assets/chars".into());
    let mut out: Vec<NativeChar> = std::fs::read_dir(dir).unwrap().filter_map(|e| {
        let path = e.unwrap().path();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        (path.extension()? == "json" && stem != "index").then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
    }).collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

#[test]
fn every_attack_box_follows_mugen_proportions() {
    let mut problems: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total = 0;
    for nc in characters() {
        let height = STANDING_UNITS * nc.stats.height.unwrap_or(100.0) as f32 / 100.0;
        let def = compile_native(&nc, 1, true);
        for mv in &nc.moves {
            let Some(anim) = nc.anims.get(&mv.anim) else { continue };
            let label = format!("State -1, {}", mv.name);
            let Some(controller) = def.states.get(&-1).unwrap().controllers.iter().find(|c| c.label == label) else { continue };
            let state: i32 = controller.raw("value").unwrap().trim().parse().unwrap();
            let Some(body) = def.states.get(&state) else { continue };
            let Some(no) = body.param("anim").and_then(|p| p.raw.trim().parse::<i32>().ok()) else { continue };
            let Some(compiled) = def.anims.get(&no) else { continue };
            for (i, frame) in compiled.frames.iter().enumerate() {
                if !anim.frames.get(i).is_some_and(|f| f.hit.is_some()) { continue; }
                total += 1;
                let tag = format!("{}/{}[{}]", nc.id, mv.anim, i);
                if frame.clsn1.is_empty() { problems.entry(nc.id.clone()).or_default().push(format!("{tag}: strike frame without hit box")); continue; }
                for b in frame.clsn1.iter().filter(|b| b[2] - b[0] <= PROJECTILE_WIDTH) {
                    let (w, h) = (b[2] - b[0], b[3] - b[1]);
                    let forward = b[0] + b[2] >= 0.0;
                    let mut bad = vec![];
                    if h < MIN_HEIGHT * height { bad.push(format!("height {h:.0} under {:.0}", MIN_HEIGHT * height)); }
                    if forward && b[2] < MIN_FORWARD_REACH * height { bad.push(format!("reaches only x={:.0}", b[2])); }
                    if forward && b[0] > MAX_START * height { bad.push(format!("starts away from the body at x={:.0}", b[0])); }
                    if forward && w < 0.395 * height { bad.push(format!("width {w:.0} under {:.0}", 0.395 * height)); }
                    if b[1] < HIGHEST_TOP * height { bad.push(format!("above the head, top y={:.0}", b[1])); }
                    if b[3] > 0.5 { bad.push(format!("under the floor, bottom y={:.0}", b[3])); }
                    if b[2] > MAX_NORMAL_REACH * height && !matches!(mv.kind.as_str(), "special" | "super") { bad.push(format!("normal reaches x={:.0}", b[2])); }
                    if !bad.is_empty() { problems.entry(nc.id.clone()).or_default().push(format!("{tag} {:?}: {}", b, bad.join(", "))); }
                }
            }
        }
    }
    for (id, list) in &problems { for line in list { println!("{id}: {line}"); } }
    println!("strike frames audited: {total}, problem frames: {}", problems.values().map(Vec::len).sum::<usize>());
    assert!(problems.is_empty(), "hit box problems found");
}
