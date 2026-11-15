use super::*;
#[test]
fn heavy_events_shake_props_and_leave_marks() {
    let mut fx = StageReact::new(StageKind::Plains);
    fx.apply(Impulse {
        x: 10.0,
        strength: 0.9,
        kind: ImpulseKind::Hit,
    });
    assert!(
        fx.shake > 0.5 && fx.flicker > 0.5,
        "heavy hit shakes props and flickers lights"
    );
    assert!(
        !fx.decals.is_empty() && !fx.bits.is_empty() && !fx.shocks.is_empty(),
        "cracks, debris and a shock wave"
    );
    let mut ko = StageReact::new(StageKind::Dojo);
    ko.apply(Impulse {
        x: 0.0,
        strength: 1.0,
        kind: ImpulseKind::Ko,
    });
    assert_eq!(ko.cheer, 1.0, "the crowd goes wild on a KO");
    assert!(ko.push_at(500.0).abs() < 1e-3, "shock waves are local");
}
#[test]
fn light_taps_do_not_leave_cracks() {
    let mut fx = StageReact::new(StageKind::Volcano);
    fx.apply(Impulse {
        x: 0.0,
        strength: 0.2,
        kind: ImpulseKind::Hit,
    });
    assert!(fx.decals.is_empty());
    assert!(fx.shake < 0.2);
}
#[test]
fn mugen_stages_are_untouched() {
    assert!(!StageReact::new(StageKind::Custom).active());
}
#[test]
fn explicit_impulses_are_queued() {
    let mut fx = StageReact::new(StageKind::Neon);
    fx.react(&[Impulse {
        x: 0.0,
        strength: 1.0,
        kind: ImpulseKind::Super,
    }]);
    assert!(fx.explicit && fx.queue.len() == 1);
}
