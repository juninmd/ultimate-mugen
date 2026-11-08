use ume_formats::stage::{load_stage, stage_requests_3d};
use ume_formats::vfs::Vfs;

#[test]
fn detects_gltf_model_requests() {
    assert!(stage_requests_3d(
        "[StageInfo]\nlocalcoord = 1280, 720\nmodel = arena.glb\n"
    ));
    assert!(stage_requests_3d("[Model]\nfile = arena.gltf\n"));
    assert!(stage_requests_3d("[StageInfo]\nscene = city\n"));
    assert!(!stage_requests_3d(
        "[StageInfo]\nlocalcoord = 320, 240\n[BGDef]\nspr = stage.sff\n"
    ));
}

#[test]
fn a_3d_stage_still_loads_its_2d_layers() {
    let ini = "[Info]\ndisplayname = Arena\n[StageInfo]\nlocalcoord = 1280, 720\nmodel = arena.glb\n[BGDef]\n[BG sky]\ntype = normal\nspriteno = 0, 0\n";
    let mut vfs = Vfs::new();
    vfs.insert("stages/arena.def".into(), ini.as_bytes().to_vec());
    let st = load_stage(&vfs, "stages/arena.def").expect("the 2D fallback still loads");
    assert_eq!(st.name, "Arena");
    assert_eq!(st.layers.len(), 1, "the 2D layer survives the 3D rejection");
}
