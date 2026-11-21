//! Malformed stage .def files must load into values the camera can clamp with: M.U.G.E.N. packs in the wild ship
//! inverted camera bounds, and `f32::clamp(min > max)` panics inside the tick.
use ume_formats::stage::{load_stage, MugenStage};
use ume_formats::vfs::Vfs;

fn stage_from(def: &str) -> MugenStage {
    let mut vfs = Vfs::new();
    vfs.insert("stages/broken.def".into(), def.as_bytes().to_vec());
    load_stage(&vfs, "stages/broken.def").expect("stage loads")
}

#[test]
fn inverted_camera_bounds_are_sorted_instead_of_panicking_the_camera() {
    let st = stage_from("[Camera]\nboundleft = 10\nboundright = -5\nboundhigh = 0\nboundlow = -9\n[BGdef]\nspr = none.sff\n[BG sky]\ntype = normal\nspriteno = 0, 0\n");
    assert_eq!((st.bound_left, st.bound_right), (-5.0, 10.0));
    assert_eq!((st.bound_high, st.bound_low), (-9.0, 0.0));
    assert!(st.bound_left <= st.bound_right && st.bound_high <= st.bound_low);
}

#[test]
fn well_ordered_camera_bounds_are_kept() {
    let st = stage_from("[Camera]\nboundleft = -150\nboundright = 150\nboundhigh = -25\nboundlow = 0\n[BGdef]\nspr = none.sff\n[BG sky]\ntype = normal\nspriteno = 0, 0\n");
    assert_eq!((st.bound_left, st.bound_right), (-150.0, 150.0));
    assert_eq!((st.bound_high, st.bound_low), (-25.0, 0.0));
}
