use std::sync::Arc;
use ume_engine::chardef::{CharDef, Controller, Param};
use ume_engine::matchs::{BgmAction, Event, Match, StageInfo};
use ume_engine::sctrl::console::{CONSOLE_LINES_PER_SECOND, CONSOLE_MAX_LINES};
use ume_engine::stage_bg::BgState;
use ume_formats::stage::{load_stage, BgCtrlKind};
use ume_formats::vfs::Vfs;

const STAGE: &str = r#"
[Info]
name = "Mod Test"

[BGdef]

[BG sky]
type = normal
spriteno = 0, 0
start = 5, 0
velocity = 1, 0
id = 1

[BG door]
type = normal
spriteno = 0, 1
id = 2

[BGCtrlDef move]
looptime = 100
ctrlid = 1

[BGCtrl hide]
type = Visible
time = 3, 6
value = 0
"#;

fn sctrl(kind: &str, params: &[(&str, &str)]) -> Controller {
    Controller { kind: kind.into(), label: kind.into(), trigger_all: vec![], triggers: vec![], params: params.iter().map(|(k, v)| (k.to_string(), Param::new(v))).collect(), persistent: 1, ignore_hitpause: false, uid: 1 }
}

fn setup() -> (Match, usize) {
    let mut cd = CharDef::empty("p");
    cd.constants.insert("data.life".into(), 1000.0);
    let cd = Arc::new(cd);
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let i = m.add_player(cd.clone(), 0, 0, 1);
    m.add_player(cd, 1, 0, 2);
    let mut v = Vfs::new();
    v.insert("stages/mod.def".into(), STAGE.as_bytes().to_vec());
    m.bg = Some(BgState::new(Arc::new(load_stage(&v, "stages/mod.def").unwrap())));
    (m, i)
}

fn run(m: &mut Match, i: usize, kind: &str, params: &[(&str, &str)]) {
    ume_engine::controllers::run(m, i, &sctrl(kind, params));
}

fn bgm_events(m: &mut Match) -> Vec<BgmAction> {
    std::mem::take(&mut m.events).into_iter().filter_map(|e| if let Event::Bgm(a) = e { Some(a) } else { None }).collect()
}

#[test]
fn play_and_modify_bgm_emit_events() {
    let (mut m, i) = setup();
    run(&mut m, i, "playbgm", &[("bgm", "\"sound/theme.ogg\""), ("volume", "50"), ("loop", "0"), ("loopstart", "10")]);
    run(&mut m, i, "modifybgm", &[("volume", "25")]);
    run(&mut m, i, "playbgm", &[("bgm", "\"\"")]);
    let ev = bgm_events(&mut m);
    assert_eq!(ev, vec![
        BgmAction::Play { file: "sound/theme.ogg".into(), looping: false, volume: 0.5, loop_start: 10, loop_end: 0, freq_mul: 1.0 },
        BgmAction::Volume(0.25),
        BgmAction::Stop,
    ]);
}

#[test]
fn modify_stage_var_updates_bounds_zoffset_and_start() {
    let (mut m, i) = setup();
    m.zoffset0 = 210.0;
    m.camera.x = 150.0;
    run(&mut m, i, "modifystagevar", &[("camera.boundleft", "-50"), ("camera.boundright", "60"), ("camera.startx", "12"), ("camera.starty", "-3"), ("stageinfo.zoffset", "190")]);
    assert_eq!((m.stage.bound_left, m.stage.bound_right), (-50.0, 60.0));
    assert_eq!((m.stage.start_x, m.stage.start_y), (12.0, -3.0));
    assert_eq!(m.stage.zoffset, 190.0);
    assert_eq!(m.zoffset0, 190.0);
    assert_eq!(m.camera.x, 60.0);
}

#[test]
fn modify_stage_bg_targets_layers_by_id() {
    let (mut m, i) = setup();
    run(&mut m, i, "modifystagebg", &[("id", "1"), ("visible", "0"), ("velocity", "3, 4"), ("pos", "20, 7")]);
    let bg = m.bg.as_ref().unwrap();
    assert!(!bg.layers[0].visible);
    assert!(bg.layers[1].visible);
    assert_eq!(bg.layers[0].vel, [3.0, 4.0]);
    assert_eq!(bg.layers[0].pos, [15.0, 7.0]);
    assert_eq!(bg.layers[1].vel, [0.0, 0.0]);
}

#[test]
fn modify_bg_ctrl_changes_timing_and_value_and_resets_each_round() {
    let (mut m, i) = setup();
    run(&mut m, i, "modifybgctrl", &[("id", "1"), ("time", "1, 2"), ("value", "1")]);
    let c = &m.bg.as_ref().unwrap().ctrl_defs[0].ctrls[0];
    assert_eq!((c.start, c.end, c.kind.clone()), (1, 2, BgCtrlKind::Visible(true)));
    m.bg.as_mut().unwrap().reset();
    let c = &m.bg.as_ref().unwrap().ctrl_defs[0].ctrls[0];
    assert_eq!((c.start, c.end, c.kind.clone()), (3, 6, BgCtrlKind::Visible(false)));
}

#[test]
fn modify_player_applies_whitelisted_fields_with_clamping() {
    let (mut m, i) = setup();
    run(&mut m, i, "modifyplayer", &[("lifemax", "500"), ("life", "9999"), ("power", "1200"), ("ctrl", "0"), ("ailevel", "20")]);
    let f = &m.fighters[i];
    assert_eq!((f.life_max, f.life), (500.0, 500.0));
    assert_eq!(f.power, 1200.0);
    assert!(!f.ctrl);
    assert_eq!(f.ai_level, 8);
    let enemy_id = m.fighters[1].id;
    run(&mut m, i, "modifyplayer", &[("id", &enemy_id.to_string()), ("life", "10")]);
    assert_eq!(m.fighters[1].life, 10.0);
    assert_eq!(m.fighters[i].life, 500.0);
}

#[test]
fn print_to_console_formats_and_rate_limits() {
    let (mut m, i) = setup();
    run(&mut m, i, "printtoconsole", &[("text", "\"hp=%d\""), ("params", "life")]);
    assert_eq!(m.console.lines().next().unwrap(), "hp=1000");
    for _ in 0..100 { run(&mut m, i, "printtoconsole", &[("text", "\"spam\"")]); }
    assert_eq!(m.console.lines().count(), CONSOLE_LINES_PER_SECOND as usize);
    assert!(m.console.dropped() > 0);
    m.game_time += 60;
    run(&mut m, i, "printtoconsole", &[("text", "\"later\"")]);
    assert_eq!(m.console.lines().last().unwrap(), "later");
    assert!(m.console.lines().count() <= CONSOLE_MAX_LINES);
}
