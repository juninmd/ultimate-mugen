mod support;
use support::*;
use ume_engine::matchs::Match;

fn shooter(params: &str) -> String {
    format!("
[Statedef 1000]
type = S
movetype = A
physics = S
anim = 0

[State 1000, p]
type = Projectile
trigger1 = Time = 0
projanim = 1000
attr = S, NP
damage = 10
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
{params}
")
}

fn fire(m: &mut Match, i: usize) { enter(m, i, 1000); m.tick(&[0, 0]); }

#[test]
fn equal_priority_projectiles_trade() {
    let mut m = duel(mugen_char("a", &shooter("velocity = 4\nprojpriority = 2")), mugen_char("b", &shooter("velocity = 4\nprojpriority = 2")));
    face_off(&mut m, 120.0);
    fire(&mut m, 0);
    m.fighters[1].facing = -1.0;
    let i1 = 1; enter(&mut m, i1, 1000); m.tick(&[0, 0]);
    assert_eq!(m.projectiles.len(), 2);
    ticks(&mut m, 30);
    assert!(m.projectiles.is_empty(), "both cancelled each other");
}

#[test]
fn higher_priority_survives_and_loses_one_point() {
    let mut m = duel(mugen_char("a", &shooter("velocity = 4\nprojpriority = 3")), mugen_char("b", &shooter("velocity = 4\nprojpriority = 1")));
    face_off(&mut m, 120.0);
    fire(&mut m, 0);
    enter(&mut m, 1, 1000);
    for _ in 0..30 { m.tick(&[0, 0]); if m.projectiles.len() == 1 { break; } }
    assert_eq!(m.projectiles.len(), 1);
    assert_eq!(m.projectiles[0].priority, 2);
    assert_eq!(m.projectiles[0].owner, m.fighters[0].id);
}

#[test]
fn height_and_stage_bounds_remove_projectiles() {
    let mut m = duel(mugen_char("a", &shooter("velocity = 0, -6\nprojheightbound = -50, 1")), mugen_char("b", ""));
    face_off(&mut m, 200.0);
    fire(&mut m, 0);
    assert_eq!(m.projectiles.len(), 1);
    ticks(&mut m, 6);
    assert_eq!(m.projectiles.len(), 1, "still under the height bound at y = {}", m.projectiles.first().map(|p| p.pos.y).unwrap_or(0.0));
    ticks(&mut m, 10);
    assert!(m.projectiles.is_empty(), "removed once above projheightbound");

    let mut m = duel(mugen_char("a", &shooter("velocity = -9\nprojedgebound = 1000\nprojstagebound = 10")), mugen_char("b", ""));
    face_off(&mut m, 200.0);
    fire(&mut m, 0);
    ticks(&mut m, 200);
    assert!(m.projectiles.is_empty(), "removed past projstagebound even with a large edge bound");
}

#[test]
fn projremove_zero_keeps_the_projectile_after_hitting() {
    let mut m = duel(mugen_char("a", &shooter("velocity = 0\nprojremove = 0\nprojmisstime = 50\noffset = 40, -30")), mugen_char("b", ""));
    face_off(&mut m, 50.0);
    fire(&mut m, 0);
    ticks(&mut m, 5);
    assert!(m.fighters[1].life < m.fighters[1].life_max, "it hit");
    assert_eq!(m.projectiles.len(), 1);
    assert!(!m.projectiles[0].dying);
}

#[test]
fn projectiles_freeze_in_pause_unless_given_move_time() {
    for (move_time, moves) in [(0, false), (20, true)] {
        let freezer = "
[Statedef 900]
type = S
physics = S

[State 900, sp]
type = SuperPause
trigger1 = Time = 0
time = 30
anim = -1
";
        let mut m = duel(mugen_char("a", &shooter(&format!("velocity = 3\nsupermovetime = {move_time}"))), mugen_char("b", freezer));
        face_off(&mut m, 200.0);
        fire(&mut m, 0);
        enter(&mut m, 1, 900);
        m.tick(&[0, 0]);
        assert!(m.pause.is_some());
        let x0 = m.projectiles[0].pos.x;
        ticks(&mut m, 10);
        assert_eq!(m.projectiles[0].pos.x != x0, moves, "supermovetime = {move_time}");
    }
}
