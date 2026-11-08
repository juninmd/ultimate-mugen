mod support;
use support::*;

const JAB: &str = "
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
damage = 30
animtype = light
hitflag = MAF
guardflag = MA
pausetime = 0, 0
ground.velocity = -6
ground.hittime = 10
ground.slidetime = 10

[State 200, end]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
";

fn cornered_jab(attacker: std::sync::Arc<ume_engine::chardef::CharDef>) -> (u32, f32) {
    let mut m = duel(attacker, mugen_char("dummy", ""));
    m.stage.bound_left = 0.0;
    m.stage.bound_right = 0.0;
    m.camera.x = 0.0;
    let edge = m.view_w / 2.0 - 16.0;
    m.fighters[1].pos.x = edge;
    m.fighters[0].pos.x = edge - 40.0;
    let before = m.fighters[1].life;
    enter(&mut m, 0, 200);
    ticks(&mut m, 20);
    (m.corner_pushes, before - m.fighters[1].life)
}

#[test]
fn corner_push_back_is_native_only() {
    let (pushes, dealt) = cornered_jab(mugen_char("mugen", JAB));
    assert!(dealt > 0.0, "the MUGEN jab landed");
    assert_eq!(pushes, 0, "MUGEN attackers keep MUGEN corner behaviour");
    let (pushes, dealt) = cornered_jab(native_char("umf", JAB));
    assert!(dealt > 0.0, "the native jab landed");
    assert!(pushes > 0, "native attackers get the extra corner push");
}

#[test]
fn combo_proration_is_native_only() {
    for (native, expect_full) in [(false, true), (true, false)] {
        let d = if native {
            native_char("a", JAB)
        } else {
            mugen_char("a", JAB)
        };
        let mut m = duel(d, mugen_char("dummy", ""));
        face_off(&mut m, 40.0);
        let mut dealt = vec![];
        for _ in 0..4 {
            face_off(&mut m, 40.0);
            let before = m.fighters[1].life;
            enter(&mut m, 0, 200);
            ticks(&mut m, 4);
            m.fighters[1].hit_over_time = 30;
            dealt.push(before - m.fighters[1].life);
            ticks(&mut m, 12);
        }
        assert!(
            dealt.iter().all(|&d| d > 0.0),
            "every jab landed: {dealt:?}"
        );
        assert_eq!(
            dealt.iter().all(|&d| d == 30.0),
            expect_full,
            "native={native} damage per hit {dealt:?}"
        );
    }
}
