mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const CROUCH_HOLD: u32 = k::D;
const PRESS_TICKS: usize = 3;
const WATCH_TICKS: usize = 50;
const GAPS: [f32; 3] = [20.0, 60.0, 110.0];

fn roster() -> Vec<NativeChar> {
    let mut out: Vec<NativeChar> = std::fs::read_dir("../../assets/chars").unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        (path.extension()? == "json" && stem != "index").then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
    }).collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    m
}

fn damage_at(def: &Arc<CharDef>, gap: f32, held: u32, button: u32) -> f32 {
    let mut m = arena(def);
    face_off(&mut m, gap);
    for _ in 0..8 { m.tick(&[held, 0]); }
    let before = m.fighters[1].life;
    for _ in 0..PRESS_TICKS { m.tick(&[held | button, 0]); }
    for _ in 0..WATCH_TICKS { m.tick(&[held, 0]); }
    before - m.fighters[1].life
}

fn damage_dealt(def: &Arc<CharDef>, held: u32, button: u32) -> f32 {
    GAPS.iter().map(|gap| damage_at(def, *gap, held, button)).fold(0.0, f32::max)
}

#[test]
fn z_and_c_land_hits_standing_and_crouching_for_every_native_fighter() {
    for nc in roster() {
        let def = Arc::new(compile_native(&nc, 1, false));
        for (held, stance) in [(0, "standing"), (CROUCH_HOLD, "crouching")] {
            for (button, name) in [(k::Z, "Soco 3"), (k::C, "Chute 3")] {
                assert!(damage_dealt(&def, held, button) > 0.0, "{} {stance} {name} did nothing", nc.id);
            }
        }
    }
}

#[test]
fn power_normals_are_listed_in_the_move_list() {
    for nc in roster() {
        let def = compile_native(&nc, 1, false);
        for input in ["z", "c"] {
            assert!(def.movelist.iter().any(|m| m.input == input), "{} has no `{input}` entry in the move list", nc.id);
        }
    }
}
