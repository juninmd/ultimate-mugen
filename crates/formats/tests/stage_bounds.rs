use ume_formats::stage::load_stage;
use ume_formats::vfs::Vfs;

fn stage(bounds: &str) -> ume_formats::stage::MugenStage {
    let ini = format!(
        "[Info]\ndisplayname = Bounds\n[Camera]\n{bounds}\n[BGdef]\n[BG 0]\ntype = normal\nspriteno = 0, 0\n"
    );
    let mut vfs = Vfs::new();
    vfs.insert("stages/bounds.def".into(), ini.into_bytes());
    load_stage(&vfs, "stages/bounds.def").unwrap()
}

#[test]
fn reversed_horizontal_bounds_are_normalized() {
    let st = stage("boundleft = 220\nboundright = -220\nboundhigh = -80\nboundlow = 10");
    assert_eq!((st.bound_left, st.bound_right), (-220.0, 220.0));
    assert_eq!((st.bound_high, st.bound_low), (-80.0, 10.0));
}

#[test]
fn reversed_vertical_bounds_are_normalized() {
    let st = stage("boundleft = -220\nboundright = 220\nboundhigh = 10\nboundlow = -80");
    assert_eq!((st.bound_left, st.bound_right), (-220.0, 220.0));
    assert_eq!((st.bound_high, st.bound_low), (-80.0, 10.0));
}

#[test]
fn valid_bounds_are_untouched() {
    let st = stage("boundleft = -300\nboundright = 400\nboundhigh = -120\nboundlow = 5");
    assert_eq!((st.bound_left, st.bound_right), (-300.0, 400.0));
    assert_eq!((st.bound_high, st.bound_low), (-120.0, 5.0));
}
