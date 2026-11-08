mod support;
use support::*;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

const CNS: &str = "
[Statedef 600]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 600, projectile helper]
type = Helper
trigger1 = Time = 0
id = 1
helpertype = projectile
stateno = 610
pos = 40, -40

[Statedef 605]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 605, normal helper]
type = Helper
trigger1 = Time = 0
id = 1
helpertype = normal
stateno = 610
pos = 40, -40

[Statedef 610]
type = S
movetype = A
physics = N
anim = 1000
ctrl = 0

[State 610, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NP
damage = 20
hitflag = MAF
guardflag = MA
pausetime = 5, 5
ground.velocity = -2
ground.hittime = 10

[Statedef 620]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 620, owning helper]
type = Helper
trigger1 = Time = 0
id = 2
stateno = 630
ownprojectile = 1

[State 620, sharing helper]
type = Helper
trigger1 = Time = 0
id = 3
stateno = 630

[Statedef 630]
type = S
movetype = A
physics = N
anim = 0
ctrl = 0

[State 630, shoot]
type = Projectile
trigger1 = Time = 0
projid = 9
projanim = 1000
attr = S, NP
damage = 10
hitflag = MAF
pausetime = 0, 0
velocity = 0
offset = 50, -40
projremovetime = 500

[Statedef 640]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 640, sized helper]
type = Helper
trigger1 = Time = 0
id = 4
stateno = 650
size.xscale = 2
size.yscale = 0.5

[State 640, plain helper]
type = Helper
trigger1 = Time = 0
id = 5
stateno = 660

[Statedef 650]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 650, inheriting child]
type = Helper
trigger1 = Time = 0
id = 6
stateno = 660

[State 650, own scale child]
type = Helper
trigger1 = Time = 0
id = 7
stateno = 660
ownclsnscale = 1

[Statedef 660]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0
";

fn ev(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &Expr::new(parse(src).unwrap()))
}

fn setup(gap: f32) -> Match {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", CNS));
    face_off(&mut m, gap);
    m
}

fn helper(m: &Match, id: i32) -> usize {
    (0..m.fighters.len())
        .find(|&i| m.fighters[i].is_helper && m.fighters[i].helper_id == id)
        .expect("helper")
}

fn hurt_size(m: &Match, i: usize) -> (f32, f32) {
    let b = m.fighters[i].boxes(2)[0];
    (b[2] - b[0], b[3] - b[1])
}

#[test]
fn projectile_helper_hits_like_a_projectile() {
    let mut m = setup(60.0);
    enter(&mut m, 0, 600);
    ticks(&mut m, 3);
    let h = helper(&m, 1);
    assert!(m.fighters[h].helper_projectile && !m.fighters[h].helper_player);
    assert!(
        m.fighters[1].life < m.fighters[1].life_max,
        "it landed a hit"
    );
    assert_eq!(
        m.fighters[h].hit_pause, 0,
        "no attacker hit pause, like a projectile"
    );
    assert_eq!(m.fighters[0].hit_pause, 0);
    assert!(
        ev(&m, h, "ProjHit") == 1.0 && ev(&m, h, "ProjContact") == 1.0,
        "proj triggers fire on the helper"
    );
    assert!(
        ev(&m, 0, "ProjHitTime(1)") >= 0.0 && ev(&m, 0, "ProjContactTime(1)") >= 0.0,
        "and on its root, keyed by the helper id"
    );
    let hd = ume_engine::fighter::HitDefData {
        hitflag: "MAF".into(),
        attr_state: "S".into(),
        ..Default::default()
    };
    assert!(
        !m.can_be_hit(h, &hd, 1),
        "a projectile helper has no hurtbox"
    );
}

#[test]
fn normal_helper_keeps_player_style_contact() {
    let mut m = setup(60.0);
    enter(&mut m, 0, 605);
    ticks(&mut m, 3);
    let h = helper(&m, 1);
    assert!(m.fighters[1].life < m.fighters[1].life_max);
    assert!(
        m.fighters[h].hit_pause > 0 || m.fighters[h].move_hit > 0,
        "regular helpers still pause on contact"
    );
    assert_eq!(ev(&m, h, "ProjHit"), 0.0);
}

#[test]
fn ownprojectile_keeps_the_projectile_on_the_helper() {
    let mut m = setup(200.0);
    enter(&mut m, 0, 620);
    ticks(&mut m, 2);
    let (own, shared) = (helper(&m, 2), helper(&m, 3));
    assert_eq!(m.projectiles.len(), 2);
    assert_eq!(ev(&m, own, "NumProj"), 1.0);
    assert_eq!(
        ev(&m, shared, "NumProj"),
        1.0,
        "the sharing helper counts the root's pool"
    );
    assert_eq!(
        ev(&m, 0, "NumProj"),
        1.0,
        "the root only counts what is not owned by a helper"
    );
    assert_eq!(ev(&m, own, "NumProjID(9)"), 1.0);
}

#[test]
fn helper_size_scales_override_and_scale_the_hurtbox() {
    let mut m = setup(200.0);
    enter(&mut m, 0, 640);
    ticks(&mut m, 2);
    let (sized, plain) = (helper(&m, 4), helper(&m, 5));
    let (pw, ph) = hurt_size(&m, plain);
    let (sw, sh) = hurt_size(&m, sized);
    assert_eq!((sw, sh), (pw * 2.0, ph * 0.5));
    assert_eq!(ev(&m, sized, "Const(size.xscale)"), 2.0);
    assert_eq!(ev(&m, plain, "Const(size.xscale)"), 1.0);
    assert_eq!(hurt_size(&m, 0), (pw, ph), "the root is untouched");
}

#[test]
fn ownclsnscale_stops_a_child_helper_from_inheriting_the_parent_scale() {
    let mut m = setup(200.0);
    enter(&mut m, 0, 640);
    ticks(&mut m, 4);
    let (plain, inherited, own) = (helper(&m, 5), helper(&m, 6), helper(&m, 7));
    let (pw, _) = hurt_size(&m, plain);
    assert_eq!(
        hurt_size(&m, inherited).0,
        pw * 2.0,
        "children inherit the parent's size overrides"
    );
    assert_eq!(
        hurt_size(&m, own).0,
        pw,
        "ownclsnscale = 1 starts from the character's own constants"
    );
}
