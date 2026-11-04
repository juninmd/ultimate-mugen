mod support;
use support::*;
use ume_engine::matchs::Match;

fn striker(params: &str) -> String {
    format!("
[Statedef 200]
type = S
movetype = A
physics = S
anim = 200
ctrl = 0

[State 200, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NA
damage = 40
hitflag = MAF
guardflag = MA
pausetime = 0, 0
sparkno = S1000
guard.sparkno = S1000
{params}
")
}

const GUARDER: &str = "
[Statedef 0]
type = S
movetype = I
physics = S
anim = 0
ctrl = 1

[State 0, guard]
type = AssertSpecial
trigger1 = 1
flag = autoguard
";

fn first_spark_scale(attacker: &str, defender: &str) -> [f32; 2] {
    let mut m: Match = duel(mugen_char("a", attacker), mugen_char("b", defender));
    face_off(&mut m, 55.0);
    enter(&mut m, 0, 200);
    for _ in 0..40 {
        m.tick(&[0, 0]);
        if let Some(e) = m.explods.first() { return e.scale; }
    }
    panic!("no spark was spawned");
}

#[test]
fn sparkscale_scales_the_hit_spark() {
    let plain = first_spark_scale(&striker(""), "");
    let big = first_spark_scale(&striker("sparkscale = 3"), "");
    assert!((big[0] / plain[0] - 3.0).abs() < 1e-4, "{big:?} vs {plain:?}");
}

#[test]
fn guard_sparkscale_scales_only_the_guard_spark() {
    let plain = first_spark_scale(&striker(""), GUARDER);
    let guarded = first_spark_scale(&striker("sparkscale = 2\nguard.sparkscale = 4"), GUARDER);
    assert!((guarded[0] / plain[0] - 4.0).abs() < 1e-4, "{guarded:?} vs {plain:?}");
}

#[test]
fn guard_sparkscale_defaults_to_sparkscale() {
    let plain = first_spark_scale(&striker(""), GUARDER);
    let guarded = first_spark_scale(&striker("sparkscale = 2"), GUARDER);
    assert!((guarded[0] / plain[0] - 2.0).abs() < 1e-4);
}
