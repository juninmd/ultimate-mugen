use crate::support;
use support::*;

const CNS: &str = r#"
[Statedef 300]
type = S
movetype = I
physics = S
anim = 0
ctrl = 0

[State 300, held]
type = Explod
trigger1 = Time = 0
id = 1
anim = 1000
pos = 20, -10
postype = p1
bindtime = -1
removetime = -1
ignorehitpause = 0

[State 300, free]
type = Explod
trigger1 = Time = 0
id = 2
anim = 1000
pos = 0, 0
postype = p1
removetime = -1
vel = 2, 0

[State 300, screen]
type = Explod
trigger1 = Time = 0
id = 3
anim = 1000
pos = 30, 40
postype = left
removetime = -1

[State 300, space]
type = Explod
trigger1 = Time = 0
id = 4
anim = 1000
pos = 0, 0
postype = p1
space = screen
removetime = -1

[State 300, tinted]
type = Explod
trigger1 = Time = 0
id = 5
anim = 1000
pos = 0, 0
postype = p1
removetime = -1
alpha = 64, 0
angle = -90

[Statedef 310]
type = S
movetype = I
physics = S
anim = 0
ctrl = 0

[State 310, bind]
type = ExplodBindTime
trigger1 = Time = 0
id = 1
time = 0

[State 310, modify]
type = ModifyExplod
trigger1 = Time = 0
id = 2
facing = -1
vfacing = -1
sprpriority = 7
ontop = 0
removetime = 30
ignorehitpause = 0
bindtime = 5
"#;

const PREFIXED: &str = r#"
[Statedef 320]
type = S
movetype = I
physics = S
anim = 0
ctrl = 0

[State 320, fx]
type = Explod
trigger1 = Time = 0
id = 10
anim = F310+var(1)
pos = 0, 0
postype = p1
removetime = -1

[State 320, own]
type = Explod
trigger1 = Time = 0
id = 11
anim = S1000+var(1)
pos = 0, 0
postype = p1
removetime = -1
"#;

fn setup() -> ume_engine::matchs::Match {
    let a = mugen_char("a", CNS);
    let b = mugen_char("b", CNS);
    let mut m = duel(a, b);
    face_off(&mut m, 120.0);
    enter(&mut m, 0, 300);
    ticks(&mut m, 1);
    m
}

fn explod(m: &ume_engine::matchs::Match, id: i32) -> &ume_engine::matchs::Explod {
    m.explods.iter().find(|e| e.owner == m.fighters[0].id && e.id == id).expect("explod")
}

#[test]
fn prefixed_anim_expressions_pick_fx_and_own_animations() {
    use std::sync::Arc;
    use ume_engine::chardef::CharDef;
    let mut a = mugen_char("a", PREFIXED);
    Arc::get_mut(&mut a)
        .unwrap()
        .anims
        .insert(1005, anim(1005, vec![frame(-1, vec![], vec![])]));
    let b = mugen_char("b", "");
    let mut m = duel(a, b);
    let mut fx = CharDef::empty("fx");
    fx.anims.insert(315, anim(315, vec![frame(-1, vec![], vec![])]));
    m.fx = Some(Arc::new(fx));
    m.fighters[0].vars[1] = 5;
    face_off(&mut m, 120.0);
    enter(&mut m, 0, 320);
    ticks(&mut m, 1);
    assert_eq!(explod(&m, 10).cur.anim, 315, "F310+var(1) loads fightfx 315");
    assert_eq!(explod(&m, 11).cur.anim, 1005, "S1000+var(1) loads the owner animation");
}

#[test]
fn alpha_and_angle_parameters_reach_the_renderer() {
    let m = setup();
    let tinted = explod(&m, 5);
    assert!((tinted.alpha - 0.25).abs() < 1e-6, "alpha 64 of 256 becomes a 0.25 multiplier, got {}", tinted.alpha);
    assert_eq!(tinted.angle, -90.0);
    assert_eq!(explod(&m, 1).alpha, 1.0, "an explod without alpha stays opaque");
    assert_eq!(explod(&m, 1).angle, 0.0);
}

#[test]
fn ignorehitpause_zero_freezes_with_the_owner() {
    let mut m = setup();
    let (t1, t2) = (explod(&m, 1).time, explod(&m, 2).time);
    m.fighters[0].hit_pause = 5;
    ticks(&mut m, 2);
    assert_eq!(explod(&m, 1).time, t1, "ignorehitpause = 0 waits for the owner");
    assert!(explod(&m, 2).time > t2, "the default ignorehitpause = 1 keeps animating");
}

#[test]
fn screen_explods_follow_the_camera() {
    let mut m = setup();
    let left = explod(&m, 3).world_pos(m.camera);
    let anchored = explod(&m, 4).world_pos(m.camera);
    m.camera.x += 50.0;
    assert!(explod(&m, 3).screen && explod(&m, 4).screen);
    assert_eq!(explod(&m, 3).world_pos(m.camera).x, left.x + 50.0);
    assert_eq!(explod(&m, 4).world_pos(m.camera).x, anchored.x + 50.0);
    assert_eq!(explod(&m, 3).pos.x, -m.view_w / 2.0 + 30.0 * m.fighters[0].lscl);
}

#[test]
fn bound_explod_tracks_its_owner_until_explodbindtime_releases_it() {
    let mut m = setup();
    m.fighters[0].pos.x += 40.0;
    ticks(&mut m, 1);
    let f = &m.fighters[0];
    assert_eq!(explod(&m, 1).pos.x, f.pos.x + 20.0 * f.lscl * f.facing);
    enter(&mut m, 0, 310);
    ticks(&mut m, 1);
    let x = explod(&m, 1).pos.x;
    m.fighters[0].pos.x += 40.0;
    ticks(&mut m, 1);
    assert_eq!(explod(&m, 1).pos.x, x, "bindtime 0 releases the explod");
}

#[test]
fn modifyexplod_sets_facing_from_the_owner_and_updates_every_field() {
    let mut m = setup();
    let owner_facing = m.fighters[0].facing;
    enter(&mut m, 0, 310);
    ticks(&mut m, 1);
    let e = explod(&m, 2);
    assert_eq!(e.facing, -owner_facing);
    assert_eq!(e.vfacing, -1.0);
    assert_eq!((e.spr_priority, e.on_top, e.ignore_hit_pause, e.bind_time), (7, false, false, 5));
    assert_eq!(explod(&m, 1).bind_time, 0, "ExplodBindTime does not touch other ids");
    enter(&mut m, 0, 310);
    ticks(&mut m, 1);
    assert_eq!(explod(&m, 2).facing, -owner_facing, "a second ModifyExplod does not flip it back");
    ticks(&mut m, 30);
    assert!(m.explods.iter().all(|e| !(e.owner == m.fighters[0].id && e.id == 2)), "removetime = 30 removes it");
}

#[test]
fn left_postype_y_is_measured_from_the_top_of_the_screen() {
    let m = setup();
    let e = explod(&m, 3);
    let screen_y = e.world_pos(m.camera).y - m.camera.y + m.stage.zoffset;
    assert_eq!(screen_y, 40.0);
}
