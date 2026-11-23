use crate::support;
use std::sync::Arc;
use support::gogeta::{load, Duel};
use support::render;
use ume_engine::chardef::CharDef;
use ume_engine::cns::parse_states;

const AFTERIMAGE: &str = include_str!("../../../tools/mugen2umf/overlays/goku_ssj4_vision_afterimage.cns");
const CHARGE: &str = include_str!("../../../tools/mugen2umf/overlays/goku_ssj4_vision_charge.cns");

fn out_dir() -> Option<std::path::PathBuf> {
    let d = std::path::PathBuf::from(std::env::var("UME_EVIDENCE_DIR").ok()?);
    std::fs::create_dir_all(&d).ok()?;
    Some(d)
}

fn patched() -> Arc<CharDef> {
    let mut d = load(1).unwrap();
    let def = Arc::get_mut(&mut d).unwrap();
    parse_states(AFTERIMAGE, &mut def.states, true);
    parse_states(CHARGE, &mut def.states, true);
    d
}

fn trail(d: Arc<CharDef>, state: i32, label: &str, out: &std::path::Path) -> usize {
    let mut r = Duel::new(d, 260.0);
    r.m.fighters[0].power = 3000.0;
    let def = r.m.fighters[0].def.clone();
    r.m.change_state(0, state, def);
    let mut shots = vec![];
    let mut ghosts = 0;
    for t in 0..140 {
        r.tick(1);
        let n = r.m.fighters[0].after_image.as_ref().map_or(0, |a| a.trail.len());
        ghosts = ghosts.max(n);
        if n > 3 && t % 3 == 0 && shots.len() < 6 {
            shots.push(render::frame(&r.m, false));
        }
    }
    if !shots.is_empty() {
        render::sheet(&shots, 3).save_png(out.join(format!("{label}.png"))).unwrap();
    }
    ghosts
}

#[test]
fn skills_leave_afterimage_trails() {
    let (Some(_), Some(out)) = (load(1), out_dir()) else { return };
    for (state, label) in [(7500, "afterimage-shunkan-ido"), (1700, "afterimage-kamehameha"), (3030, "afterimage-dragon-fist"), (1390, "afterimage-fireballs")] {
        let ghosts = trail(patched(), state, label, &out);
        eprintln!("{label}: trail {ghosts}");
        assert!(ghosts > 3, "{label} draws a trail");
    }
}

#[test]
fn charge_state_refills_ki_and_hands_control_back() {
    let (Some(_), Some(out)) = (load(1), out_dir()) else { return };
    let mut r = Duel::new(patched(), 260.0);
    r.m.fighters[0].power = 0.0;
    let def = r.m.fighters[0].def.clone();
    r.m.change_state(0, 90200, def);
    let mut shots = vec![];
    for t in 0..12 {
        r.tick(1);
        if t % 2 == 0 {
            shots.push(render::frame(&r.m, false));
        }
    }
    render::sheet(&shots, 3).save_png(out.join("carga-ki.png")).unwrap();
    let f = &r.m.fighters[0];
    eprintln!("power {} state {} ctrl {}", f.power, f.state_no, f.ctrl);
    assert!(f.power >= 72.0, "each tick of the charge adds ki");
    assert!(f.ctrl && f.state_no != 90200, "releasing the key returns control");
}
