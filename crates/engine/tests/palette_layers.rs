mod support;
use support::*;
use ume_engine::palette::PalFx;

#[test]
fn palette_layers_stack_bg_then_all_then_own() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.bg_palfx = Some(PalFx::new(0, [10.0, 0.0, 0.0], [256.0; 3]));
    m.all_palfx = Some(PalFx::new(0, [20.0, 0.0, 0.0], [256.0; 3]));
    m.fighters[0].palfx = Some(PalFx::new(0, [30.0, 0.0, 0.0], [256.0; 3]));
    let stage = m.stage_pal();
    assert!((stage.add[0] - 30.0).abs() < 1e-6, "stage = BGPalFX + AllPalFX: {}", stage.add[0]);
    let fighter = m.fighter_pal(0);
    assert!((fighter.add[0] - 50.0).abs() < 1e-6, "fighter = own + AllPalFX: {}", fighter.add[0]);
    assert!((fighter.mul[0] - 256.0).abs() < 1e-6);
}

#[test]
fn palette_changes_apply_on_the_same_tick() {
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    assert!(m.fighter_pal(0).is_identity());
    m.all_palfx = Some(PalFx::new(5, [40.0, 0.0, 0.0], [256.0; 3]));
    let p = m.fighter_pal(0);
    assert!((p.add[0] - 40.0).abs() < 1e-6, "no extra frame of latency: {}", p.add[0]);
    m.all_palfx = None;
    assert!(m.fighter_pal(0).is_identity());
}
