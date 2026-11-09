use std::path::Path;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::common::common_def;
use ume_engine::input::InputSource;
use ume_engine::matchs::{CameraParams, Event, Match, StageInfo};
use ume_engine::native::{compile_native, NativeChar};
use ume_engine::stage_bg::BgState;
use ume_formats::stage::{load_stage, MugenStage};
use ume_formats::vfs::{add_dir, add_zip, find_stage_defs, Vfs};

fn stage_to_info(st: &MugenStage) -> StageInfo {
    let scl = 320.0 / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    let scl_y = 240.0 / if st.local_h > 0.0 { st.local_h } else { 240.0 };
    StageInfo {
        name: st.name.clone(),
        bound_left: st.bound_left * scl,
        bound_right: st.bound_right * scl,
        bound_high: st.bound_high * scl_y,
        bound_low: st.bound_low * scl_y,
        p1x: st.p1x * scl,
        p2x: st.p2x * scl,
        zoffset: st.zoffset * scl_y,
        vertical_follow: st.vertical_follow,
        tension: st.tension * scl,
        start_x: st.start_x * scl,
        start_y: st.start_y * scl_y,
        zoom_out: st.zoom_out,
        zoom_in: st.zoom_in,
        camera: CameraParams {
            vertical_follow_delta: st.vertical_follow_delta * scl_y,
            y_scroll_speed: st.y_scroll_speed,
            zoom_in_speed: st.zoom_in_speed,
            zoom_out_speed: st.zoom_out_speed,
            lowest_cap: st.lowest_cap,
            dynamic_bound_high: st.dynamic_bound_high,
            bound_high_delta: st.bound_high_delta * scl_y,
        },
        auto_turn: st.auto_turn,
    }
}

fn load_fighter_json(json: &str) -> Arc<ume_engine::chardef::CharDef> {
    let nc: NativeChar = serde_json::from_str(json).expect("parse native char");
    Arc::new(compile_native(&nc, 1, true))
}

fn run_stage_combat(vfs: &Vfs, def: &str, p1: &Arc<ume_engine::chardef::CharDef>, p2: &Arc<ume_engine::chardef::CharDef>) -> (usize, i32, f32) {
    let st = load_stage(vfs, def).expect("stage load failed");
    assert!(!st.name.is_empty(), "stage name must not be empty");
    let info = stage_to_info(&st);
    let arc_st = Arc::new(st);

    let mut m = Match::new(common_def(), None, info);
    m.add_player(p1.clone(), 0, 0, 1);
    m.add_player(p2.clone(), 1, 0, 2);
    m.bg = Some(BgState::new(arc_st.clone()));
    m.start_round();

    let (mut ai_a, mut ai_b) = (Ai::new(7, 42), Ai::new(7, 99));
    let roots = m.root_indices();
    let mut hits = 0;
    let mut damage = 0.0f32;

    for _ in 0..180 {
        let inputs = [ai_a.poll(&m, roots[0]), ai_b.poll(&m, roots[1])];
        m.tick(&inputs);
        for ev in m.events.drain(..) {
            if let Event::Hit { damage: d, .. } = ev {
                hits += 1;
                damage += d;
            }
        }
        let f1 = &m.fighters[roots[0]];
        let f2 = &m.fighters[roots[1]];
        assert!(f1.pos.x.is_finite() && f1.pos.y.is_finite(), "P1 position must be finite");
        assert!(f2.pos.x.is_finite() && f2.pos.y.is_finite(), "P2 position must be finite");
        assert!(m.camera.x.is_finite() && m.camera.y.is_finite(), "Camera position must be finite");
    }

    (arc_st.layers.len(), hits, damage)
}

#[test]
fn bundled_stages_combat_smoke() {
    let mut vfs = Vfs::new();
    let zip_bytes = include_bytes!("../../../assets/mugen/elecbyte.zip");
    add_zip(&mut vfs, zip_bytes, "").expect("read elecbyte zip");
    let p1 = load_fighter_json(include_str!("../../../assets/chars/akio.json"));
    let p2 = load_fighter_json(include_str!("../../../assets/chars/kaname.json"));

    for def in ["stages/kfm.def", "stages/stage0.def", "stages/stage0-720.def"] {
        let (layers, hits, damage) = run_stage_combat(&vfs, def, &p1, &p2);
        assert!(layers > 0, "Stage {def} must have at least 1 layer");
        println!("Bundled stage {def}: layers={layers}, hits={hits}, damage={damage:.0}");
    }
}

#[test]
fn internet_stages_combat_smoke() {
    let dirs = [
        "/tmp/stage_sources/ctm-stages",
        "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Mountainside Temple",
        "/tmp/stage_sources/Ikemen-Plus-Ultra/stages/Others",
        "/tmp/stage_sources/rebrot-fighter/stages",
    ];

    let p1 = load_fighter_json(include_str!("../../../assets/chars/akio.json"));
    let p2 = load_fighter_json(include_str!("../../../assets/chars/kaname.json"));
    let mut tested = 0;

    for dir in dirs {
        let p = Path::new(dir);
        if !p.exists() { continue; }
        let mut vfs = Vfs::new();
        if add_dir(&mut vfs, p, "stg").is_err() { continue; }
        for def in find_stage_defs(&vfs) {
            let (layers, hits, damage) = run_stage_combat(&vfs, &def, &p1, &p2);
            assert!(layers > 0, "Stage {def} must have layers");
            tested += 1;
            println!("Internet stage #{tested} {def}: layers={layers}, hits={hits}, damage={damage:.0}");
        }
    }

    println!("Total internet stages verified in test: {tested}");
    assert!(tested >= 30, "Expected at least 30 internet stages tested, found {tested}");
}
