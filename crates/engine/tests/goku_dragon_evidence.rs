use crate::support;
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

fn team_scene(d: Arc<CharDef>, label: &str, out: &std::path::Path) -> Vec<f32> {
    use ume_engine::common::{common_def, flat_stage};
    use ume_engine::matchs::Match;
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d, 0, 0, 1);
    for _ in 0..3 {
        m.add_player(load(1).unwrap(), 1, 0, 2);
    }
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0; 4]);
    }
    let c = m.camera.x;
    let spots = [(c + 80.0, -110.0), (c + 150.0, 0.0), (c + 230.0, -40.0)];
    m.fighters[0].pos.x = c - 120.0;
    m.fighters[0].power = 3000.0;
    for (k, (x, y)) in spots.iter().enumerate() {
        let f = &mut m.fighters[k + 1];
        f.pos.x = *x;
        f.life = 100000.0;
        f.life_max = 100000.0;
        f.facing = -1.0;
        f.pos.y = *y;
    }
    let d = m.fighters[0].def.clone();
    m.change_state(0, 3000, d);
    let mut shots = vec![];
    let mut csv = format!("t,dx,dy,{c}\n");
    for t in 0..170 {
        for (k, (x, y)) in spots.iter().enumerate() {
            let f = &mut m.fighters[k + 1];
            f.pos.y = *y;
            f.vel.y = 0.0;
            if f.life >= 100000.0 {
                f.pos.x = *x;
            }
        }
        m.tick(&[0; 4]);
        if let Some(h) = m.fighters.iter().find(|f| f.is_helper && f.helper_id == 3050) {
            csv += &format!("{t},{},{}\n", h.pos.x, h.pos.y);
        }
        if t >= 98 && (t - 98) % 4 == 0 && shots.len() < 12 {
            shots.push(render::frame(&m, false));
        }
    }
    std::fs::write(out.join(format!("{label}.csv")), csv).unwrap();
    render::sheet(&shots, 3).save_png(out.join(format!("{label}.png"))).unwrap();
    (1..4).map(|k| 100000.0 - m.fighters[k].life).collect()
}

#[test]
fn dragon_chooses_among_three_foes() {
    let (Some(base), Some(out)) = (load(1), out_dir()) else { return };
    let straight = team_scene(base, "dragao-original-3-inimigos", &out);
    let homing = team_scene(patched(), "dragao-umf-3-inimigos", &out);
    eprintln!("damage per foe: original {straight:?}, homing {homing:?}");
    assert!(homing[0] > 0.0 && straight[0] == 0.0, "homing reaches the nearest foe, the straight dragon misses it");
}

fn afterimage_sheet(d: Arc<CharDef>, state: i32, drift: f32, label: &str, out: &std::path::Path) -> usize {
    let mut r = Duel::new(d, 260.0);
    r.m.fighters[0].power = 3000.0;
    let def = r.m.fighters[0].def.clone();
    r.m.change_state(0, state, def);
    let mut shots = vec![];
    let mut ghosts = 0;
    for t in 0..140 {
        r.tick(1);
        r.m.fighters[0].pos.x += drift;
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
fn afterimage_trails_are_drawn() {
    let (Some(base), Some(out)) = (load(1), out_dir()) else { return };
    for (state, drift, label) in [(1300, 3.0, "afterimage-shunkan-ido-drift"), (1300, 0.0, "afterimage-shunkan-ido"), (1800, 0.0, "afterimage-super-kamehameha")] {
        let none = afterimage_sheet(load(1).unwrap(), state, drift, &format!("{label}-original"), &out);
        let ghosts = afterimage_sheet(patched(), state, drift, label, &out);
        eprintln!("{label}: original trail {none}, overlay trail {ghosts}");
        assert!(ghosts > 3 && none == 0);
    }
    drop(base);
}
