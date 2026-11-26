use super::beat::{sample, Beat};
use super::config::Config;
use super::ease::Ease;
use super::{close_up, frame_with, Rig, Sample, Subject};
use ume_engine::matchs::Match;

fn duel() -> Match {
    let mut m = Match::new(ume_engine::common::common_def(), None, ume_engine::common::flat_stage());
    m.add_player(ume_engine::common::common_def(), 0, 0, 1);
    m.add_player(ume_engine::common::common_def(), 1, 0, 2);
    m.fighters[0].pos.x = -60.0;
    m.fighters[1].pos.x = 60.0;
    m
}

fn defaults() -> Config {
    Config::parse(include_str!("../../assets/cinematic.toml"), None)
}

fn intro_at(m: &mut Match, ticks: i32) {
    m.round_state = 1;
    m.round_no = 1;
    m.intro_ticks = 170;
    m.round_time = ticks;
}

#[test]
fn every_ease_starts_at_zero_and_lands_on_one() {
    for ease in [Ease::Linear, Ease::In, Ease::Out, Ease::InOut, Ease::Back] {
        assert!(ease.apply(0.0).abs() < 1e-6, "{ease:?}");
        assert!((ease.apply(1.0) - 1.0).abs() < 1e-5, "{ease:?}");
    }
}

#[test]
fn the_shipped_toml_parses_into_non_empty_shots_that_fit_the_round_windows() {
    let cfg = defaults();
    let total = |beats: &[Beat]| beats.iter().map(|b| b.duration).sum::<f32>();
    assert!(cfg.enabled);
    assert!(total(&cfg.intro) <= (60.0 + 170.0) / 60.0 + 0.01, "intro must end before FIGHT");
    assert!(total(&cfg.victory) <= 170.0 / 60.0, "victory must fit the shortest win pose");
}

#[test]
fn the_intro_hands_the_camera_from_p1_to_p2_then_pulls_back_to_neutral() {
    let cfg = defaults();
    let mut m = duel();
    intro_at(&mut m, 30);
    assert_eq!(frame_with(&m, &cfg).unwrap().subject, Subject::P1);
    intro_at(&mut m, 90);
    assert_eq!(frame_with(&m, &cfg).unwrap().subject, Subject::P2);
    intro_at(&mut m, 227);
    let end = frame_with(&m, &cfg).unwrap().sample;
    assert!(end.zoom.abs() < 0.02 && end.focus < 0.02 && end.yaw == 0.0, "{end:?}");
    assert!(end.pitch.abs() + end.lens + end.detail + end.sweep.abs() < 0.01, "{end:?}");
}

#[test]
fn skipping_the_intro_or_training_disables_the_shot() {
    let cfg = defaults();
    let mut m = duel();
    intro_at(&mut m, 30);
    m.intro_skipped = true;
    assert!(frame_with(&m, &cfg).is_none());
    m.intro_skipped = false;
    m.training = true;
    assert!(frame_with(&m, &cfg).is_none());
}

#[test]
fn only_the_round_winner_orbits_a_full_turn_during_victory() {
    let cfg = defaults();
    let mut m = duel();
    m.round_state = 4;
    m.winner_side = 1;
    m.round_time = 169;
    let frame = frame_with(&m, &cfg).unwrap();
    assert!(frame.covers(1, m.winner_side) && !frame.covers(0, m.winner_side));
    assert!((frame.sample.yaw - std::f32::consts::TAU).abs() < 0.05, "{:?}", frame.sample);
    m.winner_side = -2;
    assert!(frame_with(&m, &cfg).is_none(), "a draw has no hero shot");
}

#[test]
fn the_disabled_flag_and_user_overrides_are_honoured() {
    let off = Config::parse(include_str!("../../assets/cinematic.toml"), Some("enabled = false"));
    let mut m = duel();
    intro_at(&mut m, 30);
    assert!(frame_with(&m, &off).is_none());
    let custom = "[[victory]]\nsubject = \"both\"\nduration = 2.0\nzoom = [0.0, 0.3]\n";
    let cfg = Config::parse(include_str!("../../assets/cinematic.toml"), Some(custom));
    assert_eq!(cfg.victory.len(), 1);
    assert_eq!(cfg.intro.len(), 3, "untouched sections keep the defaults");
    let broken = Config::parse(include_str!("../../assets/cinematic.toml"), Some("[[intro"));
    assert_eq!(broken.intro.len(), 3, "invalid user file falls back to the defaults");
}

#[test]
fn hostile_values_are_clamped() {
    let beat = Beat {
        duration: f32::NAN,
        zoom: [9.0, f32::INFINITY],
        yaw: [1e9, -1e9],
        ..Default::default()
    }
    .sanitized();
    assert_eq!(beat.duration, 1.0);
    assert!(beat.zoom[0] <= 1.5 && beat.zoom[1] == 0.0);
    assert!(beat.yaw[0].abs() < 13.0 && beat.yaw[1].abs() < 13.0);
    assert!(sample(&[], 1.0).is_none());
}

#[test]
fn the_rig_eases_toward_the_subject_and_back_without_snapping() {
    let mut m = duel();
    let mut rig = Rig::default();
    intro_at(&mut m, 60);
    rig.retarget(&m);
    rig.ease(1.0 / 60.0);
    let first = rig.zoom(1.0);
    assert!(first > 0.0 && first < 0.2, "must ease in, got {first}");
    for _ in 0..120 {
        rig.ease(1.0 / 60.0);
    }
    assert!(rig.zoom(1.0) > 0.3);
    let mid = (0.0, 0.0);
    assert!(rig.focus(mid, 1.0).0 != mid.0, "focus must lean toward the subject");
    assert_eq!(rig.focus(mid, 0.0), mid, "reduce motion disables the pan");
    m.round_state = 2;
    rig.retarget(&m);
    for _ in 0..240 {
        rig.ease(1.0 / 60.0);
    }
    assert!(rig.zoom(1.0).abs() < 0.01 && rig.focus(mid, 1.0).0.abs() < 1.0);
}

#[test]
fn a_face_close_up_lands_the_aim_point_on_the_chest_line() {
    let view = Sample { detail: 1.0, aim: 0.88, ..Default::default() };
    let (mut scale, mut feet) = (2.0, 500.0);
    let height = 100.0;
    let chest_line = feet - 0.55 * height * scale;
    close_up(&mut scale, &mut feet, height, &view);
    assert_eq!(scale, 4.0);
    let aim_y = feet - 0.88 * height * scale;
    assert!((aim_y - chest_line).abs() < 1e-3, "{aim_y} vs {chest_line}");
    let (mut s2, mut f2) = (2.0, 500.0);
    close_up(&mut s2, &mut f2, height, &Sample::default());
    assert_eq!((s2, f2), (2.0, 500.0), "no detail leaves the placement untouched");
}
