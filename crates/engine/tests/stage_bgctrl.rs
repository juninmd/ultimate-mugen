use std::sync::Arc;
use ume_engine::stage_bg::BgState;
use ume_formats::stage::{load_stage, BgCtrlKind};
use ume_formats::vfs::Vfs;

const DEF: &str = r#"
[Info]
name = "Ctrl Test"

[Camera]
startx = 30
starty = -10
boundhigh = -40
zoomout = 0.8

[StageInfo]
zoffset = 210

[Shadow]
intensity = 96
color = 10, 20, 30
yscale = -0.3

[Reflection]
intensity = 64

[Music]
bgmusic = sound/theme.ogg
bgvolume = 20
bgmloopstart = 44100
bgmloopend = 88200

[BGdef]

[BG sky]
type = normal
spriteno = 0, 0
start = 0, 0
velocity = 1, 0
id = 1
window = 0, 0, 99, 49

[BG cloud]
type = normal
spriteno = 0, 1
start = 10, 5
positionlink = 1
sin.y = 4, 20, 0

[BG door]
type = anim
actionno = 10
id = 2

[Begin Action 10]
5,0, 0,0, 3
5,1, 0,0, 3

[Begin Action 11]
6,0, 0,0, -1

[BGCtrlDef move]
looptime = 100
ctrlid = 1

[BGCtrl stop]
type = VelSet
time = 10
x = 0

[BGCtrl jump]
type = PosAdd
time = 20
x = 50

[BGCtrl hide]
type = Visible
time = 30, 39
value = 0

[BGCtrl show]
type = Visible
time = 40
value = 1

[BGCtrlDef door]
ctrlid = 2

[BGCtrl swap]
type = Anim
time = 5
value = 11
"#;

fn stage() -> Arc<ume_formats::stage::MugenStage> {
    let mut v = Vfs::new();
    v.insert("stages/ctrl.def".into(), DEF.as_bytes().to_vec());
    Arc::new(load_stage(&v, "stages/ctrl.def").unwrap())
}

#[test]
fn stage_def_extras_are_parsed() {
    let st = stage();
    assert_eq!((st.start_x, st.start_y, st.bound_high, st.zoom_out), (30.0, -10.0, -40.0, 0.8));
    assert_eq!(st.shadow.intensity, 96.0);
    assert_eq!(st.shadow.color, [10.0, 20.0, 30.0]);
    assert_eq!(st.shadow.yscale, -0.3);
    assert_eq!(st.reflection, 64.0);
    let music = st.music.as_ref().expect("music section");
    assert_eq!((music.file.as_str(), music.volume, music.loop_start, music.loop_end), ("sound/theme.ogg", 20.0, 44100, 88200));
    assert_eq!(st.layers[0].window, Some([0.0, 0.0, 99.0, 49.0]));
    assert!(st.layers[1].position_link);
    assert_eq!(st.layers[1].sin_y, [4.0, 20.0, 0.0]);
    assert_eq!(st.ctrl_defs.len(), 2);
    assert_eq!(st.ctrl_defs[0].ctrls.len(), 4);
    assert_eq!(st.ctrl_defs[0].ctrls[1].kind, BgCtrlKind::PosAdd(Some(50.0), None));
    assert_eq!(st.ctrl_defs[0].ctrls[2].end, 39);
}

#[test]
fn bgctrl_drives_the_layers() {
    let mut bg = BgState::new(stage());
    for _ in 0..10 { bg.step(); }
    assert_eq!(bg.offset(0)[0], 10.0, "velocity 1 for ten ticks");
    bg.step();
    let x = bg.offset(0)[0];
    for _ in 0..5 { bg.step(); }
    assert_eq!(bg.offset(0)[0], x, "VelSet x = 0 stopped it");
    for _ in 0..10 { bg.step(); }
    assert!(bg.offset(0)[0] >= x + 50.0, "PosAdd moved it");
    assert_eq!(bg.offset(1)[0], bg.offset(0)[0], "positionlink follows the parent layer");
    for _ in 0..8 { bg.step(); }
    assert!(!bg.visible(0) && !bg.visible(1), "hidden, and the linked layer too");
    for _ in 0..12 { bg.step(); }
    assert!(bg.visible(0));
}

#[test]
fn bgctrl_anim_swaps_the_action() {
    let mut bg = BgState::new(stage());
    assert_eq!(bg.sprite(2), (5, 0));
    for _ in 0..4 { bg.step(); }
    assert_eq!(bg.sprite(2), (5, 1));
    for _ in 0..3 { bg.step(); }
    assert_eq!(bg.sprite(2), (6, 0));
    bg.reset();
    assert_eq!(bg.sprite(2), (5, 0));
}

#[test]
fn match_uses_stage_camera_start_and_steps_the_background() {
    let st = stage();
    let mut info = ume_engine::common::flat_stage();
    info.start_x = st.start_x;
    let mut m = ume_engine::matchs::Match::new(ume_engine::common::common_def(), None, info);
    m.bg = Some(BgState::new(st));
    m.start_round();
    assert_eq!(m.camera.x, 30.0);
    m.tick(&[]);
    assert_eq!(m.bg.as_ref().unwrap().offset(0)[0], 1.0);
}
