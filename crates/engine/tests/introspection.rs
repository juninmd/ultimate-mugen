mod support;
use support::*;
use ume_engine::chardef::safe_expr;
use ume_engine::diagnostics::{audit_cns, Category};
use ume_engine::matchs::Match;

const CNS: &str = r#"
[Statedef 300]
type = S
movetype = I
physics = S
anim = 0
ctrl = 0

[State 300, explod]
type = Explod
trigger1 = Time = 0
id = 7
anim = 1000
pos = 20, -10
postype = p1
vel = 3, -1
scale = 2, 0.5
bindtime = 40
removetime = 90
sprpriority = 4

[State 300, second explod]
type = Explod
trigger1 = Time = 0
id = 7
anim = 1000
pos = 0, 0
postype = p1
removetime = 10

[State 300, projectile]
type = Projectile
trigger1 = Time = 0
projid = 1200
projanim = 1000
projhits = 3
projpriority = 5
velocity = 4, 1
accel = 0.5, 0
offset = 10, -30
projremovetime = 100

[State 300, palfx]
type = PalFX
trigger1 = Time = 0
time = 30
add = 10, 20, 30
mul = 128, 128, 128
color = 100

[State 300, allpalfx]
type = AllPalFX
trigger1 = Time = 0
time = 20
add = 1, 2, 3
"#;

fn setup() -> Match {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", CNS));
    face_off(&mut m, 120.0);
    enter(&mut m, 0, 300);
    ticks(&mut m, 1);
    m
}

fn ev(m: &Match, src: &str) -> f64 { m.ev(0, &safe_expr(src)) }

#[test]
fn explod_var_reads_back_the_explod() {
    let m = setup();
    assert_eq!(ev(&m, "ExplodVar(7, 0, id)"), 7.0);
    assert_eq!(ev(&m, "ExplodVar(7, 0, vel x)"), 3.0);
    assert_eq!(ev(&m, "ExplodVar(7, 0, vel y)"), -1.0);
    assert_eq!(ev(&m, "ExplodVar(7, 0, scale x)"), 2.0);
    assert_eq!(ev(&m, "ExplodVar(7, 0, scale y)"), 0.5);
    assert_eq!(ev(&m, "ExplodVar(7, 0, removetime)"), 90.0);
    assert_eq!(ev(&m, "ExplodVar(7, 0, sprpriority)"), 4.0);
    assert_eq!(ev(&m, "ExplodVar(7, 0, anim)"), 1000.0);
    assert!(ev(&m, "ExplodVar(7, 0, time)") >= 0.0);
    assert_eq!(ev(&m, "ExplodVar(7, 1, removetime)"), 10.0, "the index selects among explods with the same id");
    assert_eq!(ev(&m, "ExplodVar(-1, 1, id)"), 7.0, "a negative id matches any explod");
}

#[test]
fn explod_var_missing_entity_is_undefined() {
    let m = setup();
    assert!(ev(&m, "ExplodVar(99, 0, id)").is_nan());
    assert!(ev(&m, "ExplodVar(7, 5, id)").is_nan());
    assert!(ev(&m, "ExplodVar(99, 0, id) = 0").is_nan(), "a comparison with an undefined value stays undefined");
}

#[test]
fn explod_var_position_follows_pos_convention() {
    let m = setup();
    let expected = m.explods.iter().find(|e| e.id == 7).map(|e| ((e.world_pos(m.camera).x - m.camera.x) / e.lscl) as f64).unwrap();
    assert_eq!(ev(&m, "ExplodVar(7, 0, pos x)"), expected);
}

#[test]
fn proj_var_reads_back_the_projectile() {
    let m = setup();
    assert_eq!(ev(&m, "ProjVar(1200, 0, projid)"), 1200.0);
    assert_eq!(ev(&m, "ProjVar(1200, 0, projhits)"), 3.0);
    assert_eq!(ev(&m, "ProjVar(1200, 0, projpriority)"), 5.0);
    assert_eq!(ev(&m, "ProjVar(1200, 0, projremovetime)"), 100.0);
    assert_eq!(ev(&m, "ProjVar(1200, 0, vel y)"), 1.0);
    assert_eq!(ev(&m, "ProjVar(1200, 0, accel x)"), 0.5);
    assert_eq!(ev(&m, "ProjVar(1200, 0, anim)"), 1000.0);
    assert!(ev(&m, "ProjVar(1200, 0, vel x)") >= 4.0);
    assert!(ev(&m, "ProjVar(5, 0, projid)").is_nan());
}

#[test]
fn palfx_var_reads_own_and_global_effects() {
    let m = setup();
    assert_eq!(ev(&m, "PalFXVar(add.r)"), 10.0);
    assert_eq!(ev(&m, "PalFXVar(add.b)"), 30.0);
    assert_eq!(ev(&m, "PalFXVar(mul.g)"), 128.0);
    assert_eq!(ev(&m, "PalFXVar(color)"), 100.0);
    assert!(ev(&m, "PalFXVar(time)") > 0.0);
    assert_eq!(ev(&m, "PalFXVar(all.add.g)"), 2.0);
    assert_eq!(ev(&m, "PalFXVar(bg.mul.r)"), 256.0, "no background effect reads as the identity");
}

#[test]
fn clsn_var_and_overlap_use_size_and_dummy_boxes() {
    let mut m = setup();
    assert_eq!(ev(&m, "ClsnVar(size, 0, back)"), -15.0);
    assert_eq!(ev(&m, "ClsnVar(size, 0, front)"), 16.0);
    assert_eq!(ev(&m, "ClsnVar(size, 0, top)"), -60.0);
    assert_eq!(ev(&m, "ClsnVar(size, 0, bottom)"), 0.0);
    assert!(ev(&m, "ClsnVar(dummy, 0, back)").is_nan());
    let enemy = m.fighters[1].id;
    let near = format!("ClsnOverlap(size, {enemy}, size)");
    assert_eq!(ev(&m, &near), 0.0);
    m.fighters[1].pos.x = m.fighters[0].pos.x + 5.0;
    assert_eq!(ev(&m, &near), 1.0);
    assert_eq!(ev(&m, &format!("ClsnOverlap(dummy, {enemy}, size)")), 0.0);
}

#[test]
fn identity_and_animation_triggers() {
    let m = setup();
    assert_eq!(ev(&m, "Index"), 0.0);
    assert_eq!(ev(&m, "RunOrder"), 1.0);
    assert_eq!(ev(&m, "IkemenVersion(major)"), 1.0);
    assert_eq!(ev(&m, "HelperIndexExist(0)"), 0.0);
    assert_eq!(ev(&m, "AnimElemVar(group)"), 0.0);
    assert_eq!(ev(&m, "AnimLength"), -1.0);
}

#[test]
fn input_time_counts_held_ticks() {
    let mut m = setup();
    for _ in 0..4 { m.fighters[0].history.push(ume_engine::input::k::A, 1); }
    assert_eq!(ev(&m, "InputTime(a)"), 4.0);
    assert_eq!(ev(&m, "InputTime(b)"), 0.0);
    m.fighters[0].history.push(0, 1);
    assert_eq!(ev(&m, "InputTime(a)"), 0.0);
    assert_eq!(ev(&m, "InputTime(N)"), 1.0);
}

#[test]
fn display_name_compares_strings() {
    let m = setup();
    assert_eq!(ev(&m, "DisplayName = \"a\""), 1.0);
    assert_eq!(ev(&m, "DisplayName = \"other\""), 0.0);
}

#[test]
fn unknown_sub_parameters_are_reported_at_load() {
    let text = "[Statedef 1]\ntype = S\n[State 1, a]\ntype = VelSet\ntrigger1 = ExplodVar(1, 0, pos x) > 0 && ExplodVar(1, 0, nonsense) = 0 && ProjVar(1, 0, pos y) < 3 && PalFXVar(wrong) = 0\nx = 1\n";
    let d = audit_cns("sub.cns", text);
    assert_eq!(d.names(Category::Trigger), vec!["explodvar(nonsense)", "palfxvar(wrong)"]);
}

#[test]
fn supported_introspection_has_no_findings() {
    let text = "[Statedef 1]\ntype = S\n[State 1, a]\ntype = VelSet\ntrigger1 = ClsnOverlap(clsn1, 3, size) || ClsnVar(clsn2, 0, top) < 0 || InputTime(F) > 2 || SpriteVar(width) > 0 || HelperVar(id) = 1\nx = 1\n";
    let d = audit_cns("ok.cns", text);
    assert!(d.is_clean(), "{}", d.detail());
}
