use super::*;
use super::window::parse_window;

#[test]
fn window_size_prefers_saved_window_over_preset() {
    let mut s = Settings::default();
    assert_eq!(s.window_size(), (1920, 1080));
    s.window = (1000, 700);
    assert_eq!(s.window_size(), (1000, 700));
    s.window = (10, 10);
    assert_eq!(s.window_size(), (1920, 1080));
}

#[test]
fn window_string_round_trips_and_rejects_garbage() {
    assert_eq!(parse_window("1366x768"), (1366, 768));
    assert_eq!(parse_window("junk"), (0, 0));
    assert_eq!(parse_window("12x"), (0, 0));
}

#[test]
fn watch_reports_only_after_the_resize_settles() {
    let mut w = WindowWatch::default();
    assert_eq!(w.observe((900, 600), 0.1, false, (0, 0)), None);
    assert_eq!(w.observe((900, 600), 0.5, false, (0, 0)), None);
    assert_eq!(w.observe((900, 600), 0.5, false, (0, 0)), Some((900, 600)));
    assert_eq!(w.observe((900, 600), 0.5, false, (900, 600)), None);
    assert_eq!(w.observe((910, 600), 2.0, false, (900, 600)), None);
}

#[test]
fn watch_ignores_fullscreen_and_minimised_windows() {
    let mut w = WindowWatch::default();
    for _ in 0..10 {
        assert_eq!(w.observe((1920, 1080), 0.5, true, (0, 0)), None);
    }
    for _ in 0..10 {
        assert_eq!(w.observe((0, 0), 0.5, false, (0, 0)), None);
    }
}

#[test]
fn version_one_files_load_with_new_defaults() {
    let s = Settings::from_text("fullscreen=1\nres=3\nquality=2\nfps=1\nfx=1\nimpact_lv=0\n");
    assert!(s.fullscreen && s.fps);
    assert_eq!((s.res, s.quality, s.fx, s.impact), (3, 2, 1, 0));
    assert_eq!(
        (s.vsync, s.master, s.rounds, s.lang),
        (true, 8, 1, crate::i18n::Lang::En)
    );
}

#[test]
fn text_round_trips() {
    let s = Settings {
        vsync: false,
        fps_cap: 3,
        master: 3,
        music: 0,
        sfx: 7,
        mute: true,
        difficulty: 7,
        round_time: 3,
        rounds: 2,
        life: 4,
        lang: crate::i18n::Lang::En,
        ..Settings::default()
    };
    assert_eq!(Settings::from_text(&s.to_text()), s);
}

#[test]
fn garbage_is_clamped() {
    let s = Settings::from_text("master=99\nres=abc\nlife=42\ndifficulty=0\nrounds=-1\n");
    assert_eq!(
        (s.master, s.res, s.life, s.difficulty, s.rounds),
        (VOLUME_STEPS, 1, LIFE_PERCENT.len() - 1, 1, 1)
    );
}

#[test]
fn rules_follow_the_chosen_options() {
    let r = Settings {
        round_time: 3,
        rounds: 2,
        life: 3,
        ..Settings::default()
    }
    .rules();
    assert_eq!(
        (r.rounds_to_win, r.round_ticks, r.life_percent),
        (3, 0, 150)
    );
    assert_eq!(Settings::default().rules(), Rules::default());
}

#[test]
fn gameplay_options_reach_the_rules() {
    let r = Settings {
        single_round: true,
        classic_pushbox: false,
        team_life_share: false,
        game_speed: 4,
        ..Settings::default()
    }
    .rules();
    assert!(r.single_round && !r.classic_pushbox && !r.team_life_share);
    assert!((r.game_speed - GAME_SPEEDS[4]).abs() < 1e-6);
    let s = Settings {
        single_round: true,
        classic_pushbox: false,
        team_life_share: false,
        game_speed: 4,
        ..Settings::default()
    };
    assert_eq!(Settings::from_text(&s.to_text()), s);
}

#[test]
fn mute_silences_master() {
    assert_eq!(
        Settings {
            mute: true,
            ..Settings::default()
        }
        .master_gain(),
        0.0
    );
    assert!((Settings::default().master_gain() - 0.8).abs() < 1e-6);
}

#[test]
fn grid_preset_survives_a_save_and_load() {
    let mut s = Settings::default();
    s.grid = 2;
    let back = Settings::from_text(&s.to_text());
    assert_eq!(back.grid, 2);
    assert_eq!(Settings::default().grid, 1);
}
