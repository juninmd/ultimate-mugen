mod support;
use support::*;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

const CNS: &str = "
[Statedef 1000]
type = S
movetype = A
physics = N
anim = 0
ctrl = 0

[State 1000, first]
type = Projectile
trigger1 = Time = 0
projid = 1
projanim = 1000
attr = S, NP
damage = 10
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
velocity = 3
offset = 20, -40
projhits = 1
projremove = 1

[State 1000, second]
type = Projectile
trigger1 = Time = 0
projid = 2
projanim = 1000
attr = S, NP
damage = 10
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
velocity = 3
offset = 20, -40

[Statedef 1010]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 1010, helper]
type = Helper
trigger1 = Time = 0
id = 9
stateno = 1020

[Statedef 1020]
type = S
movetype = A
physics = N
anim = 0
ctrl = 0

[State 1020, shoot]
type = Projectile
trigger1 = Time = 0
projid = 7
projanim = 1000
attr = S, NP
damage = 10
hitflag = MAF
pausetime = 0, 0
ground.velocity = -2
ground.hittime = 10
velocity = 3
offset = 20, -40

[State 1020, vanish]
type = DestroySelf
trigger1 = Time = 1
";

fn ev(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &Expr::new(parse(src).unwrap()))
}

fn setup() -> Match {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", CNS));
    face_off(&mut m, 120.0);
    m
}

#[test]
fn projectile_keeps_its_team_side_and_facing_when_the_owner_changes() {
    let mut m = setup();
    enter(&mut m, 0, 1000);
    ticks(&mut m, 1);
    assert_eq!(m.projectiles.len(), 2);
    assert!(m
        .projectiles
        .iter()
        .all(|p| p.teamside == 0 && p.owner == m.fighters[0].id && p.root == m.fighters[0].id));
    let facing = m.projectiles[0].facing;
    m.fighters[0].facing = -facing;
    m.fighters[0].side = 1;
    ticks(&mut m, 1);
    assert!(
        m.projectiles
            .iter()
            .all(|p| p.teamside == 0 && p.facing == facing),
        "turning or switching sides afterwards changes nothing"
    );
    let life = m.fighters[1].life;
    ticks(&mut m, 60);
    assert!(
        m.fighters[1].life < life,
        "the projectile still hits the original opponent"
    );
}

#[test]
fn numproj_and_numprojid_tally_each_projectile_separately() {
    let mut m = setup();
    enter(&mut m, 0, 1000);
    ticks(&mut m, 1);
    assert_eq!(ev(&m, 0, "NumProj"), 2.0);
    assert_eq!(ev(&m, 0, "NumProjID(1)"), 1.0);
    assert_eq!(ev(&m, 0, "NumProjID(2)"), 1.0);
    assert_eq!(ev(&m, 0, "NumProjID(3)"), 0.0);
    assert_eq!(ev(&m, 1, "NumProj"), 0.0, "the opponent owns none");
}

#[test]
fn helper_projectiles_count_for_the_helper_and_its_root_even_after_it_is_gone() {
    let mut m = setup();
    enter(&mut m, 0, 1010);
    ticks(&mut m, 2);
    let h = (0..m.fighters.len())
        .find(|&i| m.fighters[i].is_helper)
        .expect("helper alive");
    assert_eq!(m.projectiles.len(), 1);
    assert_eq!(m.projectiles[0].root, m.fighters[0].id);
    assert_eq!(m.projectiles[0].teamside, 0);
    assert_eq!(ev(&m, h, "NumProj"), 1.0);
    assert_eq!(ev(&m, 0, "NumProj"), 1.0);
    assert_eq!(ev(&m, 0, "NumProjID(7)"), 1.0);
    ticks(&mut m, 2);
    assert!(
        !m.fighters.iter().any(|f| f.is_helper),
        "the helper destroyed itself"
    );
    assert_eq!(m.projectiles.len(), 1, "its projectile outlives it");
    assert_eq!(ev(&m, 0, "NumProj"), 1.0);
    let life = m.fighters[1].life;
    ticks(&mut m, 80);
    assert!(
        m.fighters[1].life < life,
        "and still lands, credited to the root"
    );
}
