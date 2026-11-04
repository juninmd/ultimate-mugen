//! Player settings (video, audio, gameplay rules, language), persisted through `storage` (file on native, localStorage on web).
use crate::gfx::*;

pub const RESOLUTIONS: [(u32, u32, &str); 5] = [(1280, 720, "1280×720 (HD)"), (1920, 1080, "1920×1080 (Full HD)"), (2560, 1440, "2560×1440 (QHD)"), (3840, 2160, "3840×2160 (4K UHD)"), (5120, 2880, "5120×2880 (5K)")];
pub const FX_LEVELS: [&str; 3] = ["Desligado", "Médio", "Alto (bloom)"];
pub const IMPACT: [(f32, &str); 4] = [(0.0, "Mínimo (fotossensibilidade)"), (0.4, "Reduzido"), (1.0, "Normal"), (1.3, "Intenso")];
pub const QUALITY: [(f32, &str); 4] = [(0.0, "Automática"), (3.0, "Padrão (3×)"), (4.5, "Alta (4,5×)"), (6.0, "Ultra 4K (6×)")];

pub const FPS_CAPS: [u32; 5] = [0, 30, 60, 120, 144];
pub const GAME_SPEEDS: [f32; 5] = [0.75, 0.9, 1.0, 1.15, 1.3];
pub const ROUND_SECONDS: [u32; 4] = [99, 60, 30, 0];
pub const ROUNDS_TO_WIN: [i32; 3] = [1, 2, 3];
pub const LIFE_PERCENT: [u32; 5] = [50, 75, 100, 150, 200];
/// Life recovered between Survival fights, in percent of the bar.
pub const SURVIVAL_RECOVERY: [u32; 5] = [0, 20, 35, 50, 100];
pub const VOLUME_STEPS: u8 = 10;
pub const FORMAT_VERSION: u32 = 2;
const FILE: &str = "ultimate-mugen.cfg";

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings {
    pub fullscreen: bool, pub res: usize, pub quality: usize, pub fps: bool, pub fx: usize, pub impact: usize,
    pub vsync: bool, pub fps_cap: usize,
    pub keep_aspect: bool, pub linear: bool, pub window: (u32, u32),
    pub master: u8, pub music: u8, pub sfx: u8, pub mute: bool,
    pub difficulty: i32, pub round_time: usize, pub rounds: usize, pub life: usize,
    pub single_round: bool, pub classic_pushbox: bool, pub team_life_share: bool, pub game_speed: usize,
    /// Survival: life recovered between fights (index into `SURVIVAL_RECOVERY`) and the local best score
    pub survival_recovery: usize, pub survival_best: u64,
    pub lang: crate::i18n::Lang,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            fullscreen: false, res: 1, quality: 0, fps: false, fx: 2, impact: 2,
            vsync: true, fps_cap: 0,
            keep_aspect: true, linear: false, window: (0, 0),
            master: 8, music: 10, sfx: 10, mute: false,
            difficulty: 4, round_time: 0, rounds: 1, life: 2,
            single_round: false, classic_pushbox: true, team_life_share: true, game_speed: 2,
            survival_recovery: 2, survival_best: 0,
            lang: crate::i18n::Lang::Pt,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rules { pub rounds_to_win: i32, pub round_ticks: i32, pub life_percent: u32, pub single_round: bool, pub classic_pushbox: bool, pub team_life_share: bool, pub game_speed: f32 }

impl Default for Rules { fn default() -> Self { Rules { rounds_to_win: 2, round_ticks: 99 * 60, life_percent: 100, single_round: false, classic_pushbox: true, team_life_share: true, game_speed: 1.0 } } }

pub fn auto_quality(h: f32) -> f32 { if h <= 900.0 { 3.0 } else if h <= 1300.0 { 4.0 } else if h <= 1700.0 { 4.5 } else { 6.0 } }

fn pick<T: std::str::FromStr + PartialOrd + Copy>(v: &str, default: T, max: T) -> T { match v.parse::<T>() { Ok(n) if n <= max => n, Ok(_) => max, Err(_) => default } }

impl Settings {
    pub fn art_res(&self, physical_h: f32) -> f32 { let q = QUALITY[self.quality.min(QUALITY.len() - 1)].0; if q <= 0.0 { auto_quality(physical_h) } else { q } }
    pub fn rules(&self) -> Rules {
        Rules {
            rounds_to_win: ROUNDS_TO_WIN[self.rounds.min(ROUNDS_TO_WIN.len() - 1)],
            round_ticks: ROUND_SECONDS[self.round_time.min(ROUND_SECONDS.len() - 1)] as i32 * 60,
            life_percent: LIFE_PERCENT[self.life.min(LIFE_PERCENT.len() - 1)],
            single_round: self.single_round,
            classic_pushbox: self.classic_pushbox,
            team_life_share: self.team_life_share,
            game_speed: GAME_SPEEDS[self.game_speed.min(GAME_SPEEDS.len() - 1)],
        }
    }
    pub fn fps_cap_value(&self) -> u32 { FPS_CAPS[self.fps_cap.min(FPS_CAPS.len() - 1)] }
    pub fn master_gain(&self) -> f32 { if self.mute { 0.0 } else { self.master as f32 / VOLUME_STEPS as f32 } }
    pub fn music_gain(&self) -> f32 { self.music as f32 / VOLUME_STEPS as f32 }
    pub fn sfx_gain(&self) -> f32 { self.sfx as f32 / VOLUME_STEPS as f32 }

    pub fn from_text(t: &str) -> Self {
        let mut s = Settings::default();
        for l in t.lines() {
            let Some((k, v)) = l.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "fullscreen" => s.fullscreen = v == "1",
                "res" => s.res = pick(v, 1, RESOLUTIONS.len() - 1),
                "quality" => s.quality = pick(v, 0, QUALITY.len() - 1),
                "fps" => s.fps = v == "1",
                "fx" => s.fx = pick(v, 2, 2),
                "impact_lv" => s.impact = pick(v, 2, IMPACT.len() - 1),
                "keep_aspect" => s.keep_aspect = v != "0",
                "filter" => s.linear = v == "linear",
                "window" => s.window = parse_window(v),
                "vsync" => s.vsync = v != "0",
                "fps_cap" => s.fps_cap = pick(v, 0, FPS_CAPS.len() - 1),
                "master" => s.master = pick(v, 8, VOLUME_STEPS),
                "music" => s.music = pick(v, 10, VOLUME_STEPS),
                "sfx" => s.sfx = pick(v, 10, VOLUME_STEPS),
                "mute" => s.mute = v == "1",
                "difficulty" => s.difficulty = pick(v, 4, 8).max(1),
                "round_time" => s.round_time = pick(v, 0, ROUND_SECONDS.len() - 1),
                "rounds" => s.rounds = pick(v, 1, ROUNDS_TO_WIN.len() - 1),
                "life" => s.life = pick(v, 2, LIFE_PERCENT.len() - 1),
                "single_round" => s.single_round = v != "0",
                "classic_pushbox" => s.classic_pushbox = v != "0",
                "team_lifeshare" => s.team_life_share = v != "0",
                "gamespeed" => s.game_speed = pick(v, 2, GAME_SPEEDS.len() - 1),
                "survival_recovery" => s.survival_recovery = pick(v, 2, SURVIVAL_RECOVERY.len() - 1),
                "survival_best" => s.survival_best = v.parse().unwrap_or(0),
                "lang" => s.lang = if v == "en" { crate::i18n::Lang::En } else { crate::i18n::Lang::Pt },
                _ => {}
            }
        }
        s
    }

    pub fn to_text(&self) -> String {
        format!(
            "version={}\nfullscreen={}\nres={}\nquality={}\nfps={}\nfx={}\nimpact_lv={}\nkeep_aspect={}\nfilter={}\nwindow={}x{}\nvsync={}\nfps_cap={}\nmaster={}\nmusic={}\nsfx={}\nmute={}\ndifficulty={}\nround_time={}\nrounds={}\nlife={}\nsingle_round={}\nclassic_pushbox={}\nteam_lifeshare={}\ngamespeed={}\nsurvival_recovery={}\nsurvival_best={}\nlang={}\n",
            FORMAT_VERSION, self.fullscreen as i32, self.res, self.quality, self.fps as i32, self.fx, self.impact, self.keep_aspect as i32, if self.linear { "linear" } else { "nearest" }, self.window.0, self.window.1, self.vsync as i32, self.fps_cap, self.master, self.music, self.sfx, self.mute as i32,
            self.difficulty, self.round_time, self.rounds, self.life, self.single_round as i32, self.classic_pushbox as i32, self.team_life_share as i32, self.game_speed, self.survival_recovery, self.survival_best, if self.lang == crate::i18n::Lang::En { "en" } else { "pt" },
        )
    }

    pub fn load() -> Self { crate::storage::read(FILE).map(|t| Self::from_text(&t)).unwrap_or_default() }
    pub fn save(&self) { crate::storage::write(FILE, &self.to_text()); }
    pub fn apply_runtime(&self) {
        crate::i18n::set(self.lang);
        set_vsync(self.vsync);
        set_frame_cap(self.fps_cap_value());
    }
    /// Physical (pixel) height of the drawable area.
    pub fn physical_h() -> f32 { screen_height() * dpi_scale() }
    pub fn window_size(&self) -> (u32, u32) {
        if self.window.0 >= MIN_WINDOW.0 && self.window.1 >= MIN_WINDOW.1 { return self.window; }
        let (w, h, _) = RESOLUTIONS[self.res.min(RESOLUTIONS.len() - 1)];
        (w, h)
    }
    pub fn apply_window(&self) {
        set_fullscreen(self.fullscreen);
        #[cfg(not(target_arch = "wasm32"))]
        if !self.fullscreen { let (w, h) = self.window_size(); { let d = dpi_scale().max(0.5); request_new_screen_size(w as f32 / d, h as f32 / d) }; }
    }
}

pub const MIN_WINDOW: (u32, u32) = (320, 180);

fn parse_window(v: &str) -> (u32, u32) {
    v.split_once('x').and_then(|(w, h)| Some((w.trim().parse().ok()?, h.trim().parse().ok()?))).unwrap_or((0, 0))
}

#[derive(Default)]
pub struct WindowWatch { seen: (u32, u32), stable: f32 }

impl WindowWatch {
    const SETTLE_SECONDS: f32 = 0.8;

    pub fn observe(&mut self, size: (u32, u32), dt: f32, fullscreen: bool, saved: (u32, u32)) -> Option<(u32, u32)> {
        if fullscreen || size.0 < MIN_WINDOW.0 || size.1 < MIN_WINDOW.1 { self.seen = (0, 0); return None; }
        if size != self.seen { self.seen = size; self.stable = 0.0; return None; }
        self.stable += dt;
        (self.stable >= Self::SETTLE_SECONDS && size != saved).then_some(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for _ in 0..10 { assert_eq!(w.observe((1920, 1080), 0.5, true, (0, 0)), None); }
        for _ in 0..10 { assert_eq!(w.observe((0, 0), 0.5, false, (0, 0)), None); }
    }

    #[test]
    fn version_one_files_load_with_new_defaults() {
        let s = Settings::from_text("fullscreen=1\nres=3\nquality=2\nfps=1\nfx=1\nimpact_lv=0\n");
        assert!(s.fullscreen && s.fps);
        assert_eq!((s.res, s.quality, s.fx, s.impact), (3, 2, 1, 0));
        assert_eq!((s.vsync, s.master, s.rounds, s.lang), (true, 8, 1, crate::i18n::Lang::Pt));
    }

    #[test]
    fn text_round_trips() {
        let s = Settings { vsync: false, fps_cap: 3, master: 3, music: 0, sfx: 7, mute: true, difficulty: 7, round_time: 3, rounds: 2, life: 4, lang: crate::i18n::Lang::En, ..Settings::default() };
        assert_eq!(Settings::from_text(&s.to_text()), s);
    }

    #[test]
    fn garbage_is_clamped() {
        let s = Settings::from_text("master=99\nres=abc\nlife=42\ndifficulty=0\nrounds=-1\n");
        assert_eq!((s.master, s.res, s.life, s.difficulty, s.rounds), (VOLUME_STEPS, 1, LIFE_PERCENT.len() - 1, 1, 1));
    }

    #[test]
    fn rules_follow_the_chosen_options() {
        let r = Settings { round_time: 3, rounds: 2, life: 3, ..Settings::default() }.rules();
        assert_eq!((r.rounds_to_win, r.round_ticks, r.life_percent), (3, 0, 150));
        assert_eq!(Settings::default().rules(), Rules::default());
    }

    #[test]
    fn gameplay_options_reach_the_rules() {
        let r = Settings { single_round: true, classic_pushbox: false, team_life_share: false, game_speed: 4, ..Settings::default() }.rules();
        assert!(r.single_round && !r.classic_pushbox && !r.team_life_share);
        assert!((r.game_speed - GAME_SPEEDS[4]).abs() < 1e-6);
        let s = Settings { single_round: true, classic_pushbox: false, team_life_share: false, game_speed: 4, ..Settings::default() };
        assert_eq!(Settings::from_text(&s.to_text()), s);
    }

    #[test]
    fn mute_silences_master() {
        assert_eq!(Settings { mute: true, ..Settings::default() }.master_gain(), 0.0);
        assert!((Settings::default().master_gain() - 0.8).abs() < 1e-6);
    }
}
