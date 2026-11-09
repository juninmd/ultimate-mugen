mod support;
use std::collections::HashMap;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const STAND_NORMAL_ANIMS: [&str; 4] = ["jab", "strong", "lkick", "hkick"];
const MAX_GAP: i32 = 150;

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

fn state_of(nc: &NativeChar, def: &ume_engine::chardef::CharDef, anim: &str) -> i32 {
    let name = &nc.moves.iter().find(|m| m.anim == anim).unwrap_or_else(|| panic!("{}: no {anim} move", nc.id)).name;
    let label = format!("State -1, {name}");
    let minus_one = def.states.get(&-1).unwrap();
    let controller = minus_one.controllers.iter().find(|c| c.label == label).unwrap();
    controller.raw("value").unwrap().trim().parse().unwrap()
}

fn reach(base: &Match, state: i32) -> (i32, i32) {
    let mut farthest = 0;
    let mut total = 0.0;
    for gap in (10..=MAX_GAP).step_by(2) {
        let mut m = base.clone();
        face_off(&mut m, gap as f32);
        let before = m.fighters[1].life;
        enter(&mut m, 0, state);
        ticks(&mut m, 60);
        let dealt = before - m.fighters[1].life;
        if dealt > 0.0 {
            farthest = gap;
            total = f32::max(total, dealt);
        }
    }
    (farthest, total as i32)
}

#[test]
fn every_fighter_has_its_own_normal_reach_table() {
    let mut tables: HashMap<Vec<i32>, String> = HashMap::new();
    for nc in characters() {
        let def = Arc::new(compile_native(&nc, 1, true));
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(def.clone(), 0, 0, 1);
        m.add_player(def.clone(), 1, 0, 2);
        m.start_round();
        while m.round_state != 2 { m.tick(&[0, 0]); }
        let row: Vec<(i32, i32)> = STAND_NORMAL_ANIMS.iter().map(|a| reach(&m, state_of(&nc, &def, a))).collect();
        println!("{:8} reach/damage jab {:?} strong {:?} lkick {:?} hkick {:?}", nc.id, row[0], row[1], row[2], row[3]);
        assert!(row.iter().all(|(r, d)| *r > 0 && *d > 0), "{}: a stand normal never connects", nc.id);
        let key: Vec<i32> = row.iter().flat_map(|(r, d)| [*r, *d]).collect();
        if let Some(other) = tables.insert(key, nc.id.clone()) {
            panic!("{} and {} share the same normal reach and damage table", nc.id, other);
        }
    }
}
