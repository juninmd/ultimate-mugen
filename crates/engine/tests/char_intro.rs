mod support;
use support::gogeta::*;

#[test]
fn intro_frames() {
    let (Some(d), Ok(out)) = (load(1), std::env::var("UME_EVIDENCE_DIR")) else {
        return;
    };
    let mut m = ume_engine::matchs::Match::new(
        ume_engine::common::common_def(),
        None,
        ume_engine::common::flat_stage(),
    );
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d, 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    let mut shots = vec![];
    for t in 0..460 {
        m.tick(&[0, 0]);
        if t % 60 == 59 || m.round_state == 2 && shots.len() < 8 {
            shots.push(support::render::frame(&m, false));
        }
        if m.round_state == 2 {
            break;
        }
    }
    support::render::sheet(&shots[..shots.len().min(8)], 2)
        .save_png(std::path::Path::new(&out).join("intro-native.png"))
        .unwrap();
}
