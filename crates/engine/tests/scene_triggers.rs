mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::{Match, BASE_H, BASE_W};
use ume_engine::palette::Trans;
use ume_engine::stage_bg::BgState;
use ume_formats::stage::load_stage;
use ume_formats::vfs::Vfs;

fn ev(m: &Match, i: usize, src: &str) -> f64 { m.ev(i, &Expr::new(parse(src).unwrap())) }

fn near(a: f64, b: f64) -> bool { (a - b).abs() < 1e-2 }

fn hd_char() -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char("hd", "")).ok().unwrap();
    d.local_w = 1280.0;
    d.local_h = 720.0;
    Arc::new(d)
}

fn field() -> Match { duel(mugen_char("sd", ""), hd_char()) }

#[test]
fn screen_size_is_the_visible_area_in_each_localcoord() {
    let m = field();
    assert!(near(ev(&m, 0, "ScreenWidth"), BASE_W as f64));
    assert!(near(ev(&m, 0, "ScreenHeight"), BASE_H as f64));
    assert!(near(ev(&m, 1, "ScreenWidth"), BASE_W as f64 * 4.0));
    assert!(near(ev(&m, 1, "ScreenHeight"), BASE_H as f64 * 4.0));
}

#[test]
fn vertical_bounds_at_normal_zoom() {
    let mut m = field();
    m.fighters[0].pos.y = 0.0;
    assert!(near(ev(&m, 0, "TopBoundDist"), 270.0));
    assert!(near(ev(&m, 0, "TopBoundBodyDist"), 210.0));
    assert!(near(ev(&m, 0, "BotBoundDist"), 30.0));
    assert!(near(ev(&m, 0, "BotBoundBodyDist"), 30.0));
    m.fighters[0].pos.y = -100.0;
    assert!(near(ev(&m, 0, "TopBoundDist"), 170.0));
    assert!(near(ev(&m, 0, "BotBoundDist"), 130.0));
    m.fighters[1].pos.y = 0.0;
    assert!(near(ev(&m, 1, "TopBoundDist"), 270.0 * 4.0));
}

#[test]
fn bounds_and_screen_grow_with_the_five_versus_five_zoom_out() {
    let mut m = field();
    m.set_zoom_for_teams(5, 0.5);
    assert!(near(m.zoom as f64, 1.4));
    assert!(near(ev(&m, 0, "ScreenWidth"), BASE_W as f64 * 1.4));
    assert!(near(ev(&m, 0, "ScreenHeight"), BASE_H as f64 * 1.4));
    m.fighters[0].pos.y = 0.0;
    assert!(near(ev(&m, 0, "TopBoundDist"), 318.0));
    assert!(near(ev(&m, 0, "BotBoundDist"), 78.0));
}

#[test]
fn stage_edge_distances_follow_facing_and_the_stage_bounds() {
    let mut m = field();
    let half = BASE_W as f64 / 2.0;
    m.fighters[0].pos.x = 50.0;
    m.fighters[0].facing = 1.0;
    assert!(near(ev(&m, 0, "StageFrontEdgeDist"), 200.0 + half - 50.0));
    assert!(near(ev(&m, 0, "StageBackEdgeDist"), 50.0 + 200.0 + half));
    m.fighters[0].facing = -1.0;
    assert!(near(ev(&m, 0, "StageFrontEdgeDist"), 50.0 + 200.0 + half));
    assert!(near(ev(&m, 0, "StageBackEdgeDist"), 200.0 + half - 50.0));
}

#[test]
fn localcoord_components_and_the_1080p_constant() {
    let m = field();
    assert_eq!((ev(&m, 0, "LocalCoord X"), ev(&m, 0, "LocalCoord Y")), (320.0, 240.0));
    assert_eq!((ev(&m, 1, "LocalCoord X"), ev(&m, 1, "LocalCoord Y")), (1280.0, 720.0));
    assert!(near(ev(&m, 1, "Const1080p(30)"), 30.0 * 1280.0 / 1920.0));
    assert!(near(ev(&m, 0, "Const1080p(60)"), 10.0));
}

const BG_DEF: &str = r#"
[Info]
name = "Bg"

[Constants]
gravity = 0.5
Wind.Speed = -2

[BGdef]

[BG sky]
type = normal
spriteno = 5, 2
start = 10, 20
delta = 0.5, 0.8
velocity = 1, 0
layerno = 1
id = 3

[BG ground]
type = normal
spriteno = 0, 0
id = 4
"#;

fn bg_match() -> Match {
    let mut v = Vfs::new();
    v.insert("stages/b.def".into(), BG_DEF.as_bytes().to_vec());
    let mut m = field();
    m.bg = Some(BgState::new(Arc::new(load_stage(&v, "stages/b.def").unwrap())));
    m
}

#[test]
fn stage_background_queries() {
    let mut m = bg_match();
    assert_eq!(ev(&m, 0, "NumStageBG"), 2.0);
    assert_eq!(ev(&m, 0, "NumStageBG(3)"), 1.0);
    assert_eq!(ev(&m, 0, "NumStageBG(9)"), 0.0);
    assert_eq!(ev(&m, 0, "StageBGVar(3,start.x)"), 10.0);
    assert_eq!(ev(&m, 0, "StageBGVar(3,delta.y)"), 0.8f32 as f64);
    assert_eq!(ev(&m, 0, "StageBGVar(3,layerno)"), 1.0);
    assert_eq!(ev(&m, 0, "StageBGVar(3,sprite.group)"), 5.0);
    assert_eq!(ev(&m, 0, "StageBGVar(3,pos.x)"), 10.0);
    for _ in 0..3 { m.bg.as_mut().unwrap().step(); }
    assert_eq!(ev(&m, 0, "StageBGVar(3,pos.x)"), 13.0);
    assert_eq!(ev(&m, 0, "StageBGVar(3,velocity.x)"), 1.0);
    assert_eq!(ev(&m, 0, "StageBGVar(9,pos.x)"), 0.0);
    assert_eq!(ev(&m, 0, "StageBGVar(3,visible)"), 1.0);
}

#[test]
fn stage_constants_come_from_the_constants_group() {
    let m = bg_match();
    assert_eq!(ev(&m, 0, "StageConst(gravity)"), 0.5);
    assert_eq!(ev(&m, 0, "StageConst(Wind.Speed)"), -2.0);
    assert_eq!(ev(&m, 0, "StageConst(missing)"), 0.0);
    let flat = field();
    assert_eq!(ev(&flat, 0, "StageConst(gravity)"), 0.0);
    assert_eq!(ev(&flat, 0, "NumStageBG"), 0.0);
}

#[test]
fn drawing_state_triggers_read_the_fighter() {
    let mut m = field();
    assert_eq!((ev(&m, 0, "Alpha X"), ev(&m, 0, "Alpha Y")), (256.0, 0.0));
    assert_eq!((ev(&m, 0, "Scale X"), ev(&m, 0, "Scale Y")), (1.0, 1.0));
    m.fighters[0].angle = 30.0;
    m.fighters[0].draw_scale = [1.5, 2.0];
    m.fighters[0].trans = Some(Trans::parse("add", Some((200.0, 100.0))));
    m.fighters[0].spr_priority = 3;
    assert_eq!(ev(&m, 0, "Angle"), 30.0);
    assert_eq!((ev(&m, 0, "Scale X"), ev(&m, 0, "Scale Y")), (1.5, 2.0));
    assert_eq!((ev(&m, 0, "Alpha X"), ev(&m, 0, "Alpha Y")), (200.0, 100.0));
    assert_eq!(ev(&m, 0, "SprPriority"), 3.0);
    assert_eq!((ev(&m, 0, "XAngle"), ev(&m, 0, "YAngle"), ev(&m, 0, "XShear"), ev(&m, 0, "LayerNo")), (0.0, 0.0, 0.0, 0.0));
}
