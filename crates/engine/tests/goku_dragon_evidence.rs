mod support;
use std::sync::Arc;
use support::gogeta::{load, Duel};
use support::render;
use ume_engine::chardef::CharDef;
use ume_engine::cns::parse_states;

const DRAGON: &str = include_str!("../../../tools/mugen2umf/overlays/goku_ssj4_ryuken.cns");
const AFTERIMAGE: &str = include_str!("../../../tools/mugen2umf/overlays/goku_ssj4_afterimage.cns");

fn out_dir() -> Option<std::path::PathBuf> {
    let d = std::path::PathBuf::from(std::env::var("UME_EVIDENCE_DIR").ok()?);
    std::fs::create_dir_all(&d).ok()?;
    Some(d)
}

fn patched() -> Arc<CharDef> {
    let mut d = load(1).unwrap();
    let def = Arc::get_mut(&mut d).unwrap();
    parse_states(DRAGON, &mut def.states, true);
    parse_states(AFTERIMAGE, &mut def.states, true);
    d
}

fn run(d: Arc<CharDef>, raised: f32, behind: bool, label: &str, out: &std::path::Path) -> (f32, f32, bool) {
    let mut r = Duel::new(d, 220.0);
    if behind {
        r.m.fighters[0].facing = -1.0;
    }
    r.m.fighters[1].pos.y = raised;
    let life0 = r.m.fighters[1].life;
    r.m.fighters[0].power = 3000.0;
    let d = r.m.fighters[0].def.clone();
    r.m.change_state(0, 3000, d);
    let mut shots = vec![];
    let mut top = 0.0f32;
    let mut trailed = false;
    let mut csv = String::from("t,dx,dy,fx,fy\n");
    for t in 0..200 {
        r.m.fighters[1].pos.y = raised;
        r.m.fighters[1].vel.y = 0.0;
        r.tick(1);
        trailed |= r.m.fighters[0].after_image.as_ref().is_some_and(|a| a.time != 0);
        for f in r.m.fighters.iter().filter(|f| f.is_helper && f.helper_id == 3050) {
            top = top.min(f.pos.y);
            if t >= 96 { csv += &format!("{t},{},{},{},{}\n", f.pos.x, f.pos.y, r.m.fighters[1].pos.x, r.m.fighters[1].pos.y); }
        }
        if t >= 96 && t % 4 == 0 && shots.len() < 8 {
            shots.push(render::frame(&r.m, false));
        }
    }
    std::fs::write(out.join(format!("{label}.csv")), csv).unwrap();
    render::sheet(&shots, 2).save_png(out.join(format!("{label}.png"))).unwrap();
    (life0 - r.m.fighters[1].life, top, trailed)
}

#[test]
fn dragon_homes_in_on_a_raised_foe() {
    let (Some(base), Some(out)) = (load(1), out_dir()) else { return };
    let (orig_dmg, _, orig_trail) = run(base, -100.0, false, "original-straight", &out);
    let (umf_dmg, top, umf_trail) = run(patched(), -100.0, false, "umf-homing", &out);
    eprintln!("original dmg {orig_dmg}, homing dmg {umf_dmg}, dragon top y {top}");
    assert!(umf_dmg > orig_dmg, "homing connects where the straight dragon misses");
    assert!(umf_trail && !orig_trail, "the skill leaves an afterimage trail only with the overlay");
    let (behind_orig, _, _) = run(load(1).unwrap(), 0.0, true, "original-behind", &out);
    let (behind_umf, _, _) = run(patched(), 0.0, true, "umf-behind", &out);
    eprintln!("behind: original dmg {behind_orig}, homing dmg {behind_umf}");
    assert!(behind_umf > behind_orig, "homing turns around for a foe behind the owner");
}
