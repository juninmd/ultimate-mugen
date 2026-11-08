use crate::vfx::Vfx;
use ume_engine::matchs::Match;

fn duel(apart: f32) -> Match {
    let mut m = Match::new(ume_engine::common::common_def(), None, ume_engine::common::flat_stage());
    m.add_player(ume_engine::common::common_def(), 0, 0, 1);
    m.add_player(ume_engine::common::common_def(), 1, 0, 2);
    m.fighters[0].pos.x = -apart / 2.0;
    m.fighters[1].pos.x = apart / 2.0;
    m
}

fn settled_camera_zoom(m: &Match) -> f32 {
    let mut v = Vfx::new();
    for _ in 0..240 {
        v.tick(m);
        v.update(1.0 / 60.0);
    }
    v.camera(m, 1.0).0
}

#[test]
fn the_neutral_camera_zoom_ignores_the_fighter_distance() {
    let close = settled_camera_zoom(&duel(60.0));
    let far = settled_camera_zoom(&duel(400.0));
    assert!((close - 1.0).abs() < 1e-6, "neutral play must not zoom the stage: {close}");
    assert_eq!(close.to_bits(), far.to_bits(), "the stage must not breathe as the fighters move");
}
