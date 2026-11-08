mod support;
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
