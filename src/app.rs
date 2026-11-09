//! Screens and game flow.
use crate::audio::Audio;
use crate::controls::{nav, Controls};
use crate::gfx::*;
use crate::registry::Registry;
use crate::render::{DrawOpts, Gfx, MatchRenderer};
use crate::session::{Pilot, Session};
use crate::settings::{Rules, Settings, WindowWatch, FX_LEVELS, IMPACT, QUALITY, RESOLUTIONS};
use crate::stages::Stage;
use crate::ui::*;
use std::fmt::Write;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;

mod arcade;
mod cutscene;
mod fight_seed;
mod keyedit;
mod loading;
mod pause;
mod market;
mod market_text;
mod menu_fighters;
mod mugen_catalog;
mod mugen_gallery;
mod replay_browser;
mod settings_screen;
mod story;
mod story_screen;
mod survival;
mod update_banner;
mod versus;
#[cfg(target_arch = "wasm32")]
mod web_import;
mod controls_screen;
pub(crate) use keyedit::KeyEdit;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Arcade,
    Story,
    Survival,
    Cpu,
    Versus,
    Training,
    Watch,
    TeamSeq,
    TeamSimul,
    TeamTag,
}
impl Mode {
    fn label(self) -> &'static str {
        crate::i18n::t(match self {
            Mode::Arcade => "mode.arcade",
            Mode::Story => return crate::i18n::tr("HISTÓRIA", "STORY"),
            Mode::Survival => "mode.survival",
            Mode::Cpu => "mode.cpu",
            Mode::Versus => "mode.versus",
            Mode::Training => "mode.training",
            Mode::Watch => "mode.watch",
            Mode::TeamSeq => "mode.team_seq",
            Mode::TeamSimul => "mode.team_simul",
            Mode::TeamTag => "mode.team_tag",
        })
    }
    /// Team size (1 for the regular modes).
    pub fn is_team(self) -> bool {
        matches!(self, Mode::TeamSeq | Mode::TeamSimul | Mode::TeamTag)
    }
    /// Stable key used by replay files.
    pub fn key(self) -> &'static str {
        match self {
            Mode::Arcade => "Arcade",
            Mode::Story => "Story",
            Mode::Survival => "Survival",
            Mode::Cpu => "Cpu",
            Mode::Versus => "Versus",
            Mode::Training => "Training",
            Mode::Watch => "Watch",
            Mode::TeamSeq => "TeamSeq",
            Mode::TeamSimul => "TeamSimul",
            Mode::TeamTag => "TeamTag",
        }
    }
    pub fn from_key(key: &str) -> Mode {
        match key {
            "Arcade" => Mode::Arcade,
            "Story" => Mode::Story,
            "Survival" => Mode::Survival,
            "Versus" => Mode::Versus,
            "Training" => Mode::Training,
            "Watch" => Mode::Watch,
            "TeamSeq" => Mode::TeamSeq,
            "TeamSimul" => Mode::TeamSimul,
            "TeamTag" => Mode::TeamTag,
            _ => Mode::Cpu,
        }
    }
}

pub enum Screen {
    Title,
    Menu {
        cur: usize,
    },
    Select {
        cur: [usize; 2],
        picking: usize,
        picked: [Option<usize>; 2],
    },
    Stages {
        cur: usize,
    },
    Vs {
        t: f32,
        step: u8,
    },
    Fight,
    Results {
        winner: usize,
        cur: usize,
    },
    Story {
        t: f32,
    },
    /// Survival game over: final score and fights won
    SurvivalOver {
        score: i64,
        rounds: usize,
    },
    Controls,
    Mugen {
        view: u8,
        page: usize,
    },
    Market,
    /// on-disk replay browser
    Replays {
        cur: usize,
    },
    Settings {
        tab: usize,
        cur: usize,
        from_fight: bool,
    },
    /// story scene: 0 arcade opening · 1 before the boss · 2 ending
    Cutscene {
        kind: u8,
        page: usize,
        t: f32,
    },
}

/// Fight assets prepared step by step behind the VS screen.
#[derive(Default)]
pub struct Pending {
    defs: [Vec<Arc<CharDef>>; 2],
    stage: Option<Stage>,
    stage_job: Option<crate::registry::StageJob>,
    sess: Option<Session>,
    jobs: Vec<crate::registry::LoadJob>,
    warm: loading::Warm,
}

/// Everything needed to re-simulate a finished fight: same fighters, stage and mode, plus the recorded inputs per slot.
#[derive(Clone)]
pub struct Replay {
    sel: (usize, usize, usize),
    mode: Mode,
    team: Option<Team>,
    rec: Vec<Vec<u32>>,
    rules: Rules,
    seed: u32,
    life: [f32; 2],
    pal: [i32; 2],
    team_pal: Vec<i32>,
    net: Option<crate::replays::NetplayConfig>,
}

/// Team battle state: sequence (KOF-style turns, winner keeps its remaining life) or parallel (2v2 simultaneous).
#[derive(Clone)]
pub struct Team {
    pub members: [Vec<usize>; 2],
    pub idx: [usize; 2],
    pub life: [f32; 2],
    /** KOF XV: the team shares one super meter */
    pub power: [f32; 2],
    pub ko: [Vec<bool>; 2],
    pub simul: bool,
    pub tag: bool,
}
impl Team {
    pub fn new(a: Vec<usize>, b: Vec<usize>, simul: bool) -> Self {
        let (la, lb) = (a.len(), b.len());
        Team {
            members: [a, b],
            idx: [0, 0],
            life: [1.0, 1.0],
            power: [0.0, 0.0],
            ko: [vec![false; la], vec![false; lb]],
            simul,
            tag: false,
        }
    }
    pub fn tagged(a: Vec<usize>, b: Vec<usize>) -> Self {
        Team {
            tag: true,
            ..Team::new(a, b, false)
        }
    }
    pub fn whole_squad(&self) -> bool {
        self.simul || self.tag
    }
}
const VIEW_ROWS: usize = 4;
const SELECT_ROWS: usize = 7;
const NUMS: [&str; 5] = ["1", "2", "3", "4", "5"];

fn opaque_box(d: &CharDef, key: (i32, i32), s: &ume_engine::chardef::Sprite) -> (f32, f32, f32, f32) {
    let dims = s.image.as_ref().map(|im| (im.w as usize, im.h as usize)).or_else(|| d.indexed.get(&key).map(|ix| (ix.w as usize, ix.h as usize)));
    let Some((iw, ih)) = dims else { return (0.0, 0.0, s.width, s.height) };
    if iw == 0 || ih == 0 { return (0.0, 0.0, s.width, s.height); }
    let visible = |x: usize, y: usize| match (&s.image, d.indexed.get(&key)) {
        (Some(im), _) => im.rgba[(y * iw + x) * 4 + 3] > 16,
        (None, Some(ix)) => ix.pixels.get(y * iw + x).is_some_and(|&p| p != 0),
        _ => false,
    };
    let (mut x0, mut y0, mut x1, mut y1) = (iw, ih, 0, 0);
    for y in 0..ih {
        for x in 0..iw {
            if visible(x, y) { x0 = x0.min(x); x1 = x1.max(x + 1); y0 = y0.min(y); y1 = y1.max(y + 1); }
        }
    }
    if x0 >= x1 || y0 >= y1 {
        return (0.0, 0.0, s.width, s.height);
    }
    let (kx, ky) = (s.width / iw as f32, s.height / ih as f32);
    (
        x0 as f32 * kx,
        y0 as f32 * ky,
        (x1 - x0) as f32 * kx,
        (y1 - y0) as f32 * ky,
    )
}

fn clip(s: &str, n: usize) -> &str {
    s.char_indices().nth(n).map_or(s, |(i, _)| &s[..i])
}

pub struct App {
    pub reg: Registry,
    pub market: crate::market::Market,
    pub(crate) market_ui: market::MarketUi,
    pub ui: Ui,
    pub g: Gfx,
    pub c: Controls,
    pub audio: Audio,
    pub screen: Screen,
    pub mode: Mode,
    pub ai_level: i32,
    pub sel: (usize, usize, usize),
    pub session: Option<Session>,
    pub arcade: Option<(Vec<usize>, usize)>,
    pub boss_progression: crate::boss::BossProgression,
    pause: pause::PauseState,
    pub debug_boxes: bool,
    pub quit: bool,
    pub msg: String,
    /// seconds left before the last `msg` disappears on its own (0 = no toast)
    msg_ttl: f32,
    msg_seen: String,
    preview_t: f32,
    pub settings: Settings,
    pub(crate) key_edit: Option<keyedit::KeyEdit>,
    pending: Pending,
    import_jobs: Vec<std::thread::JoinHandle<Result<(String, ume_formats::vfs::Vfs), String>>>,
    import_progress: Option<ume_formats::vfs::ZipProgress>,
    #[cfg(target_arch = "wasm32")]
    wasm_import: Option<web_import::WasmImport>,
    pub team: Option<Team>,
    /// Survival run in progress (round, score, carried life)
    survival: Option<survival::Survival>,
    story: Option<crate::story::StoryRun>,
    /// Arcade/Survival with a second human on the same side (couch co-op)
    pub coop: bool,
    /// Screenpack loaded from mugen/data or mugen/screenpacks (menu labels, roster order, storyboards, lifebar geometry)
    sp: Option<crate::screenpack::ScreenpackUi>,
    /// Palette chosen per side (1..=PALETTE_COUNT) and per team member in pick order
    pub pal: [i32; 2],
    pub pal_pick: i32,
    pub team_pal: Vec<i32>,
    /// last finished fight (replayable) and the replay being played now
    last_fight: Option<Replay>,
    pub replaying: Option<Replay>,
    /// web deep link waiting for the imported packs to land in the registry
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    deep: Option<(Option<String>, Option<String>, Option<usize>, Mode, bool, Option<String>, f32)>,
    /// fighters per team in the team modes (2..=5)
    pub team_n: usize,
    pub team_pick: Vec<usize>,
    /// select-screen state: filter (0 all · 1 originals · 2 MUGEN), first visible row, cursor dwell timer
    sel_filter: u8,
    pub ending_shown: bool,
    /// colour multiplier for `draw_char` (cutscenes dim the character who is not speaking)
    char_tint: Color,
    sel_row: usize,
    sel_last: [usize; 2],
    sel_dwell: f32,
    sel_buf: String,
    names_up: Vec<String>,
    fit: std::collections::HashMap<String, [f32; 4]>,
    sel_cur: [(f32, f32); 2],
    sel_flash: (usize, f32),
    stage_row: usize,
    quality_check: f32,
    window_watch: WindowWatch,
    window_pinned: bool,
    updater: crate::update::Updater,
    pub trans: f32,
    screen_t: f32,
    menu_art: crate::menu_art::MenuArt,
    last_disc: Option<std::mem::Discriminant<Screen>>,
    applied: Option<Settings>,
    settings_back: Option<(usize, bool)>,
    fight_rules: Rules,
    /// replay browser state
    replay_list: Vec<crate::replays::Entry>,
    replay_problems: Vec<String>,
    replay_rename: Option<String>,
    replay_confirm: bool,
    replay_by_name: bool,
    /// a desync report was already written for the current online match
    desync_reported: bool,
    /// Roster signature of the last sprite warm-up, so a rematch keeps the atlases it already filled.
    warm_key: String,
}

const MENU: [(&str, &str); 16] = [
    ("menu.arcade", "desc.arcade"),
    ("menu.story", "desc.story"),
    ("menu.survival", "desc.survival"),
    ("menu.cpu", "desc.cpu"),
    ("menu.versus", "desc.versus"),
    ("menu.training", "desc.training"),
    ("menu.watch", "desc.watch"),
    ("menu.team_seq", "desc.team_seq"),
    ("menu.team_simul", "desc.team_simul"),
    ("menu.team_tag", "desc.team_tag"),
    ("menu.mugen", "desc.mugen"),
    ("menu.market", "desc.market"),
    ("menu.controls", "desc.controls"),
    ("menu.settings", "desc.settings"),
    ("menu.replays", "desc.replays"),
    ("menu.quit", "desc.quit"),
];

impl App {
    /// One splash frame of the boot loading screen.
    async fn boot(ui: &mut Ui, p: f32, label: &str) {
        ui.begin();
        ui.loading_screen(p, label);
        next_frame().await;
    }

    pub async fn new(touch: bool, audio_on: bool) -> Self {
        let mut ui = Ui::new();
        Self::boot(&mut ui, 0.05, "Iniciando…").await;
        let mut settings = Settings::load();
        if touch {
            settings.fx = settings.fx.min(1);
        } // phones: no bloom passes
        ume_engine::native::rig::set_art_res(settings.art_res(Settings::physical_h()));
        Self::boot(&mut ui, 0.15, "Lutadores originais…").await;
        #[cfg(not(target_arch = "wasm32"))]
        let mut reg = {
            let progress = Arc::new(crate::registry::ScanProgress::default());
            let worker = {
                let progress = progress.clone();
                std::thread::Builder::new().name("registry-scan".into()).spawn(move || {
                    let mut reg = Registry::new();
                    reg.add_bundled_mugen();
                    reg.scan_folders_with(&progress);
                    reg.dedupe();
                    reg.sort_mugen();
                    reg
                })
            };
            match worker {
                Ok(worker) => {
                    // pump the boot screen while the folder index and the roster are built off-thread:
                    // a M.U.G.E.N collection of a few thousand files used to freeze the window here
                    let started = get_time();
                    while !worker.is_finished() {
                        let (files, chars, stages) = progress.counts();
                        let secs = (get_time() - started) as f32;
                        let p = 0.2 + 0.55 * (1.0 - (-secs * 0.12).exp());
                        Self::boot(
                            &mut ui,
                            p,
                            &crate::i18n::fill(
                                crate::i18n::tr3("Catalogando MUGEN: {f} arquivos · {c} lutadores · {s} cenários · {t}s", "Cataloguing MUGEN: {f} files · {c} fighters · {s} stages · {t}s", "Catalogando MUGEN: {f} archivos · {c} luchadores · {s} escenarios · {t}s"),
                                &[("f", &files.to_string()), ("c", &chars.to_string()), ("s", &stages.to_string()), ("t", &format!("{secs:.1}"))],
                            ),
                        )
                        .await;
                    }
                    worker.join().unwrap_or_else(|_| {
                        let mut reg = Registry::new();
                        reg.add_bundled_mugen();
                        reg
                    })
                }
                Err(_) => {
                    let mut reg = Registry::new();
                    reg.add_bundled_mugen();
                    reg.scan_folders();
                    reg.dedupe();
                    reg.sort_mugen();
                    reg
                }
            }
        };
        #[cfg(target_arch = "wasm32")]
        let mut reg = {
            Self::boot(&mut ui, 0.35, "Pacote Elecbyte (Kung Fu Man)…").await;
            let mut reg = Registry::new();
            reg.add_bundled_mugen();
            reg.dedupe();
            reg.sort_mugen();
            reg
        };
        let import_msg = if reg.import_errors.is_empty() {
            String::new()
        } else {
            let first: Vec<&str> = reg
                .import_errors
                .iter()
                .take(3)
                .map(|s| s.as_str())
                .collect();
            format!(
                "{} erro(s) ao ler mugen/: {}",
                reg.import_errors.len(),
                first.join(" | ")
            )
        };
        Self::boot(&mut ui, 0.9, crate::i18n::tr3("Áudio…", "Audio…", "Audio…")).await;
        let audio = Audio::new(audio_on).await;
        Self::boot(&mut ui, 1.0, crate::i18n::tr3("Pronto!", "Ready!", "¡Listo!")).await;
        App {
            reg,
            market: crate::market::Market::open(),
            market_ui: market::MarketUi::default(),
            ui,
            g: Gfx::new(),
            c: Controls::new(touch),
            audio,
            screen: Screen::Title,
            mode: Mode::Cpu,
            ai_level: settings.difficulty,
            sel: (0, 1, 0),
            session: None,
            arcade: None,
            boss_progression: crate::boss::BossProgression::load(),
            pause: pause::PauseState::new(),
            debug_boxes: false,
            quit: false,
            msg: import_msg,
            msg_ttl: 0.0,
            msg_seen: String::new(),
            preview_t: 0.0,
            deep: None,
            settings,
            key_edit: None,
            pending: Pending::default(),
            import_jobs: vec![],
            import_progress: None,
            #[cfg(target_arch = "wasm32")]
            wasm_import: None,
            window_watch: WindowWatch::default(),
            window_pinned: false,
            team: None,
            last_fight: None,
            replaying: None,
            team_n: 3,
            team_pick: vec![],
            sel_filter: 0,
            ending_shown: false,
            char_tint: WHITE,
            sel_row: 0,
            sel_last: [usize::MAX; 2],
            sel_dwell: 0.0,
            sel_buf: String::new(),
            names_up: vec![],
            fit: Default::default(),
            sel_cur: [(-1.0, 0.0); 2],
            sel_flash: (0, 0.0),
            stage_row: 0,
            quality_check: 0.0,
            trans: 0.0,
            screen_t: 0.0,
            menu_art: crate::menu_art::MenuArt::new(),
            last_disc: None,
            applied: None,
            settings_back: None,
            fight_rules: Rules::default(),
            replay_list: vec![],
            replay_problems: vec![],
            replay_rename: None,
            replay_confirm: false,
            replay_by_name: false,
            desync_reported: false,
            warm_key: String::new(),
            survival: None,
            story: None,
            coop: false,
            sp: crate::screenpack::probe(),
            pal: [settings.pal1 as i32, settings.pal2 as i32],
            pal_pick: 1,
            team_pal: vec![],
            updater: crate::update::Updater::start(&std::env::args().collect::<Vec<_>>()),
        }
    }

    /// Re-rasterize art when the effective quality changes (resolution switch, fullscreen, setting change).
    fn apply_quality(&mut self) {
        let r = self.settings.art_res(Settings::physical_h());
        if (r - ume_engine::native::rig::art_res()).abs() > 0.01 {
            ume_engine::native::rig::set_art_res(r);
            self.reg.unload_all();
            self.g.trim_next_frame();
        }
    }
    fn remember_window_size(&mut self, dt: f32) {
        if self.window_pinned
            || cfg!(any(
                target_arch = "wasm32",
                target_os = "android",
                target_os = "ios"
            ))
        {
            return;
        }
        let (w, h) = window_size();
        if let Some(size) = self.window_watch.observe(
            (w as u32, h as u32),
            dt,
            self.settings.fullscreen,
            self.settings.window,
        ) {
            self.settings.window = size;
            self.settings.save();
        }
    }
    fn toggle_fullscreen(&mut self) {
        self.settings.fullscreen = !self.settings.fullscreen;
        self.settings.apply_window();
        self.settings.save();
    }

    fn clicked_rect(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }
        let p = self.ui.to_virtual(mouse_position().into());
        p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h
    }
    fn hover_rect(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let p = self.ui.to_virtual(mouse_position().into());
        p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h
    }

    pub async fn frame(&mut self) {
        self.g.begin_frame();
        self.ui.begin();
        if self.ai_level != self.settings.difficulty {
            self.settings.difficulty = self.ai_level;
            self.settings.save();
        }
        if self.applied != Some(self.settings) {
            self.settings.apply_runtime();
            self.audio.set_levels(
                self.settings.master_gain(),
                self.settings.music_gain(),
                self.settings.sfx_gain(),
            );
            self.applied = Some(self.settings);
        }
        self.g.post.level = crate::vfx::quality::set(self.settings.fx);
        set_keep_aspect(self.settings.keep_aspect);
        set_linear_filter(self.settings.linear);
        self.g.fx_gain = IMPACT[self.settings.impact.min(IMPACT.len() - 1)].0;
        self.c.update();
        self.preview_t += get_frame_time();
        let dt = get_frame_time();
        #[cfg(target_arch = "wasm32")]
        if self.wasm_import.is_some() {
            self.wasm_import_step(dt);
            if self.wasm_import.is_some() {
                self.draw_import_overlay();
                self.apply_update_if_ready();
                return;
            }
        }
        self.updater.poll();
        match &mut self.screen {
            Screen::Title => self.title(),
            Screen::Menu { .. } => self.menu(),
            Screen::Select { .. } => self.select(),
            Screen::Stages { .. } => self.stages(),
            Screen::Vs { .. } => self.vs(dt).await,
            Screen::Settings { .. } => self.settings_screen(),
            Screen::Fight => self.fight(dt).await,
            Screen::Results { .. } => self.results().await,
            Screen::Controls => self.controls_screen(),
            Screen::Mugen { .. } => self.mugen_screen(),
            Screen::Market => self.market_screen(),
            Screen::Story { .. } => self.story_screen(dt),
            Screen::SurvivalOver { .. } => self.survival_over_screen(),
            Screen::Replays { .. } => self.replays_screen(),
            Screen::Cutscene { .. } => self.cutscene(),
        }
        if is_key_pressed(KeyCode::F11) {
            self.toggle_fullscreen();
        }
        self.remember_window_size(dt);
        if is_key_pressed(KeyCode::F3) {
            self.settings.fps = !self.settings.fps;
            self.settings.save();
        }
        // auto art quality follows the display (checked outside fights so live sprites are never swapped)
        self.quality_check += dt;
        if self.quality_check > 0.7
            && matches!(
                self.screen,
                Screen::Menu { .. }
                    | Screen::Title
                    | Screen::Settings {
                        from_fight: false,
                        ..
                    }
                    | Screen::Select { .. }
                    | Screen::Stages { .. }
            )
        {
            self.quality_check = 0.0;
            self.apply_quality();
        }
        if self.msg != self.msg_seen {
            self.msg_seen.clone_from(&self.msg);
            self.msg_ttl = if self.msg.is_empty() { 0.0 } else { 6.0 };
        } else if !self.msg.is_empty() {
            self.msg_ttl -= dt;
            if self.msg_ttl <= 0.0 {
                self.msg.clear();
                self.msg_seen.clear();
            }
        }
        if self.settings.fps {
            let fps = get_fps();
            let col = if fps >= 58 {
                hex("#6dff9e")
            } else if fps >= 30 {
                hex("#ffdd55")
            } else {
                hex("#ff5555")
            };
            let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
            let ft_ms = dt * 1000.0;
            self.ui.text(
                &format!(
                    "{} FPS ({:.1} ms) · {}T · {}×{} · art {:.1}×",
                    fps,
                    ft_ms,
                    threads,
                    (screen_width() * dpi_scale()) as i32,
                    Settings::physical_h() as i32,
                    ume_engine::native::rig::art_res()
                ),
                10.0,
                20.0,
                16.0,
                col,
                Align::Left,
                false,
            );
            let st = stats();
            self.ui.text(&format!("{} v · {} dc · {} up/{} KB · atlas {:.0}%/{:.0}%", st.vertices, st.draw_calls, st.uploads, st.upload_kb, st.rgba_fill * 100.0, st.index_fill * 100.0), 10.0, 42.0, 14.0, col, Align::Left, false);
        }
        // screen transition: diagonal wipe whenever the screen changes (not into/out of the fight itself)
        let disc = std::mem::discriminant(&self.screen);
        if self.last_disc != Some(disc) {
            if self.last_disc.is_some() && !matches!(self.screen, Screen::Fight) {
                self.trans = 1.0;
            }
            self.last_disc = Some(disc);
            self.screen_t = 0.0;
        }
        self.screen_t += dt;
        self.menu_art.tick(dt);
        if self.trans > 0.0 {
            self.ui.wipe(self.trans);
            self.trans = (self.trans - dt * 2.6).max(0.0);
        }
        self.audio.process().await;
        self.market.poll();
        if self.market.has_registry_work()
            && matches!(
                self.screen,
                Screen::Title
                    | Screen::Menu { .. }
                    | Screen::Market
                    | Screen::Settings { .. }
                    | Screen::Controls
                    | Screen::Mugen { .. }
            )
            && self.market.apply_to(&mut self.reg)
        {
            self.after_registry_change();
        }
        crate::import_button::flush();
        #[cfg(not(target_arch = "wasm32"))]
        let incoming = crate::take_imports();
        #[cfg(not(target_arch = "wasm32"))]
        if !incoming.is_empty() {
            ume_shell::watchdog::label("import");
        }
        #[cfg(not(target_arch = "wasm32"))]
        for zip in incoming {
            let prog = ume_formats::vfs::ZipProgress::default();
            self.import_progress = Some(prog.clone());
            if let Ok(h) = std::thread::Builder::new()
                .name("zip-import".into())
                .spawn(move || crate::registry::read_zip_with_progress(&zip, Some(&prog)))
            {
                self.import_jobs.push(h);
                self.msg = crate::i18n::tr3("Importando recursos MUGEN…", "Importing MUGEN resources…", "Importando recursos MUGEN…").into();
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if self.wasm_import.is_none()
                && (crate::webimport::busy() || crate::webimport::queued_imports() > 0)
                && !matches!(
                    self.screen,
                    Screen::Fight
                        | Screen::Vs { .. }
                        | Screen::Results { .. }
                        | Screen::Cutscene { .. }
                )
            {
                ume_shell::watchdog::label("import");
                self.wasm_import = Some(web_import::WasmImport::new());
                self.msg = crate::i18n::tr3("Importando recursos MUGEN…", "Importing MUGEN resources…", "Importando recursos MUGEN…").into();
            }
            self.resolve_deep_link(dt);
        }
        if self.import_jobs.iter().any(|h| h.is_finished()) {
            ume_shell::watchdog::label("import");
            let (done, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.import_jobs)
                .into_iter()
                .partition(|h| h.is_finished());
            self.import_jobs = rest;
            for h in done {
                let r = h
                    .join()
                    .unwrap_or_else(|_| Err("import thread panicked".into()));
                self.finish_import(r);
            }
            if self.import_jobs.is_empty() {
                self.import_progress = None;
            }
        }
        if !self.import_jobs.is_empty() {
            let (done, total, cur_file) = if let Some(p) = &self.import_progress {
                let d = p.done.load(std::sync::atomic::Ordering::Relaxed);
                let t = p.total.load(std::sync::atomic::Ordering::Relaxed);
                let cur = p.current_name.lock().map(|s| s.clone()).unwrap_or_default();
                (d, t, cur)
            } else {
                (0, 0, String::new())
            };
            let ratio = if total > 0 { (done as f32 / total as f32).clamp(0.0, 1.0) } else { 0.15 + 0.15 * (self.ui.t * 3.0).sin().abs() };
            let banner_h = 68.0;
            self.ui.rect(0.0, VH - banner_h, VW, banner_h, Color::new(0.04, 0.03, 0.08, 0.94));
            self.ui.rect(0.0, VH - banner_h, VW, 2.0, hex("#00f0ff"));
            let label = if !cur_file.is_empty() {
                format!("{}: {} ({}/{})", crate::i18n::tr3("IMPORTANDO RECURSOS MUGEN", "IMPORTING MUGEN RESOURCES", "IMPORTANDO RECURSOS MUGEN"), cur_file, done, total)
            } else {
                format!("{} ({}/{})", crate::i18n::tr3("IMPORTANDO RECURSOS MUGEN", "IMPORTING MUGEN RESOURCES", "IMPORTANDO RECURSOS MUGEN"), done, total)
            };
            self.ui.progress_bar(80.0, VH - banner_h + 12.0, VW - 160.0, ratio, &label);
            let hw = format!("⚡ PIPELINE: {} {} · SIMD", ume_formats::par::workers(total.max(1)), crate::i18n::tr3("THREADS PARALELAS", "PARALLEL THREADS", "HILOS PARALELOS"));
            self.ui.text(&hw, VW - 80.0, VH - banner_h + 8.0, 13.0, hex("#00f0ff"), Align::Right, true);
        } else if !self.msg.is_empty() {
            self.ui
                .rect(0.0, VH - 36.0, VW, 36.0, Color::new(0.0, 0.0, 0.0, 0.7));
            self.ui.text(
                &self.msg,
                VW / 2.0,
                VH - 12.0,
                18.0,
                hex("#6dff9e"),
                Align::Center,
                false,
            );
            if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Left) {
                self.msg.clear();
            }
        }
        self.apply_update_if_ready();
    }

    fn after_registry_change(&mut self) {
        let (nc, ns) = (self.reg.chars.len().max(1), self.reg.stages.len().max(1));
        self.sel = (
            self.sel.0.min(nc - 1),
            self.sel.1.min(nc - 1),
            self.sel.2.min(ns - 1),
        );
        self.arcade = None;
        self.team_pick.clear();
        self.last_fight = None;
        self.replaying = None;
        self.sel_last = [usize::MAX; 2];
        self.sel_cur = [(-1.0, 0.0); 2];
        self.sel_row = 0;
        self.stage_row = 0;
    }
    fn menu_art_fade(&self) -> f32 {
        self.menu_art.fade_in()
    }

    /// Web deep link: `?p1=<lutador>&p2=<lutador>&cenario=<n>&modo=cpu|versus|treino|watch&luta=1`
    /// opens the stage pick or the fight itself (shareable links, QA and performance runs). Names are resolved once
    /// the imported packs reach the registry, so the link keeps working while the pack is still loading.
    #[cfg(target_arch = "wasm32")]
    pub fn apply_deep_link(&mut self) {
        let q = crate::query_params();
        let get = |k: &str| q.iter().find(|(a, _)| a == k).map(|(_, b)| b.clone());
        let (p1, p2) = (get("p1"), get("p2"));
        let screen = get("tela");
        if p1.is_none() && p2.is_none() && screen.is_none() { return; }
        let mode = match get("modo").as_deref() {
            Some("versus") => Mode::Versus,
            Some("treino") => Mode::Training,
            Some("watch") => Mode::Watch,
            _ => Mode::Cpu,
        };
        let stage = get("cenario").and_then(|v| v.parse::<usize>().ok());
        self.deep = Some((p1, p2, stage, mode, get("luta").is_some_and(|v| v != "0"), screen, 0.0));
    }

    #[cfg(target_arch = "wasm32")]
    fn resolve_deep_link(&mut self, dt: f32) {
        let Some(deep) = self.deep.as_mut() else { return };
        deep.6 += dt;
        let (p1, p2, stage, mode, fight, screen, waited) = (deep.0.clone(), deep.1.clone(), deep.2, deep.3, deep.4, deep.5.clone(), deep.6);
        let find = |needle: &str| {
            let n = needle.to_lowercase();
            self.reg.chars.iter().position(|c| {
                c.id.to_lowercase().contains(&n) || c.key.to_lowercase().contains(&n) || c.name.to_lowercase().contains(&n)
            })
        };
        let first = p1.as_deref().and_then(find).or_else(|| p2.as_deref().and_then(find));
        let screen_only = p1.is_none() && p2.is_none();
        if first.is_none() && !screen_only && waited < 20.0 { return; }
        let p1 = p1.as_deref().and_then(find).or(first);
        let p2 = p2.as_deref().and_then(find).or(first);
        if let Some(i) = p1 { self.sel.0 = i; }
        if let Some(i) = p2 { self.sel.1 = i; }
        if let Some(s) = stage.filter(|&s| s < self.reg.stages.len()) { self.sel.2 = s; }
        self.mode = mode;
        self.team = None;
        self.arcade = None;
        self.screen = match screen.as_deref() {
            Some("select") => { self.sel_filter = 2; Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] } }
            Some("stages") => Screen::Stages { cur: self.sel.2 },
            _ if fight => Screen::Vs { t: 0.0, step: 0 },
            _ => Screen::Stages { cur: self.sel.2 },
        };
        self.deep = None;
    }

    fn go(&mut self, s: Screen) { self.audio.sfx("confirm"); self.screen = s; }

    // ------------------------------------------------------------ title / menu
    fn title(&mut self) {
        self.menu_art.select(0);
        self.menu_art.draw(&self.ui);
        self.ui.logo(VW / 2.0, 330.0, 200.0);
        self.ui.text(
            crate::i18n::tr3("ENGINE DE LUTA · RUST · DESKTOP · WEB · MOBILE", "FIGHTING ENGINE · RUST · DESKTOP · WEB · MOBILE", "MOTOR DE LUCHA · RUST · ESCRITORIO · WEB · MÓVIL"),
            VW / 2.0,
            420.0,
            20.0,
            hex("#a49cc0"),
            Align::Center,
            true,
        );
        if (self.ui.t * 2.0) as i32 % 2 == 0 {
            self.ui.text_bold(
                crate::i18n::tr3("PRESSIONE START / ENTER", "PRESS START / ENTER", "PULSA START / ENTER"),
                VW / 2.0,
                540.0,
                30.0,
                WHITE,
                hex("#12051f"),
                Align::Center,
            );
        }
        self.ui.text(
            crate::i18n::tr3("Compatível com personagens M.U.G.E.N  ·  DualShock 4 / DualSense com vibração", "Compatible with M.U.G.E.N characters  ·  DualShock 4 / DualSense with rumble", "Compatible con personajes M.U.G.E.N  ·  DualShock 4 / DualSense con vibración"),
            VW / 2.0,
            690.0,
            18.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
        self.draw_update_status();
        if self.c.pressed(nav::OK | nav::START)
            || is_mouse_button_pressed(MouseButton::Left)
            || !touches().is_empty() && touches()[0].phase == TouchPhase::Started
        {
            self.go(Screen::Menu { cur: 0 });
        }
    }

    fn menu(&mut self) {
        let Screen::Menu { cur } = &mut self.screen else {
            return;
        };
        let mut cur_v = *cur;
        if self.c.pressed(nav::DOWN) {
            cur_v = (cur_v + 1) % MENU.len();
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::UP) {
            cur_v = (cur_v + MENU.len() - 1) % MENU.len();
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::LEFT) {
            self.ai_level = (self.ai_level - 1).max(1);
        }
        if self.c.pressed(nav::RIGHT) {
            self.ai_level = (self.ai_level + 1).min(8);
        }
        let mut chosen = if self.c.pressed(nav::OK | nav::START) {
            Some(cur_v)
        } else {
            None
        };
        self.menu_backdrop(cur_v);
        let ui = &self.ui;
        let accent = crate::menu_art::accent_of(cur_v);
        crate::menu_art::glass(ui, -400.0, 0.0, 400.0 + 600.0, VH, 1.0);
        ui.rect(
            -400.0,
            0.0,
            400.0 + 600.0,
            VH,
            Color::new(0.0, 0.0, 0.0, 0.18),
        );
        ui.rect(
            600.0,
            0.0,
            3.0,
            VH,
            Color::new(accent.r, accent.g, accent.b, 0.8),
        );
        ui.logo(250.0, 118.0, 74.0);
        let n = MENU.len();
        let step = 456.0 / n as f32;
        for (i, (key, _)) in MENU.iter().enumerate() {
            let label = self.menu_text(key);
            let k_in = ((self.screen_t - 0.05 - i as f32 * 0.04) / 0.28).clamp(0.0, 1.0);
            let k_in = 1.0 - (1.0 - k_in).powi(3);
            let (x, y, w, h) = (
                40.0 - (1.0 - k_in) * 120.0,
                196.0 + i as f32 * step,
                520.0,
                step - 4.0,
            );
            if self.hover_rect(x, y, w, h) && (mouse_delta_position().length() > 0.0) {
                cur_v = i;
            }
            if self.clicked_rect(x, y, w, h) {
                cur_v = i;
                chosen = Some(i);
            }
            let f = i == cur_v;
            let off = if f { 22.0 } else { 0.0 };
            if f {
                let pulse = 0.8 + 0.2 * (ui.t * 5.0).sin();
                ui.quad(
                    [
                        vec2(x + off + 18.0, y),
                        vec2(x + off + w, y),
                        vec2(x + off + w - 18.0, y + h),
                        vec2(x + off, y + h),
                    ],
                    Color::new(
                        accent.r * pulse,
                        accent.g * pulse,
                        accent.b * pulse,
                        0.92 * k_in,
                    ),
                );
                ui.quad(
                    [
                        vec2(x + off - 14.0, y),
                        vec2(x + off - 4.0, y),
                        vec2(x + off - 18.0, y + h),
                        vec2(x + off - 28.0, y + h),
                    ],
                    Color::new(1.0, 1.0, 1.0, k_in),
                );
                ui.text(
                    &label,
                    x + off + 30.0,
                    y + h * 0.72,
                    24.0,
                    hex("#12051f"),
                    Align::Left,
                    true,
                );
            } else {
                ui.quad(
                    [
                        vec2(x + 18.0, y),
                        vec2(x + w - 80.0, y),
                        vec2(x + w - 98.0, y + h),
                        vec2(x, y + h),
                    ],
                    Color::new(1.0, 1.0, 1.0, 0.07 * k_in),
                );
                ui.text(
                    &label,
                    x + 30.0,
                    y + h * 0.72,
                    22.0,
                    Color::new(1.0, 1.0, 1.0, 0.82 * k_in),
                    Align::Left,
                    true,
                );
            }
        }
        let title = self.menu_text(MENU[cur_v].0);
        let desc = crate::i18n::t(MENU[cur_v].1);
        let ta = ((self.menu_art_fade()) * 1.0).clamp(0.0, 1.0);
        ui.text_bold(
            &title.to_uppercase(),
            VW - 60.0,
            590.0,
            if title.chars().count() > 12 {
                44.0
            } else {
                64.0
            },
            Color::new(1.0, 1.0, 1.0, ta),
            Color::new(0.05, 0.02, 0.1, ta),
            Align::Right,
        );
        ui.rect(
            VW - 460.0,
            608.0,
            400.0,
            4.0,
            Color::new(accent.r, accent.g, accent.b, ta),
        );
        ui.text(
            desc,
            VW - 60.0,
            646.0,
            22.0,
            Color::new(1.0, 1.0, 1.0, 0.9 * ta),
            Align::Right,
            false,
        );
        let (nm, nn) = (
            self.reg.chars.iter().filter(|c| c.mugen).count(),
            self.playable().len(),
        );
        let stats = crate::i18n::fill(
            crate::i18n::tr3("{nn} lutadores · {st} cenários · {nm} MUGEN", "{nn} fighters · {st} stages · {nm} MUGEN", "{nn} luchadores · {st} escenarios · {nm} MUGEN"),
            &[("nn", &nn.to_string()), ("st", &self.reg.stages.len().to_string()), ("nm", &nm.to_string())],
        );
        ui.text(
            &stats,
            VW - 40.0,
            44.0,
            20.0,
            Color::new(1.0, 1.0, 1.0, 0.85),
            Align::Right,
            false,
        );
        if cur_v <= 8 {
            ui.text(
                crate::i18n::tr3("Dificuldade CPU (esq./dir.)", "CPU difficulty (left/right)", "Dificultad CPU (izq./der.)"),
                VW - 40.0,
                86.0,
                16.0,
                hex("#d8d0f0"),
                Align::Right,
                false,
            );
            for i in 0..8 {
                ui.rect(
                    VW - 40.0 - 8.0 * 26.0 + i as f32 * 26.0,
                    96.0,
                    20.0,
                    12.0,
                    if i < self.ai_level {
                        accent
                    } else {
                        Color::new(1.0, 1.0, 1.0, 0.18)
                    },
                );
            }
        }
        ui.text(
            &crate::update::build_label(),
            40.0,
            VH - 20.0,
            16.0,
            hex("#a49cc0"),
            Align::Left,
            false,
        );
        let pads = &self.c.pad_info;
        let pad_line = if pads.is_empty() {
            crate::i18n::tr3("teclado", "keyboard", "teclado").to_string()
        } else {
            format!("{} {}", pads.len(), crate::i18n::tr3("controle(s)", "controller(s)", "mando(s)"))
        };
        ui.text(
            &pad_line,
            560.0,
            VH - 20.0,
            16.0,
            hex("#a49cc0"),
            Align::Right,
            false,
        );
        self.screen = Screen::Menu { cur: cur_v };
        if self.c.pressed(nav::BACK) {
            self.screen = Screen::Title;
        }
        if let Some(i) = chosen {
            let mode = [
                Mode::Arcade,
                Mode::Story,
                Mode::Survival,
                Mode::Cpu,
                Mode::Versus,
                Mode::Training,
                Mode::Watch,
                Mode::TeamSeq,
                Mode::TeamSimul,
                Mode::TeamTag,
            ];
            match i {
                0..=9 => {
                    self.mode = mode[i];
                    self.team = None;
                    self.team_pick.clear();
                    if self.mode == Mode::TeamSimul {
                        self.team_n = 5;
                    } else if self.mode == Mode::TeamSeq {
                        self.team_n = 3;
                    } else if self.mode == Mode::TeamTag {
                        self.team_n = 2;
                    }
                    self.go(Screen::Select {
                        cur: [self.sel.0, self.sel.1],
                        picking: 0,
                        picked: [None, None],
                    });
                }
                10 => self.go(Screen::Mugen { view: 0, page: 0 }),
                11 => self.go(Screen::Market),
                12 => {
                    self.settings_back = None;
                    self.go(Screen::Controls)
                }
                13 => self.go(Screen::Settings {
                    tab: 0,
                    cur: 0,
                    from_fight: false,
                }),
                14 => {
                    self.refresh_replays();
                    self.go(Screen::Replays { cur: 0 })
                }
                _ => self.quit = true,
            }
        }
    }

    // ------------------------------------------------------------ select
    pub fn draw_char(
        &mut self,
        d: &Arc<CharDef>,
        anim: i32,
        t: f32,
        x: f32,
        y: f32,
        height_px: f32,
        mirror: bool,
    ) {
        let base_h = (d.c("size.height", 60.0) as f32 * 1.7).max(90.0);
        self.draw_char_k(d, anim, t, x, y, height_px / base_h, mirror);
    }

    fn draw_char_k(
        &mut self,
        d: &Arc<CharDef>,
        anim: i32,
        t: f32,
        x: f32,
        y: f32,
        kv: f32,
        mirror: bool,
    ) {
        let (form_g, anim) = if anim <= -1000 {
            (3, -anim - 1000)
        } else {
            (1, anim)
        };
        let Some(a) = d.anims.get(&anim).or(d.anims.get(&0)) else {
            return;
        };
        if a.frames.is_empty() {
            return;
        }
        let tot = a.total_time.max(1);
        let tm = ((t * 60.0) as i32) % tot;
        let (mut acc, mut idx) = (0, 0);
        for (i, f) in a.frames.iter().enumerate() {
            idx = i;
            if f.time < 0 || tm < acc + f.time {
                break;
            }
            acc += f.time;
        }
        let fr = &a.frames[idx];
        let key = if form_g > 1
            && fr.group == 1
            && d.sprites.contains_key(&(form_g, fr.index))
        {
            (form_g, fr.index)
        } else {
            (fr.group, fr.index)
        };
        let Some(spr) = d.sprites.get(&key) else { return };
        let Some(tex) = self.g.sprite_tex(d, key, &[]) else { return };
        let l = 320.0 / d.local_w;
        let k = kv * self.ui.s;
        let ks = k * spr.scale * l;
        let flip = mirror ^ fr.flip_h;
        let w = spr.width * ks;
        let h = spr.height * ks;
        let px = self.ui.x(x) + fr.x * l * k * if mirror { -1.0 } else { 1.0 };
        let py = self.ui.y(y) + fr.y * l * k;
        let left = if flip { px - (spr.width - spr.axis_x) * ks } else { px - spr.axis_x * ks };
        tex.draw(left, py - spr.axis_y * ks, self.char_tint, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_x: flip, ..Default::default() });
    }

    fn char_extent(&mut self, d: &Arc<CharDef>) -> [f32; 4] {
        if let Some(e) = self.fit.get(d.id.as_str()) {
            return *e;
        }
        let l = 320.0 / d.local_w;
        let mut e = [f32::MAX, f32::MIN, f32::MAX, f32::MIN];
        if let Some(a) = d.anims.get(&0) {
            for fr in &a.frames {
                let Some(s) = d.sprites.get(&(fr.group, fr.index)) else {
                    continue;
                };
                let sc = s.scale * l;
                let (ox, oy, ow, oh) = opaque_box(d, (fr.group, fr.index), s);
                let (top, left) = (fr.y * l + (oy - s.axis_y) * sc, if fr.flip_h { fr.x * l + (s.width - ow - ox - s.axis_x) * sc } else { fr.x * l + (ox - s.axis_x) * sc });
                e = [e[0].min(top), e[1].max(top + oh * sc), e[2].min(left), e[3].max(left + ow * sc)];
            }
        }
        if e[0] >= e[1] || e[2] >= e[3] {
            e = [-150.0, 0.0, -60.0, 60.0];
        }
        self.fit.insert(d.id.clone(), e);
        e
    }

    pub fn draw_char_fit(
        &mut self,
        d: &Arc<CharDef>,
        anim: i32,
        t: f32,
        cx: f32,
        bottom: f32,
        fw: f32,
        fh: f32,
        mirror: bool,
    ) {
        let e = self.char_extent(d);
        let k = (fh / (e[1] - e[0])).min(fw / (e[3] - e[2])).min(6.0);
        let sgn = if mirror { -1.0 } else { 1.0 };
        self.draw_char_k(
            d,
            anim,
            t,
            cx - sgn * (e[2] + e[3]) * 0.5 * k,
            bottom - e[1] * k,
            k,
            mirror,
        );
    }

    /// Registry indices visible under the current filter (never empty).
    fn char_view(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self
            .playable()
            .into_iter()
            .filter(|&i| match self.sel_filter {
                1 => !self.reg.chars[i].mugen,
                2 => self.reg.chars[i].mugen,
                _ => true,
            })
            .collect();
        if v.is_empty() {
            v = self.playable();
        }
        if let Some(sp) = &self.sp {
            v.sort_by_key(|&i| {
                let e = &self.reg.chars[i];
                sp.roster_pos(&e.id)
                    .or_else(|| sp.roster_pos(&e.name))
                    .unwrap_or(usize::MAX)
            });
        }
        v
    }

    /// Menu label honouring a loaded screenpack (`system.def` `menu.itemname.*`), else the i18n catalog.
    fn menu_text(&self, key: &'static str) -> String {
        let mugen = match key {
            "menu.arcade" => "arcade",
            "menu.survival" => "survival",
            "menu.team_seq" => "teamarcade",
            "menu.team_simul" => "teamversus",
            "menu.team_tag" => "teamcoop",
            "menu.training" => "training",
            "menu.watch" => "watch",
            "menu.quit" => "exit",
            _ => "",
        };
        match self
            .sp
            .as_ref()
            .filter(|_| !mugen.is_empty())
            .and_then(|s| s.menu_label(mugen))
        {
            Some(label) => label.to_string(),
            None => crate::i18n::t(key).to_string(),
        }
    }

    fn select(&mut self) {
        let Screen::Select {
            mut cur,
            mut picking,
            mut picked,
        } = self.screen
        else {
            return;
        };
        let dt = get_frame_time();
        let mut view = self.char_view();
        if self.sel_filter != 0 && view.len() == self.playable().len() {
            self.sel_filter = 0;
        }
        let m = view.len();
        let pos_of = |view: &[usize], i: usize| view.iter().position(|&v| v == i).unwrap_or(0);
        let cols = m.div_ceil(SELECT_ROWS).clamp(6, 10);
        let total_rows = m.div_ceil(cols);
        let two = self.mode == Mode::Versus;
        let coop_mode = self.coop && matches!(self.mode, Mode::Arcade | Mode::Survival);
        let mut confirm = false;
        let p = picking;
        let mut pos = pos_of(&view, cur[p]);
        let old = pos;
        if self.c.pressed(nav::RIGHT) {
            pos = (pos + 1) % m;
        }
        if self.c.pressed(nav::LEFT) {
            pos = (pos + m - 1) % m;
        }
        if self.c.pressed(nav::DOWN) {
            let (row, col) = (pos / cols, pos % cols);
            pos = if row + 1 < total_rows {
                ((row + 1) * cols + col).min(m - 1)
            } else {
                col
            };
        }
        if self.c.pressed(nav::UP) {
            let (row, col) = (pos / cols, pos % cols);
            pos = if row > 0 {
                pos - cols
            } else {
                ((total_rows - 1) * cols + col).min(m - 1)
            };
        }
        if is_key_pressed(KeyCode::PageDown) {
            pos = (pos + cols * SELECT_ROWS).min(m - 1);
        }
        if is_key_pressed(KeyCode::PageUp) {
            pos = pos.saturating_sub(cols * SELECT_ROWS);
        }
        if is_key_pressed(KeyCode::Home) {
            pos = 0;
        }
        if is_key_pressed(KeyCode::End) {
            pos = m - 1;
        }
        let wheel = mouse_wheel().1;
        if wheel > 0.0 {
            pos = pos.saturating_sub(cols);
        } else if wheel < 0.0 {
            pos = (pos + cols).min(m - 1);
        }
        if is_key_pressed(KeyCode::R) {
            pos = (get_time() * 1000.0) as usize % m;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.sel_filter = (self.sel_filter + 1) % 3;
            view = self.char_view();
            if view.len() == self.playable().len() {
                self.sel_filter = 0;
            }
            pos = pos_of(&view, cur[p]).min(view.len() - 1);
            self.sel_row = 0;
            self.audio.sfx("select");
            // re-enter with the new view next frame (geometry depends on it)
            cur[p] = view[pos];
            self.screen = Screen::Select {
                cur,
                picking,
                picked,
            };
            return;
        }
        if pos != old {
            cur[p] = view[pos];
            self.audio.sfx("select");
        }
        if matches!(self.mode, Mode::Arcade | Mode::Survival) && is_key_pressed(KeyCode::V) {
            self.coop = !self.coop;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::OK | nav::START) {
            confirm = true;
        }
        let back = self.c.pressed(nav::BACK);
        // keep the cursor row on screen
        let crow = pos / cols;
        if crow < self.sel_row {
            self.sel_row = crow;
        }
        if crow >= self.sel_row + SELECT_ROWS {
            self.sel_row = crow + 1 - SELECT_ROWS;
        }
        self.sel_row = self.sel_row.min(total_rows.saturating_sub(SELECT_ROWS));

        self.ui.background();
        if self.mode.is_team() {
            if is_key_pressed(KeyCode::E) {
                self.team_n = (self.team_n + 1).min(if self.mode == Mode::TeamTag { 3 } else { 5 });
                self.audio.sfx("select");
            }
            if is_key_pressed(KeyCode::Q) {
                self.team_n = (self.team_n - 1).max(2).max(self.team_pick.len() + 1);
                self.audio.sfx("select");
            }
        }
        if self.names_up.len() != self.reg.chars.len() {
            self.names_up = self
                .reg
                .chars
                .iter()
                .map(|c| c.name.to_uppercase())
                .collect();
        }
        let accents = [hex("#3fa9ff"), hex("#ff4d6d")];
        let active = [picking == 0 || picked[0].is_some(), picking == 1];
        let s_ui = self.ui.s;
        for side in 0..2 {
            let Some(i) = picked[side].or(if side == picking {
                Some(cur[side])
            } else {
                None
            }) else {
                continue;
            };
            let c = rgb(self.reg.chars[i].color);
            glow(
                self.ui.x(if side == 0 { 190.0 } else { VW - 190.0 }),
                self.ui.y(430.0),
                420.0 * s_ui,
                Color::new(c.r, c.g, c.b, 0.5),
            );
        }

        crate::menu_art::glass(&self.ui, 0.0, 0.0, VW, 84.0, 1.0);
        self.ui
            .rect(0.0, 82.0, VW, 2.0, alpha(accents[picking.min(1)], 0.85));
        let team_mode = self.mode.is_team();
        let pal_dir = if is_key_pressed(KeyCode::Period) {
            1
        } else if is_key_pressed(KeyCode::Comma) {
            -1
        } else {
            0
        };
        if pal_dir != 0 {
            if team_mode {
                self.pal_pick = cycle_palette(self.pal_pick, pal_dir);
            } else {
                self.pal[picking.min(1)] = cycle_palette(self.pal[picking.min(1)], pal_dir);
            }
            self.audio.sfx("select");
        }
        if is_key_pressed(KeyCode::P) {
            let p = 1
                + ((get_time() * 7919.0) as i32).rem_euclid(crate::settings::PALETTE_COUNT as i32);
            if team_mode {
                self.pal_pick = p;
            } else {
                self.pal[picking.min(1)] = p;
            }
            self.audio.sfx("select");
        }
        if pal_dir != 0 || is_key_pressed(KeyCode::P) {
            self.settings.pal1 = self.side_pal(0) as usize;
            self.settings.pal2 = self.side_pal(1) as usize;
            self.settings.save();
        }
        let pal_side0 = if team_mode {
            self.pal_pick
                .clamp(1, crate::settings::PALETTE_COUNT as i32)
        } else {
            self.pal_for_side(0, picked)
        };
        let pal_side1 = self.pal_for_side(1, picked);
        self.sel_buf.clear();
        if team_mode {
            let _ = write!(
                self.sel_buf,
                "{} {}v{}",
                self.mode.label(),
                self.team_n,
                self.team_n
            );
        } else {
            self.sel_buf.push_str(self.mode.label());
        }
        self.ui.text(
            &self.sel_buf,
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        let hint = if team_mode {
            crate::i18n::t("select.team_hint")
        } else if picking == 0 {
            crate::i18n::t("select.p1")
        } else if two {
            crate::i18n::t("select.p2")
        } else if coop_mode {
            crate::i18n::t("select.partner")
        } else {
            crate::i18n::t("select.opponent")
        };
        self.ui.text(
            hint,
            VW - 40.0,
            56.0,
            20.0,
            hex("#d8d0f0"),
            Align::Right,
            false,
        );

        let focus = cur[picking];
        if self.sel_last[picking] != focus {
            self.sel_last[picking] = focus;
            self.sel_dwell = 0.0;
        } else {
            self.sel_dwell += dt;
        }
        let mut want: Vec<(usize, i32)> =
            vec![(focus, if picking == 0 { pal_side0 } else { pal_side1 })];
        for side in 0..2 {
            if let Some(i) = picked[side] {
                want.push((i, if side == 0 { pal_side0 } else { pal_side1 }));
            }
        }
        ume_shell::watchdog::label("select:preview");
        self.reg.reap_streams();
        let mut loaded_one = false;
        for (i, pal) in want {
            if i == usize::MAX {
                continue;
            }
            let e = &self.reg.chars[i];
            if e.peek(pal).is_some() || e.broken.get() || !e.tile_art_ready() {
                continue;
            }
            if self.sel_dwell < if e.mugen { 0.18 } else { 0.03 } {
                continue;
            }
            e.stream_preview(pal);
            if e.errs_has(pal) {
                self.msg = format!("{} {}", crate::i18n::tr3("Falha ao carregar", "Failed to load", "Error al cargar"), e.name);
            }
            loaded_one = true;
        }

        let gap = 6.0;
        let cw = ((640.0 - (cols as f32 - 1.0) * gap) / cols as f32).min(96.0);
        let ch = cw * 1.17;
        let gw = cols as f32 * cw + (cols as f32 - 1.0) * gap;
        let gx = (VW - gw) / 2.0;
        let gy = 98.0;
        let gh = SELECT_ROWS as f32 * (ch + gap) - gap;
        crate::menu_art::glass(&self.ui, gx - 12.0, gy - 12.0, gw + 24.0, gh + 24.0, 1.0);
        self.ui.rect_lines(
            gx - 12.0,
            gy - 12.0,
            gw + 24.0,
            gh + 24.0,
            1.0,
            Color::new(1.0, 1.0, 1.0, 0.14),
        );
        let mut click = None;
        let first = self.sel_row * cols;
        let last = ((self.sel_row + SELECT_ROWS) * cols).min(m);
        for k in first..last {
            let i = view[k];
            let (cx, cy) = (
                gx + (k % cols) as f32 * (cw + gap),
                gy + ((k / cols) - self.sel_row) as f32 * (ch + gap),
            );
            if self.clicked_rect(cx, cy, cw, ch) {
                click = Some(i);
            }
            let (ecol, emugen, broken) = {
                let e = &self.reg.chars[i];
                (e.color, e.mugen, e.broken.get())
            };
            let col = rgb(ecol);
            self.ui
                .rect(cx, cy, cw, ch, Color::new(0.06, 0.04, 0.12, 0.92));
            self.ui.rect(cx, cy, cw, ch * 0.62, alpha(col, 0.5));
            self.ui
                .rect(cx, cy + ch * 0.62, cw, ch * 0.38, alpha(col, 0.18));
            let mut got = self.reg.chars[i].peek(1);
            let tile_thumb = if emugen { None } else { self.reg.chars[i].tile_portrait(1) };
            if got.is_none() && tile_thumb.is_none() && !emugen && !loaded_one && !broken && self.reg.chars[i].tile_art_ready() {
                got = self.reg.chars[i].load_preview(1).ok();
                loaded_one = got.is_some();
            }
            // MUGEN tiles use a thumbnail cached apart from the (size-capped) definition cache, decoded one per frame once the cursor rests
            if emugen
                && !loaded_one
                && !broken
                && self.sel_dwell > 0.15
                && !self.reg.chars[i].has_portrait_attempt()
            {
                self.reg.chars[i].stream_portrait();
                loaded_one = true;
            }
            let thumb = if emugen {
                self.reg.chars[i].portrait()
            } else {
                tile_thumb.or_else(|| {
                    got.as_ref()
                        .and_then(|d| d.sprites.get(&(9000, 1)).cloned())
                })
            };
            if thumb.is_some() || got.is_some() {
                let por = thumb.and_then(|p| p.image.clone().map(|im| (im, p.width, p.height)));
                if let Some((im, pw, ph)) = por {
                    let tex = self.g.texture(&im, true);
                    let kk = ((cw - 8.0) / pw).min((ch - 26.0) / ph);
                    draw_texture_ex(
                        &tex,
                        self.ui.x(cx + (cw - pw * kk) / 2.0),
                        self.ui.y(cy + 4.0),
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(pw * kk * self.ui.s, ph * kk * self.ui.s)),
                            ..Default::default()
                        },
                    );
                } else if let Some(d) = got {
                    self.ui_clip(cx, cy, cw, ch - 18.0, |app| {
                        app.draw_char_fit(
                            &d,
                            0,
                            0.0,
                            cx + cw / 2.0,
                            cy + ch - 20.0,
                            cw - 8.0,
                            ch - 30.0,
                            false,
                        )
                    });
                }
            } else {
                let init = self.names_up[i]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
                self.ui.text(
                    if broken {
                        "X"
                    } else {
                        &self.names_up[i][..init]
                    },
                    cx + cw / 2.0,
                    cy + ch * 0.58,
                    cw * 0.55,
                    Color::new(1.0, 1.0, 1.0, if broken { 0.5 } else { 0.28 }),
                    Align::Center,
                    true,
                );
            }
            self.ui
                .rect(cx, cy + ch - 18.0, cw, 18.0, Color::new(0.0, 0.0, 0.0, 0.7));
            let up = self.names_up[i].as_str();
            let nm = if emugen {
                clip(up, 13)
            } else {
                clip(up.split_whitespace().next().unwrap_or(""), 12)
            };
            self.ui.text(
                nm,
                cx + cw / 2.0,
                cy + ch - 4.0,
                (cw * 0.125).min(12.0),
                WHITE,
                Align::Center,
                true,
            );
            if emugen {
                self.ui
                    .rect(cx + cw - 44.0, cy + 3.0, 41.0, 13.0, hex("#22e8ff"));
                self.ui.text(
                    "MUGEN",
                    cx + cw - 23.5,
                    cy + 14.0,
                    10.0,
                    BLACK,
                    Align::Center,
                    true,
                );
            }
            let cursored = (cur[0] == i && active[0]) || (cur[1] == i && active[1]);
            if !cursored {
                self.ui
                    .rect(cx, cy, cw, ch, Color::new(0.0, 0.0, 0.08, 0.25));
            }
            for side in 0..2 {
                if picked[side] == Some(i) {
                    let tx = if side == 0 { cx } else { cx + cw - 24.0 };
                    self.ui.rect(tx, cy, 24.0, 14.0, accents[side]);
                    self.ui.text(
                        if side == 0 { "P1" } else { "P2" },
                        tx + 12.0,
                        cy + 12.0,
                        11.0,
                        BLACK,
                        Align::Center,
                        true,
                    );
                }
            }
            if let Some(n) = self.team_pick.iter().position(|&t| t == i) {
                self.ui.rect(cx, cy, 20.0, 18.0, hex("#ffcc33"));
                self.ui.text(
                    NUMS[n.min(4)],
                    cx + 10.0,
                    cy + 14.0,
                    14.0,
                    BLACK,
                    Align::Center,
                    true,
                );
            }
        }
        if total_rows > SELECT_ROWS {
            let bx = gx + gw + 3.0;
            self.ui
                .rect(bx, gy, 5.0, gh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (gh * SELECT_ROWS as f32 / total_rows as f32).max(18.0);
            self.ui.rect(
                bx,
                gy + (gh - th) * self.sel_row as f32 / (total_rows - SELECT_ROWS) as f32,
                5.0,
                th,
                hex("#ff4d6d"),
            );
        }
        let pulse = 0.5 + 0.5 * (self.ui.t * 7.0).sin();
        for side in 0..2 {
            if !active[side] {
                continue;
            }
            let kpos = pos_of(&view, cur[side]);
            if kpos < first || kpos >= last {
                continue;
            }
            let target = (
                gx + (kpos % cols) as f32 * (cw + gap),
                gy + ((kpos / cols) - self.sel_row) as f32 * (ch + gap),
            );
            let c = &mut self.sel_cur[side];
            if c.0 < 0.0 {
                *c = target;
            } else {
                let f = 1.0 - (-dt * 24.0).exp();
                c.0 += (target.0 - c.0) * f;
                c.1 += (target.1 - c.1) * f;
            }
            let (rx, ry) = (self.sel_cur[side].0, self.sel_cur[side].1);
            let a = accents[side];
            let pad = 2.0 + 2.0 * pulse + side as f32 * 3.0;
            self.ui.rect(
                rx - pad,
                ry - pad,
                cw + pad * 2.0,
                ch + pad * 2.0,
                alpha(a, 0.10 + 0.08 * pulse),
            );
            self.ui.rect_lines(
                rx - pad,
                ry - pad,
                cw + pad * 2.0,
                ch + pad * 2.0,
                2.0,
                alpha(a, 0.9),
            );
            let (bl, bt) = (16.0, 4.0);
            for (ox, oy, dx, dy) in [
                (rx - pad, ry - pad, 1.0, 1.0),
                (rx + cw + pad, ry - pad, -1.0, 1.0),
                (rx - pad, ry + ch + pad, 1.0, -1.0),
                (rx + cw + pad, ry + ch + pad, -1.0, -1.0),
            ] {
                self.ui.rect(
                    ox.min(ox + dx * bl),
                    oy + if dy > 0.0 { -2.0 } else { 2.0 - bt },
                    bl,
                    bt,
                    WHITE,
                );
                self.ui.rect(
                    ox + if dx > 0.0 { -2.0 } else { 2.0 - bt },
                    oy.min(oy + dy * bl),
                    bt,
                    bl,
                    WHITE,
                );
            }
            if picked[side].is_none() {
                let tag_w = if side == 0 { 26.0 } else if two { 26.0 } else { 32.0 };
                let tag_x = if side == 0 { rx - pad } else { rx + cw + pad - tag_w };
                self.ui.rect(tag_x, ry - pad - 15.0, tag_w, 15.0, a);
                self.ui.text(
                    if side == 0 {
                        "P1"
                    } else if two {
                        "P2"
                    } else {
                        "CPU"
                    },
                    tag_x + tag_w / 2.0,
                    ry - pad - 3.0,
                    12.0,
                    BLACK,
                    Align::Center,
                    true,
                );
            }
        }
        if let Some(i) = click {
            if cur[picking] == i {
                confirm = true;
            } else {
                cur[picking] = i;
                self.audio.sfx("select");
            }
        }

        for side in 0..2 {
            let idx = picked[side].or(if side == picking {
                Some(cur[side])
            } else {
                None
            });
            let label = if side == 0 {
                "P1"
            } else if two {
                "P2"
            } else {
                "CPU"
            };
            let pal = if side == 0 { pal_side0 } else { pal_side1 };
            self.select_side(
                side,
                idx,
                picked[side].is_some(),
                side == picking,
                pal,
                label,
            );
            if idx.is_some() {
                let (px, py) = (if side == 0 { 24.0 } else { VW - 304.0 }, 100.0);
                let y = py + 38.0;
                self.ui
                    .rect(px + 12.0, y, 256.0, 46.0, Color::new(0.0, 0.0, 0.0, 0.42));
                self.ui.text(
                    &format!(
                        "{} {pal}/{} · {}",
                        crate::i18n::t("select.palette"),
                        crate::settings::PALETTE_COUNT,
                        crate::i18n::t("select.palette_hint")
                    ),
                    px + 20.0,
                    y + 16.0,
                    12.0,
                    hex("#d8d0f0"),
                    Align::Left,
                    true,
                );
                for n in 1..=crate::settings::PALETTE_COUNT as i32 {
                    let bx = px + 14.0 + (n - 1) as f32 * 20.0;
                    let by = y + 22.0;
                    let on = n == pal;
                    if self.clicked_rect(bx, by, 18.0, 20.0) {
                        if team_mode {
                            self.pal_pick = n;
                        } else {
                            self.pal[side.min(1)] = n;
                        }
                        self.settings.pal1 = self.side_pal(0) as usize;
                        self.settings.pal2 = self.side_pal(1) as usize;
                        self.settings.save();
                    }
                    self.ui.rect(
                        bx,
                        by,
                        18.0,
                        20.0,
                        if on {
                            hex("#ffcc33")
                        } else {
                            Color::new(1.0, 1.0, 1.0, 0.12)
                        },
                    );
                    self.ui.text(
                        &n.to_string(),
                        bx + 9.0,
                        by + 15.0,
                        11.0,
                        if on { BLACK } else { WHITE },
                        Align::Center,
                        true,
                    );
                }
            }
        }
        self.sel_flash.1 = (self.sel_flash.1 - dt * 3.0).max(0.0);

        if team_mode {
            let n = self.team_n;
            let sw = ((640.0 - (n as f32 - 1.0) * 6.0) / n as f32).min(118.0);
            let x0 = (VW - (n as f32 * sw + (n as f32 - 1.0) * 6.0)) / 2.0;
            for k in 0..n {
                let x = x0 + k as f32 * (sw + 6.0);
                let y = 598.0;
                crate::menu_art::glass(&self.ui, x, y, sw, 46.0, 1.0);
                match self.team_pick.get(k).copied() {
                    Some(i) => {
                        self.ui
                            .rect(x, y, sw, 46.0, alpha(rgb(self.reg.chars[i].color), 0.5));
                        self.portrait_fit(i, x + 2.0, y + 2.0, 42.0, 42.0);
                        self.ui.text(
                            clip(self.names_up[i].split_whitespace().next().unwrap_or(""), 7),
                            x + 46.0,
                            y + 28.0,
                            if n > 4 { 11.0 } else { 13.0 },
                            WHITE,
                            Align::Left,
                            true,
                        );
                    }
                    None => {
                        let now = k == self.team_pick.len();
                        if now {
                            self.ui.rect(
                                x,
                                y,
                                sw,
                                46.0,
                                Color::new(1.0, 0.8, 0.2, 0.14 + 0.12 * pulse),
                            );
                            self.ui.rect_lines(x, y, sw, 46.0, 2.0, hex("#ffcc33"));
                        }
                        self.ui.text(
                            NUMS[k.min(4)],
                            x + sw / 2.0,
                            y + 31.0,
                            20.0,
                            Color::new(1.0, 1.0, 1.0, if now { 0.9 } else { 0.3 }),
                            Align::Center,
                            true,
                        );
                    }
                }
            }
        }

        crate::menu_art::glass(&self.ui, 0.0, 652.0, VW, 68.0, 1.0);
        if matches!(self.mode, Mode::Arcade | Mode::Survival) {
            let (bx, by, bw, bh) = (990.0, 664.0, 250.0, 42.0);
            let on = self.coop;
            if self.clicked_rect(bx, by, bw, bh) {
                self.coop = !self.coop;
                self.audio.sfx("select");
            }
            self.ui.rect(
                bx,
                by,
                bw,
                bh,
                if on {
                    alpha(hex("#3fa9ff"), 0.65)
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.08)
                },
            );
            self.ui.text(
                if on {
                    crate::i18n::t("select.coop_on")
                } else {
                    crate::i18n::t("select.coop_off")
                },
                bx + bw / 2.0,
                by + 27.0,
                15.0,
                WHITE,
                Align::Center,
                true,
            );
        }
        self.ui
            .rect(0.0, 652.0, VW, 1.5, alpha(accents[picking.min(1)], 0.7));
        let desc = &self.reg.chars[cur[picking]].description;
        self.ui.text(
            desc,
            VW / 2.0,
            680.0,
            18.0,
            hex("#d8d0f0"),
            Align::Center,
            false,
        );
        let flabel = [
            crate::i18n::t("select.filter_all"),
            crate::i18n::t("select.filter_originals"),
            crate::i18n::t("select.filter_mugen"),
        ][self.sel_filter as usize % 3];
        self.sel_buf.clear();
        let _ = write!(
            self.sel_buf,
            "[Tab] {flabel} · {}/{m} · [R] {} · PgUp/PgDn",
            pos + 1,
            crate::i18n::t("select.random").to_lowercase()
        );
        self.ui.text(
            &self.sel_buf,
            VW / 2.0,
            706.0,
            14.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
        // bound memory with hundreds of fighters
        let mut keep: Vec<usize> = cur.to_vec();
        keep.extend(picked.iter().flatten());
        self.reg.trim_cache(&keep);
        if confirm {
            if self.reg.chars[cur[picking]].broken.get() {
                self.msg = crate::i18n::tr3("Este personagem não pôde ser carregado.", "This character could not be loaded.", "No se pudo cargar este personaje.").into();
                self.screen = Screen::Select {
                    cur,
                    picking,
                    picked,
                };
                return;
            }
            self.audio.sfx("confirm");
            self.sel_flash = (picking, 1.0);
            if self.mode.is_team() {
                let me = cur[0];
                if self.team_pick.contains(&me) {
                    self.msg = crate::i18n::tr3("Este lutador já está na equipe.", "This fighter is already on the team.", "Este luchador ya está en el equipo.").into();
                    self.screen = Screen::Select {
                        cur,
                        picking: 0,
                        picked: [None, None],
                    };
                    return;
                }
                self.team_pick.push(me);
                self.team_pal.push(
                    self.pal_pick
                        .clamp(1, crate::settings::PALETTE_COUNT as i32),
                );
                let n = self.team_n;
                if self.team_pick.len() < n {
                    self.screen = Screen::Select {
                        cur,
                        picking: 0,
                        picked: [None, None],
                    };
                    return;
                }
                // opponents: random distinct fighters (originals first), then straight to the stage
                let mut pool: Vec<usize> = (0..self.reg.chars.len())
                    .filter(|i| {
                        !self.team_pick.contains(i)
                            && !self.reg.chars[*i].broken.get()
                            && !self.reg.chars[*i].boss
                    })
                    .collect();
                let seed = (get_time() * 1000.0) as usize;
                if !pool.is_empty() {
                    let l = pool.len();
                    pool.rotate_left(seed % l);
                }
                pool.sort_by_key(|&i| self.reg.chars[i].mugen as u8);
                let mut foes: Vec<usize> = pool.into_iter().take(n).collect();
                while foes.len() < n {
                    foes.push(self.team_pick[foes.len() % self.team_pick.len()]);
                }
                let t = if self.mode == Mode::TeamTag {
                    Team::tagged(self.team_pick.clone(), foes)
                } else {
                    Team::new(self.team_pick.clone(), foes, self.mode == Mode::TeamSimul)
                };
                self.sel = (t.members[0][0], t.members[1][0], self.sel.2);
                self.team = Some(t);
                self.screen = Screen::Stages { cur: 1 };
                return;
            }
            picked[picking] = Some(cur[picking]);
            let coop = self.coop && matches!(self.mode, Mode::Arcade | Mode::Survival);
            if picking == 0 && coop {
                picking = 1;
            } else if picking == 0 && self.mode == Mode::Arcade {
                self.arcade_start(cur[0], None);
                return;
            } else if picking == 0 && self.mode == Mode::Story {
                self.story_begin(cur[0]);
                return;
            } else if picking == 0 && self.mode == Mode::Survival {
                self.survival_begin(cur[0], None);
                return;
            } else if picking == 1 && coop {
                let Some(me) = picked[0] else {
                    self.screen = Screen::Select {
                        cur,
                        picking: 0,
                        picked: [None, None],
                    };
                    return;
                };
                if cur[1] == me {
                    self.msg = "O parceiro precisa ser outro lutador.".into();
                    self.screen = Screen::Select {
                        cur,
                        picking: 1,
                        picked: [Some(me), None],
                    };
                    return;
                }
                if self.mode == Mode::Arcade {
                    self.arcade_start(me, Some(cur[1]));
                } else {
                    self.survival_begin(me, Some(cur[1]));
                }
                return;
            } else if picking == 0 {
                picking = 1;
            } else {
                self.sel = (picked[0].unwrap(), picked[1].unwrap(), self.sel.2);
                self.screen = Screen::Stages {
                    cur: self.sel.2 + 1,
                };
                return;
            }
        }
        if back {
            if self.mode.is_team() && !self.team_pick.is_empty() {
                self.team_pick.pop();
                self.team_pal.pop();
                self.screen = Screen::Select {
                    cur,
                    picking: 0,
                    picked: [None, None],
                };
                return;
            }
            if picking == 1 {
                picking = 0;
                picked = [None, None];
            } else {
                self.screen = Screen::Menu { cur: 0 };
                return;
            }
        }
        self.screen = Screen::Select {
            cur,
            picking,
            picked,
        };
        self.draw_boss_toast(dt);
    }



    fn portrait_fit(&mut self, i: usize, x: f32, y: f32, w: f32, h: f32) {
        let e = &self.reg.chars[i];
        let por = if e.mugen {
            e.portrait()
        } else {
            e.tile_portrait(1)
                .or_else(|| e.peek(1).and_then(|d| d.sprites.get(&(9000, 1)).cloned()))
        };
        let Some((im, pw, ph)) =
            por.and_then(|p| p.image.clone().map(|im| (im, p.width, p.height)))
        else {
            return;
        };
        let tex = self.g.texture(&im, true);
        let kk = (w / pw).min(h / ph);
        draw_texture_ex(
            &tex,
            self.ui.x(x + (w - pw * kk) / 2.0),
            self.ui.y(y + (h - ph * kk) / 2.0),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(pw * kk * self.ui.s, ph * kk * self.ui.s)),
                ..Default::default()
            },
        );
    }

    fn select_side(
        &mut self,
        side: usize,
        idx: Option<usize>,
        locked: bool,
        hovering: bool,
        pal: i32,
        label: &str,
    ) {
        let acc = if side == 0 {
            hex("#3fa9ff")
        } else {
            hex("#ff4d6d")
        };
        let (px, py, pw, ph) = (
            if side == 0 { 24.0 } else { VW - 304.0 },
            100.0,
            280.0,
            540.0,
        );
        let cx = px + pw / 2.0;
        crate::menu_art::glass(&self.ui, px, py, pw, ph, 1.0);
        self.ui.rect(px, py, pw, 3.0, acc);
        self.ui.rect(
            if side == 0 { px } else { px + pw - 3.0 },
            py,
            3.0,
            ph,
            alpha(acc, 0.55),
        );
        self.ui
            .rect_lines(px, py, pw, ph, 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
        let s = self.ui.s;
        let Some(i) = idx else {
            self.ui.text(
                "?",
                cx,
                py + 340.0,
                220.0,
                Color::new(1.0, 1.0, 1.0, 0.07),
                Align::Center,
                true,
            );
            self.ui.text(
                if self.mode.is_team() {
                    crate::i18n::tr3("ADVERSÁRIOS SORTEADOS", "RANDOM OPPONENTS", "RIVALES AL AZAR")
                } else {
                    crate::i18n::tr3("AGUARDANDO", "WAITING", "ESPERANDO")
                },
                cx,
                py + ph - 24.0,
                15.0,
                hex("#a49cc0"),
                Align::Center,
                true,
            );
            self.ui.rect(px + 12.0, py + 12.0, 44.0, 20.0, acc);
            self.ui.text(
                label,
                px + 34.0,
                py + 28.0,
                15.0,
                BLACK,
                Align::Center,
                true,
            );
            return;
        };
        let col = rgb(self.reg.chars[i].color);
        let t = self.preview_t;
        let ease = if hovering && !locked {
            1.0 - (1.0 - (self.sel_dwell / 0.3).min(1.0)).powi(3)
        } else {
            1.0
        };
        glow(
            self.ui.x(cx),
            self.ui.y(py + 340.0),
            230.0 * s,
            alpha(col, 0.55),
        );
        draw_ellipse(
            self.ui.x(cx),
            self.ui.y(py + 488.0),
            105.0 * s,
            13.0 * s,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.5),
        );
        draw_ellipse(
            self.ui.x(cx),
            self.ui.y(py + 488.0),
            105.0 * s,
            13.0 * s,
            0.0,
            alpha(acc, 0.12),
        );
        if let Some(d) = self.reg.chars[i].peek(pal) {
            let anim = if locked && d.anims.contains_key(&181) {
                181
            } else {
                0
            };
            self.char_tint = Color::new(1.0, 1.0, 1.0, ease);
            let slide = (1.0 - ease) * if side == 0 { -60.0 } else { 60.0 };
            self.draw_char_fit(&d, anim, t, cx + slide, py + 478.0, 270.0, 450.0, side == 1);
            self.char_tint = WHITE;
        } else if !self.reg.chars[i].broken.get() {
            self.ui.progress_bar(
                cx - 90.0,
                py + 300.0,
                180.0,
                (self.sel_dwell / 0.4).min(0.95),
                crate::i18n::t("common.loading"),
            );
        }
        if self.sel_flash.0 == side && self.sel_flash.1 > 0.0 {
            let f = self.sel_flash.1;
            self.ui
                .rect(px, py, pw, ph, Color::new(1.0, 1.0, 1.0, 0.45 * f * f));
            draw_circle_lines(
                self.ui.x(cx),
                self.ui.y(py + 300.0),
                (1.0 - f) * 320.0 * s,
                4.0 * s,
                alpha(acc, f),
            );
        }
        let e = &self.reg.chars[i];
        let by = py + ph - 56.0;
        self.ui
            .rect(px, by, pw, 56.0, Color::new(0.0, 0.0, 0.05, 0.7));
        self.ui.rect(px, by, pw, 2.0, alpha(col, 0.9));
        let nm = self.names_up[i].as_str();
        self.ui.text_bold(
            nm,
            cx,
            by + 28.0,
            if nm.chars().count() > 14 { 19.0 } else { 26.0 },
            if locked { hex("#ffcc33") } else { WHITE },
            BLACK,
            Align::Center,
        );
        self.ui.text(
            &e.series,
            cx,
            by + 48.0,
            14.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
        self.ui.rect(px + 12.0, py + 12.0, 44.0, 20.0, acc);
        self.ui.text(
            label,
            px + 34.0,
            py + 28.0,
            15.0,
            BLACK,
            Align::Center,
            true,
        );
        if locked {
            self.ui
                .rect(px + pw - 82.0, py + 12.0, 70.0, 20.0, hex("#ffcc33"));
            self.ui.text(
                "PRONTO",
                px + pw - 47.0,
                py + 28.0,
                14.0,
                BLACK,
                Align::Center,
                true,
            );
        } else if e.mugen {
            self.ui
                .rect(px + pw - 62.0, py + 12.0, 50.0, 16.0, hex("#22e8ff"));
            self.ui.text(
                "MUGEN",
                px + pw - 37.0,
                py + 25.0,
                12.0,
                BLACK,
                Align::Center,
                true,
            );
        }
    }

    fn ui_clip(&mut self, x: f32, y: f32, w: f32, h: f32, f: impl FnOnce(&mut App)) {
        let (sx, sy, sw, sh) = (self.ui.x(x), self.ui.y(y), w * self.ui.s, h * self.ui.s);
        set_scissor(Some((sx as i32, sy as i32, sw as i32, sh as i32)));
        f(self);
        set_scissor(None);
    }

    fn stages(&mut self) {
        let Screen::Stages { mut cur } = self.screen else {
            return;
        };
        let n = self.reg.stages.len() + 1;
        let cols = 4usize;
        let total_rows = n.div_ceil(cols);
        if self.c.pressed(nav::RIGHT) {
            cur = (cur + 1) % n;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::LEFT) {
            cur = (cur + n - 1) % n;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::DOWN) {
            cur = (cur + cols).min(n - 1);
        }
        if self.c.pressed(nav::UP) {
            cur = cur.saturating_sub(cols);
        }
        if is_key_pressed(KeyCode::PageDown) {
            cur = (cur + cols * VIEW_ROWS).min(n - 1);
        }
        if is_key_pressed(KeyCode::PageUp) {
            cur = cur.saturating_sub(cols * VIEW_ROWS);
        }
        let wheel = mouse_wheel().1;
        if wheel > 0.0 {
            cur = cur.saturating_sub(cols);
        } else if wheel < 0.0 {
            cur = (cur + cols).min(n - 1);
        }
        let crow = cur / cols;
        if crow < self.stage_row {
            self.stage_row = crow;
        }
        if crow >= self.stage_row + VIEW_ROWS {
            self.stage_row = crow + 1 - VIEW_ROWS;
        }
        self.stage_row = self.stage_row.min(total_rows.saturating_sub(VIEW_ROWS));
        let mut ok = self.c.pressed(nav::OK | nav::START);
        self.ui.background();
        self.ui.text(
            crate::i18n::t("select.choose_stage"),
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        self.ui.text(
            &format!("{}/{} · PgUp/PgDn · {}", cur + 1, n, crate::i18n::tr3("roda do mouse", "mouse wheel", "rueda del ratón")),
            VW - 40.0,
            56.0,
            18.0,
            hex("#a49cc0"),
            Align::Right,
            false,
        );
        let (cw, ch) = (290.0, 134.0);
        for i in (self.stage_row * cols)..((self.stage_row + VIEW_ROWS) * cols).min(n) {
            let (x, y) = (
                40.0 + (i % cols) as f32 * (cw + 13.0),
                88.0 + ((i / cols) - self.stage_row) as f32 * (ch + 14.0),
            );
            if self.clicked_rect(x, y, cw, ch) {
                if cur == i {
                    ok = true;
                }
                cur = i;
            }
            let (name, th, mugen) = if i == 0 {
                (
                    crate::i18n::t("select.random").to_string(),
                    [[255, 47, 208], [34, 232, 255], [255, 226, 58]],
                    false,
                )
            } else {
                let s = &self.reg.stages[i - 1];
                (s.name.clone(), s.thumb, s.mugen)
            };
            self.ui
                .gradient_v(x, y, cw, ch * 0.5, rgb(th[0]), rgb(th[1]));
            self.ui
                .gradient_v(x, y + ch * 0.5, cw, ch * 0.5, rgb(th[1]), rgb(th[2]));
            if let Some(image) = i
                .checked_sub(1)
                .and_then(|index| self.reg.stage_preview(index))
            {
                let texture = self.g.texture(&image, true);
                draw_texture_ex(
                    &texture,
                    self.ui.x(x),
                    self.ui.y(y),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(cw * self.ui.s, ch * self.ui.s)),
                        source: Some(Rect::new(
                            0.0,
                            (image.h as f32 - image.w as f32 * ch / cw) * 0.5,
                            image.w as f32,
                            image.w as f32 * ch / cw,
                        )),
                        ..Default::default()
                    },
                );
            }
            self.ui
                .rect(x, y + ch - 30.0, cw, 30.0, Color::new(0.0, 0.0, 0.0, 0.6));
            self.ui.text(
                &name.chars().take(28).collect::<String>(),
                x + 10.0,
                y + ch - 9.0,
                16.0,
                WHITE,
                Align::Left,
                true,
            );
            if mugen {
                self.ui
                    .rect(x + cw - 60.0, y + 6.0, 54.0, 16.0, hex("#22e8ff"));
                self.ui.text(
                    "MUGEN",
                    x + cw - 33.0,
                    y + 19.0,
                    12.0,
                    BLACK,
                    Align::Center,
                    true,
                );
            }
            if cur == i {
                self.ui
                    .rect_lines(x - 3.0, y - 3.0, cw + 6.0, ch + 6.0, 4.0, hex("#ffcc33"));
            }
        }
        if total_rows > VIEW_ROWS {
            let bh = VIEW_ROWS as f32 * (ch + 14.0) - 14.0;
            self.ui
                .rect(VW - 22.0, 88.0, 6.0, bh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (bh * VIEW_ROWS as f32 / total_rows as f32).max(18.0);
            self.ui.rect(
                VW - 22.0,
                88.0 + (bh - th) * self.stage_row as f32 / (total_rows - VIEW_ROWS) as f32,
                6.0,
                th,
                hex("#ff4d6d"),
            );
        }
        if ok {
            self.sel.2 = if cur == 0 {
                (get_time() * 7.0) as usize % self.reg.stages.len()
            } else {
                cur - 1
            };
            self.go(Screen::Vs { t: 0.0, step: 0 });
            return;
        }
        if self.c.pressed(nav::BACK) {
            self.screen = Screen::Select {
                cur: [self.sel.0, self.sel.1],
                picking: 0,
                picked: [None, None],
            };
            return;
        }
        self.screen = Screen::Stages { cur };
    }

    /// Fighters to load for the current bout as (side, slot, registry index): leaders only, or whole teams in parallel mode.
    fn load_plan(&self) -> Vec<(usize, usize, usize)> {
        let mut v = vec![];
        match &self.team {
            Some(t) if t.whole_squad() => {
                for k in 0..t.members[0].len().max(t.members[1].len()) {
                    for side in 0..2 {
                        if let Some(&r) = t.members[side].get(k) {
                            v.push((side, k, r));
                        }
                    }
                }
            }
            _ => {
                v.push((0, 0, self.sel.0));
                v.push((1, 0, self.sel.1));
            }
        }
        v
    }

    /// One unit of fight preparation (runs behind the VS screen, one per frame, so the bar animates and the UI never freezes).
    /// Palette for a planned fighter: explicit team/player choice, with the classic duplicate-fighter fallback.
    fn pal_for(&self, plan: &[(usize, usize, usize)], st: usize) -> i32 {
        let (side, k, r) = plan[st];
        if let Some(p) = self
            .team_pal
            .get(k)
            .copied()
            .filter(|_| side == 0 && self.team.is_some())
        {
            return p.clamp(1, crate::settings::PALETTE_COUNT as i32);
        }
        let mut pal = self.side_pal(side);
        let dup = (side == 1 && plan.iter().any(|p| p.0 == 0 && p.2 == r))
            || plan.iter().any(|p| p.0 == side && p.1 < k && p.2 == r);
        if dup && pal == self.side_pal(0) {
            pal = (pal % crate::settings::PALETTE_COUNT as i32) + 1;
        }
        pal
    }

    pub(crate) fn side_pal(&self, side: usize) -> i32 {
        self.pal[side.min(1)].clamp(1, crate::settings::PALETTE_COUNT as i32)
    }

    /// Palette shown for a side on the select screen (duplicate fighters get a different palette by default).
    pub(crate) fn pal_for_side(&self, side: usize, picked: [Option<usize>; 2]) -> i32 {
        let pal = self.side_pal(side);
        if side == 1 && picked[0].is_some() && picked[0] == picked[1] && pal == self.side_pal(0) {
            (pal % crate::settings::PALETTE_COUNT as i32) + 1
        } else {
            pal
        }
    }

    async fn load_step(&mut self, step: u8, wait: bool) -> Result<bool, String> {
        let clock = ume_shell::Clock::now();
        let out = self.load_step_inner(step, wait).await;
        if crate::perf_enabled() {
            let msg = format!(
                "[perf] step {step} ({}): {:.1} ms",
                self.load_label(step),
                clock.elapsed().as_secs_f64() * 1000.0
            );
            #[cfg(target_arch = "wasm32")]
            crate::web_log(&msg);
            #[cfg(not(target_arch = "wasm32"))]
            eprintln!("{msg}");
        }
        out
    }

    async fn load_step_inner(&mut self, step: u8, wait: bool) -> Result<bool, String> {
        let plan = self.load_plan();
        let s = self.sel.2;
        let n = plan.len();
        let st = step as usize;
        if st < n {
            #[cfg(target_arch = "wasm32")]
            if !crate::web_sprites::ready(&self.reg.chars[plan[st].2])? { return Ok(false); }
            ume_shell::watchdog::label("load:character");
            if st == 0 && self.pending.jobs.is_empty() {
                let keep: Vec<usize> = plan.iter().map(|p| p.2).collect();
                self.reg.trim_cache(&keep);
                self.pending = Pending::default();
                let mut seen: Vec<(usize, i32)> = vec![];
                for k in 0..n {
                    let key = (plan[k].2, self.pal_for(&plan, k));
                    self.pending.jobs.push(if seen.contains(&key) {
                        None
                    } else {
                        seen.push(key);
                        self.reg.chars[key.0].spawn_load(key.1)
                    });
                }
            }
            if !wait
                && self
                    .pending
                    .jobs
                    .get(st)
                    .is_some_and(|j| j.as_ref().is_some_and(|h| !h.is_finished()))
            {
                return Ok(false);
            }
            let (side, _, r) = plan[st];
            let job = self.pending.jobs.get_mut(st).and_then(|j| j.take());
            let mut d = self.reg.chars[r].finish_load(self.pal_for(&plan, st), job)?;
            let is_boss_match = self.mode == Mode::Arcade
                && side == 1
                && self
                    .arcade
                    .as_ref()
                    .is_some_and(|(l, i)| *i + 1 == l.len() && l.last() == Some(&r));
            if is_boss_match {
                if let crate::registry::Src::Native(nc) = &self.reg.chars[r].src {
                    let mut boss_nc = (**nc).clone();
                    boss_nc.boss = true;
                    let mut def = ume_engine::native::compile_native(
                        &boss_nc,
                        self.pal_for(&plan, st),
                        true,
                    );
                    if def.c("data.life", 0.0) < 1800.0 {
                        def.constants.insert("data.life".into(), 2000.0);
                    }
                    if def.c("data.attack", 0.0) < 115.0 {
                        def.constants.insert("data.attack".into(), 125.0);
                    }
                    d = Arc::new(def);
                }
            }
            self.pending.defs[side].push(d);
        } else if st == n {
            ume_shell::watchdog::label("load:stage");
            self.pending.jobs.clear();
            let job = self
                .pending
                .stage_job
                .get_or_insert_with(|| self.reg.prepare_stage(s));
            if !job.ready_now() {
                return Ok(false);
            }
            let built = job.take()?;
            self.pending.stage_job = None;
            self.reg.adopt_stage_preview(s, built.preview);
            self.pending.stage = Some(built.stage);
        } else if st == n + 1 {
            ume_shell::watchdog::label("load:session");
            let stage = self.pending.stage.take().ok_or("stage")?;
            let teams = std::mem::take(&mut self.pending.defs);
            let boss_phase2 = self.mode == Mode::Arcade
                && self.arcade.as_ref().is_some_and(|(l, i)| {
                    *i > 0 && l.get(*i) == l.get(*i - 1) && l.get(*i).copied() == self.boss_idx()
                });
            let lvl = if let Some(story_lvl) = self.story_level() {
                story_lvl
            } else if self.mode == Mode::Survival {
                self.survival
                    .map(|s| s.ai_level(self.ai_level))
                    .unwrap_or(self.ai_level)
            } else if boss_phase2 {
                8
            } else {
                match &self.arcade {
                    Some((_, i)) if self.mode == Mode::Arcade => {
                        (2 + *i as i32 + self.ai_level / 3).min(8)
                    }
                    _ => self.ai_level,
                }
            };
            let leader = |side: usize| -> Pilot {
                match (self.mode, side) {
                    (Mode::Watch, 0) => Pilot::Cpu(Ai::new(lvl, 11)),
                    (Mode::Watch, _) => Pilot::Cpu(Ai::new(lvl, 77)),
                    (Mode::Versus, 0) => Pilot::Human(0),
                    (Mode::Versus, _) => Pilot::Human(1),
                    (Mode::Training, 0) => Pilot::Human(0),
                    (Mode::Training, _) => Pilot::Dummy,
                    (_, 0) => Pilot::Human(0),
                    _ => Pilot::Cpu(Ai::new(lvl, 99)),
                }
            };
            let mut pilots = vec![];
            let replay = self.replaying.clone();
            let coop = self.coop && matches!(self.mode, Mode::Arcade | Mode::Survival);
            for k in 0..teams[0].len().max(teams[1].len()) {
                for side in 0..2 {
                    if teams[side].get(k).is_some() {
                        pilots.push(if k == 0 {
                            leader(side)
                        } else {
                            Pilot::Cpu(Ai::new((lvl - 1).max(1), 31 + (side * 7 + k) as u32))
                        });
                    }
                }
            }
            if coop && teams[0].len() > 1 && pilots.len() > 2 {
                pilots[2] = Pilot::Human(1);
            }
            if let Some(r) = &replay {
                for (slot, p) in pilots.iter_mut().enumerate() {
                    *p = Pilot::Replay(
                        r.rec
                            .iter()
                            .map(|b| b.get(slot).copied().unwrap_or(0))
                            .collect(),
                        0,
                    );
                }
            }
            let (simul, tag, seq) = (
                self.team.as_ref().is_some_and(|t| t.simul),
                self.team.as_ref().is_some_and(|t| t.tag),
                self.team.as_ref().is_some_and(|t| !t.whole_squad()),
            );
            let life = if let Some(r) = &replay {
                r.life
            } else if matches!(self.mode, Mode::Survival | Mode::Story) {
                self.run_life()
            } else {
                self.team
                    .as_ref()
                    .filter(|t| !t.whole_squad())
                    .map(|t| t.life)
                    .unwrap_or([1.0, 1.0])
            };
            let info = self.team.as_ref().map(|t| crate::session::TeamInfo {
                names: [0, 1].map(|sd| {
                    t.members[sd]
                        .iter()
                        .map(|&r| {
                            (
                                self.reg.chars[r]
                                    .name
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_uppercase(),
                                self.reg.chars[r].color,
                            )
                        })
                        .collect()
                }),
                idx: t.idx,
                ko: t.ko.clone(),
                simul: t.whole_squad(),
            });
            self.c.clear_macros();
            let mut sess = Session::new(
                teams,
                stage,
                pilots,
                self.mode == Mode::Training,
                self.mode == Mode::Versus || coop,
                simul,
                tag,
                seq,
                life,
            );
            sess.hud.bars = self.sp.as_ref().map(|sp| {
                [
                    sp.bar_span(0, false, sp.localcoord),
                    sp.bar_span(1, false, sp.localcoord),
                ]
            });
            if let Some(t) = &info {
                sess.m.team_lead =
                    [0, 1].map(|sd| t.names[sd].first().map(|n| n.0.clone()).unwrap_or_default());
            }
            if let Some(t) = &info {
                sess.m.team_turns = seq;
                sess.m.team_size = [t.names[0].len(), t.names[1].len()];
                sess.m.team_member = t.idx;
            }
            sess.team = info;
            sess.is_replay = replay.is_some();
            let seed = replay.as_ref().map_or_else(crate::app::fight_seed::fresh_seed, |r| r.seed);
            sess.m.rng = seed;
            sess.seed = seed;
            self.fight_rules = replay
                .as_ref()
                .map(|r| r.rules)
                .unwrap_or_else(|| self.settings.rules());
            sess.apply_rules(self.fight_rules);
            self.story_tune(&mut sess);
            if boss_phase2 {
                sess.m.awaken_boss_final();
            }
            if seq {
                if let Some(t) = &self.team {
                    for side in 0..2 {
                        if let Some(f) = sess
                            .m
                            .fighters
                            .iter_mut()
                            .find(|f| !f.is_helper && f.side == side)
                        {
                            f.power = t.power[side].min(f.power_max);
                        }
                    }
                }
            }
            sess.r.debug = self.debug_boxes;
            if seq && self.team.as_ref().is_some_and(|t| t.idx != [0, 0]) {
                sess.vfx.banner(
                    crate::i18n::tr3("PRÓXIMO LUTADOR", "NEXT FIGHTER", "SIGUIENTE LUCHADOR"),
                    &format!(
                        "{} vs {}",
                        self.reg.chars[self.sel.0].name, self.reg.chars[self.sel.1].name
                    ),
                    [0.4, 0.9, 1.0],
                );
            }
            self.pending.sess = Some(sess);
        } else if st == n + 2 {
            ume_shell::watchdog::label("load:audio");
            let music = self
                .pending
                .sess
                .as_ref()
                .map(|s| s.stage.music)
                .unwrap_or(0);
            let mut sfx_ready = true;
            if let Some(sess) = &self.pending.sess {
                let mut chars: Vec<crate::audio::CharSounds> = vec![];
                for f in &sess.m.fighters {
                    if !chars.iter().any(|(id, _)| *id == f.def.id) {
                        chars.push((&f.def.id, &f.def.sounds));
                    }
                }
                sfx_ready = self.audio.preload_match(&chars);
            }
            let music_ready = self.audio.music_ready(music);
            if !sfx_ready || !music_ready {
                if wait {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                    return Box::pin(self.load_step(step, wait)).await;
                }
                return Ok(false);
            }
            self.audio.prepare_music(music).await;
        } else if !self.warm_step() {
            if wait {
                std::thread::sleep(std::time::Duration::from_millis(2));
                return Box::pin(self.load_step(step, wait)).await;
            }
            return Ok(false);
        }
        Ok(true)
    }

    fn finish_import(&mut self, read: Result<(String, ume_formats::vfs::Vfs), String>) {
        match read.map(|(pack, vfs)| self.reg.finish_import(pack, vfs)) {
            Ok(r) if r.duplicate_pack || (r.chars + r.stages == 0 && r.errors == 0) => {
                self.msg = crate::i18n::tr3("Pacote já importado", "Pack already imported", "Paquete ya importado").into()
            }
            Ok(r) => {
                self.reg.sort_mugen();
                self.sel = (
                    self.sel.0.min(self.reg.chars.len() - 1),
                    self.sel.1.min(self.reg.chars.len() - 1),
                    self.sel.2.min(self.reg.stages.len() - 1),
                );
                self.msg = crate::i18n::fill(
                    crate::i18n::tr3("Importado: {c} personagem(ns), {s} cenário(s){e}", "Imported: {c} character(s), {s} stage(s){e}", "Importado: {c} personaje(s), {s} escenario(s){e}"),
                    &[("c", &r.chars.to_string()), ("s", &r.stages.to_string()), ("e", &if r.errors == 0 { String::new() } else { format!(" · {} {}", r.errors, crate::i18n::tr3("erro(s)", "error(s)", "error(es)")) })],
                );
            }
            Err(e) => self.msg = format!("{}: {e}", crate::i18n::tr3("Zip inválido", "Invalid zip", "Zip no válido")),
        }
    }

    fn abort_load(&mut self, e: String) {
        self.msg = e;
        self.pending = Pending::default();
        self.screen = Screen::Menu { cur: 0 };
    }

    async fn vs(&mut self, dt: f32) {
        let Screen::Vs { t, step } = &mut self.screen else {
            return;
        };
        *t += dt;
        let (t, step_v) = (*t, *step);
        // work for the step labelled on the previous frame (that frame is still on screen while we block)
        let mut step_now = step_v;
        let total = self.load_total();
        if t > dt * 1.5 && step_v < total {
            match self.load_step(step_v, false).await {
                Err(e) => {
                    self.abort_load(e);
                    return;
                }
                Ok(true) => {
                    step_now = step_v + 1;
                    if let Screen::Vs { step, .. } = &mut self.screen {
                        *step = step_now;
                    }
                }
                Ok(false) => {}
            }
        }
        let ready = step_now >= total;
        self.draw_versus(t);
        if !ready {
            let within = if step_now + 1 == total { self.warm_progress() } else { 0.0 };
            self.ui.progress_bar(
                440.0,
                75.0,
                400.0,
                (step_now as f32 + within) / total as f32,
                &self.load_label(step_now),
            );
        } else if t > 1.8
            || self.c.pressed(nav::OK | nav::START)
            || is_mouse_button_pressed(MouseButton::Left)
        {
            self.finish_fight().await;
        }
    }

    /// Synchronous variant (screenshot tour, rematch): runs every loading step back to back.
    /// The sprite warm-up is the one step that is paid over several frames here: it copies 1 MiB per
    /// frame, yielding between chunks, so a rematch or a netplay start does not freeze the window for
    /// the hundreds of milliseconds a full fighter upload takes.
    pub async fn start_fight(&mut self) {
        let total = self.load_total();
        for st in 0..total {
            if st + 1 == total {
                while !self.warm_step() {
                    next_frame().await;
                }
                continue;
            }
            loop {
                match self.load_step(st, false).await {
                    Ok(true) => break,
                    Ok(false) => {
                        self.ui.begin();
                        self.ui.loading_screen((st as f32 + 0.5) / total as f32, &self.load_label(st));
                        next_frame().await;
                    }
                    Err(e) => {
                        self.abort_load(e);
                        return;
                    }
                }
            }
        }
        self.finish_fight().await;
    }

    async fn finish_fight(&mut self) {
        let Some(sess) = self.pending.sess.take() else {
            self.screen = Screen::Menu { cur: 0 };
            return;
        };
        let music = sess.stage.music;
        let bgm = sess.stage.bgm.clone().or_else(|| self.market.pick_track());
        // DualShock/DualSense lightbars: P1 blue, P2 red
        self.c.pads.set_light(0, [30, 90, 255]);
        self.c.pads.set_light(1, [255, 40, 60]);
        self.session = Some(sess);
        self.screen = Screen::Fight;
        self.audio.reset_bgm();
        if let Some(b) = bgm.as_ref() {
            self.audio.register_music_file("stage", b.0.clone(), b.1);
        }
        let custom = match bgm {
            Some(b) => self.audio.play_music_file(&b.0, b.1).await,
            None => false,
        };
        if !custom {
            self.audio.play_music(music).await;
        }
    }

    async fn fight(&mut self, dt: f32) {
        let replaying = self.replaying.is_some();
        let Some(s) = self.session.as_mut() else {
            self.screen = Screen::Menu { cur: 0 };
            return;
        };
        if replaying && is_key_pressed(KeyCode::Escape) {
            self.leave_replay();
            return;
        }
        if self.c.pressed(nav::AUX) {
            s.r.debug = !s.r.debug;
            self.debug_boxes = s.r.debug;
        }
        let pause_btn = if replaying {
            is_key_pressed(KeyCode::Space) || self.c.pressed(nav::START)
        } else {
            (self.c.pressed(nav::START) && !is_key_down(self.c.keys.slots[0][15]) && !is_key_pressed(KeyCode::Enter) && !is_key_pressed(KeyCode::KpEnter))
                || is_key_pressed(KeyCode::Escape)
                || is_key_pressed(KeyCode::P)
                || (!s.paused && is_key_pressed(KeyCode::Enter))
        };
        let (paused, accept_back) = pause_step(s.paused, pause_btn);
        if pause_btn {
            self.pause.reset();
            self.audio.sfx("select");
        }
        s.paused = paused;
        if replaying {
            if is_key_pressed(KeyCode::Key1) {
                s.set_speed(1.0);
            }
            if is_key_pressed(KeyCode::Key2) {
                s.set_speed(2.0);
            }
            if is_key_pressed(KeyCode::Key4) {
                s.set_speed(4.0);
            }
            if s.paused && (is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::Right)) {
                s.step_frame(&mut self.c, &mut self.audio);
            }
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if s.net.is_some() {
            s.paused = false;
        }
        // pause button (touch) top-center
        let pb = (VW / 2.0 - 22.0, 92.0, 44.0, 44.0);
        if !s.paused && self.c.touch.enabled && is_mouse_button_pressed(MouseButton::Left) {
            let p = self.ui.to_virtual(mouse_position().into());
            if p.x > pb.0 && p.x < pb.0 + pb.2 && p.y > pb.1 && p.y < pb.1 + pb.3 {
                s.paused = true;
            }
        }
        if is_key_pressed(KeyCode::Tab) && !s.paused && !replaying {
            s.switch_control();
        }
        s.update(dt, &mut self.c, &mut self.audio);
        s.draw(&mut self.g, &self.ui);
        if replaying {
            self.replay_bar();
        }
        if self.survival_is_playing() {
            self.survival_hud();
        }
        if self.story_is_playing() {
            self.story_hud();
        }
        self.c.touch.draw();
        if self.c.touch.enabled {
            self.ui
                .rect(pb.0, pb.1, pb.2, pb.3, Color::new(0.0, 0.0, 0.0, 0.35));
            self.ui.text(
                "II",
                VW / 2.0,
                pb.1 + 30.0,
                20.0,
                WHITE,
                Align::Center,
                true,
            );
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(n) = self.session.as_ref().and_then(|s| s.net.as_ref()) {
            self.net_overlay(n);
            if n.desyncs > 0 && !self.desync_reported {
                self.desync_reported = true;
                let (frame, checksum) = n.desync_at.unwrap_or((0, 0));
                let rec = self.session.as_ref().map(|s| s.rec.clone()).unwrap_or_default();
                match crate::replays::save_desync(
                    &crate::replays::dir(),
                    Some(&n.config),
                    frame,
                    checksum,
                    rec.len(),
                    &rec,
                ) {
                    Ok(p) => {
                        self.msg = format!(
                            "Desync: report at {}",
                            p.file_name()
                                .map(|f| f.to_string_lossy().into_owned())
                                .unwrap_or_default()
                        )
                    }
                    Err(e) => self.msg = e,
                }
            }
        }
        let s = self.session.as_mut().unwrap();
        if s.paused {
            if !replaying {
                self.pause_menu(accept_back).await;
            }
            return;
        }
        if let Some(w) = s.over {
            if s.over_timer > 1.0 && !self.team_next(w) {
                // keep the fight so it can be replayed (not for replays themselves or sequence team bouts)
                if self.replaying.is_none() && !self.team.as_ref().is_some_and(|t| !t.whole_squad())
                {
                    if let Some(s) = self.session.as_ref() {
                        self.last_fight = Some(Replay {
                            sel: self.sel,
                            mode: self.mode,
                            team: self.team.clone(),
                            rec: s.rec.clone(),
                            rules: self.fight_rules,
                            seed: s.seed,
                            life: self.run_life(),
                            pal: self.pal,
                            team_pal: self.team_pal.clone(),
                            net: s.netplay_config(),
                        });
                    }
                    self.archive_last_fight();
                }
                self.screen = Screen::Results { winner: w, cur: 0 };
            }
        }
    }

    /// Sequence mode: after a bout the winner keeps (most of) its life and the loser's next teammate steps in.
    /// Returns true when another bout was queued (false → the team match is over / not a sequence).
    fn team_next(&mut self, w: usize) -> bool {
        let Some(t) = self.team.as_mut() else {
            return false;
        };
        if t.whole_squad() {
            return false;
        }
        let Some(s) = self.session.as_ref() else {
            return false;
        };
        let ratio =
            s.m.roots()
                .find(|f| f.side == w)
                .map(|f| (f.life / f.life_max).clamp(0.0, 1.0))
                .unwrap_or(1.0);
        // shared team meter carries over to the next fighter
        for side in 0..2 {
            if let Some(f) = s.m.roots().find(|f| f.side == side) {
                t.power[side] = f.power;
            }
        }
        let lose = 1 - w;
        if let Some(k) = t.ko[lose].get_mut(t.idx[lose]) {
            *k = true;
        }
        t.idx[lose] += 1;
        t.life[lose] = 1.0;
        t.life[w] = (ratio + 0.12).min(1.0);
        if t.idx[lose] >= t.members[lose].len() {
            return false;
        }
        self.sel = (t.members[0][t.idx[0]], t.members[1][t.idx[1]], self.sel.2);
        self.audio.stop_music();
        self.session = None;
        self.screen = Screen::Vs { t: 0.0, step: 0 };
        true
    }
    fn reset_team(&mut self) {
        if let Some(t) = self.team.as_mut() {
            *t = Team {
                tag: t.tag,
                ..Team::new(t.members[0].clone(), t.members[1].clone(), t.simul)
            };
            self.sel = (t.members[0][0], t.members[1][0], self.sel.2);
        }
    }

    async fn results(&mut self) {
        let Screen::Results { winner, mut cur } = self.screen else {
            return;
        };
        if let Some(s) = self.session.as_mut() {
            s.draw(&mut self.g, &self.ui);
        }
        // survival: win advances to the next opponent, loss ends the run
        if matches!(self.mode, Mode::Survival | Mode::Story) && self.replaying.is_none() {
            if winner == 0 {
                self.run_win();
                return;
            }
            self.run_loss();
            return;
        }
        // arcade progression
        if self.mode == Mode::Arcade && winner == 0 {
            let boss = self.boss_idx();
            let mut next_action = None;
            if let Some((ladder, i)) = self.arcade.as_mut() {
                *i += 1;
                if *i < ladder.len() {
                    let is_boss = Some(ladder[*i]) == boss;
                    let is_phase2 = is_boss && *i > 0 && Some(ladder[*i - 1]) == boss;
                    next_action = Some((*i, ladder[*i], is_boss, is_phase2, false));
                } else if *i == ladder.len()
                    && !self.ending_shown
                    && boss.is_some_and(|b| ladder.last() == Some(&b))
                {
                    self.ending_shown = true;
                    next_action = Some((*i, 0, false, false, true));
                }
            }
            if let Some((idx, next_char, is_boss, is_phase2, is_ending)) = next_action {
                self.audio.stop_music();
                if is_ending {
                    let hero_id = self.player_char_id();
                    let boss_entry_id = boss
                        .map(|b| {
                            self.reg.chars[b]
                                .id
                                .strip_prefix("umf:")
                                .unwrap_or(&self.reg.chars[b].id)
                                .to_string()
                        })
                        .unwrap_or_default();
                    self.boss_progression.on_clear(&hero_id, &boss_entry_id);
                    self.screen = Screen::Cutscene {
                        kind: 2,
                        page: 0,
                        t: 0.0,
                    };
                    return;
                }
                self.sel.1 = next_char;
                let st_spirit = self.reg.stages.iter().position(|s| s.id.contains("spirit"));
                self.sel.2 = if is_phase2 {
                    st_spirit.unwrap_or(0)
                } else if is_boss {
                    self.boss_stage_idx(next_char)
                } else {
                    (idx * 3 + 1) % self.reg.stages.len()
                };
                if self.coop {
                    if let Some(t) = self.team.as_mut() {
                        t.members[1] = vec![next_char];
                    }
                }
                self.screen = if is_phase2 {
                    Screen::Cutscene {
                        kind: 3,
                        page: 0,
                        t: 0.0,
                    }
                } else if is_boss {
                    Screen::Cutscene {
                        kind: 1,
                        page: 0,
                        t: 0.0,
                    }
                } else {
                    Screen::Vs { t: 0.0, step: 0 }
                };
                return;
            }
        }
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.02, 0.01, 0.06, 0.6),
        );
        self.ui.motes(70, 1.6);
        let w = if winner == 0 { self.sel.0 } else { self.sel.1 };
        let e = &self.reg.chars[w];
        self.ui.gradient_v(
            160.0,
            200.0,
            960.0,
            300.0,
            alpha(rgb(e.color), 0.9),
            Color::new(0.08, 0.05, 0.16, 0.95),
        );
        let done =
            self.mode == Mode::Arcade && self.arcade.as_ref().is_some_and(|(l, i)| *i >= l.len());
        let title = if done {
            crate::i18n::t("results.arcade_complete").to_string()
        } else if self.team.is_some() {
            {
                let lead = self
                    .team
                    .as_ref()
                    .and_then(|t| t.members[winner.min(1)].first().copied())
                    .unwrap_or(w);
                crate::i18n::tf(
                    "results.team_wins",
                    &[(
                        "{name}",
                        &self.reg.chars[lead]
                            .name
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_uppercase(),
                    )],
                )
            }
        } else {
            crate::i18n::tf("results.wins", &[("{name}", &e.name.to_uppercase())])
        };
        let pal = if winner == 1 && self.sel.0 == self.sel.1 {
            2
        } else {
            1
        };
        if let Some(d) = e.peek(pal) {
            let t = self.preview_t;
            self.draw_char(
                &d,
                if d.anims.contains_key(&181) { 181 } else { 0 },
                t,
                330.0,
                480.0,
                260.0,
                false,
            );
        }
        self.ui.text_bold(
            &title,
            480.0,
            300.0,
            44.0,
            hex("#ffcc33"),
            BLACK,
            Align::Left,
        );
        let can_replay = self.last_fight.is_some();
        let items: Vec<&str> = if can_replay {
            vec![
                crate::i18n::t("results.rematch"),
                crate::i18n::t("results.replay"),
                crate::i18n::t("results.characters"),
                crate::i18n::t("results.menu"),
            ]
        } else {
            vec![
                crate::i18n::t("results.rematch"),
                crate::i18n::t("results.characters"),
                crate::i18n::t("results.menu"),
            ]
        };
        let nitems = items.len();
        if self.c.pressed(nav::RIGHT) {
            cur = (cur + 1) % nitems;
        }
        if self.c.pressed(nav::LEFT) {
            cur = (cur + nitems - 1) % nitems;
        }
        let mut ok = self.c.pressed(nav::OK | nav::START);
        for (i, it) in items.iter().enumerate() {
            let (bx, by) = (480.0 + i as f32 * 160.0, 380.0);
            if self.clicked_rect(bx, by, 150.0, 50.0) {
                cur = i;
                ok = true;
            }
            self.ui.rect(
                bx,
                by,
                150.0,
                50.0,
                if i == cur {
                    hex("#ff4d6d")
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.08)
                },
            );
            self.ui
                .text(it, bx + 75.0, by + 33.0, 20.0, WHITE, Align::Center, true);
        }
        self.screen = Screen::Results { winner, cur };
        if ok {
            self.audio.stop_music();
            if done {
                self.screen = self.arcade_return_to_select();
                return;
            }
            // with a replay available the menu is Revanche / Replay / Personagens / Menu; without it Revanche / Personagens / Menu
            let choice = if can_replay {
                cur
            } else if cur >= 1 {
                cur + 1
            } else {
                0
            };
            match choice {
                0 => {
                    self.replaying = None;
                    if let Some((l, i)) = self.arcade.as_mut() {
                        *i = (*i).min(l.len().saturating_sub(1));
                    }
                    self.reset_team();
                    self.start_fight().await;
                }
                1 => {
                    self.start_replay().await;
                }
                2 => {
                    self.replaying = None;
                    self.session = None;
                    self.team = None;
                    self.team_pick.clear();
                    self.screen = Screen::Select {
                        cur: [self.sel.0, self.sel.1],
                        picking: 0,
                        picked: [None, None],
                    };
                }
                _ => {
                    self.replaying = None;
                    self.session = None;
                    self.team = None;
                    self.screen = Screen::Menu { cur: 0 };
                }
            }
        }
    }

    /// Snapshot the running session as the replayable "last fight" (used by the screenshot tour's determinism check).
    pub fn capture_replay(&mut self) {
        if let Some(s) = self.session.as_ref() {
            self.last_fight = Some(Replay {
                sel: self.sel,
                mode: self.mode,
                team: self.team.clone(),
                rec: s.rec.clone(),
                rules: self.fight_rules,
                seed: s.seed,
                life: self.run_life(),
                pal: self.pal,
                team_pal: self.team_pal.clone(),
                net: s.netplay_config(),
            });
        }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub async fn start_netplay(&mut self, a: &crate::netlink::NetArgs) {
        let find_char = |id: &str| {
            self.reg
                .chars
                .iter()
                .position(|c| c.id.eq_ignore_ascii_case(id) || c.name.eq_ignore_ascii_case(id))
        };
        let (c1, c2) = match &a.chars {
            Some((x, y)) => (find_char(x), find_char(y)),
            None => (Some(0), Some(1.min(self.reg.chars.len() - 1))),
        };
        let (Some(c1), Some(c2)) = (c1, c2) else {
            self.msg = format!("Netplay: character not found ({:?})", a.chars);
            return;
        };
        let stage = a.stage.as_ref().map_or(Some(0), |id| {
            self.reg
                .stages
                .iter()
                .position(|s| s.id.eq_ignore_ascii_case(id) || s.name.eq_ignore_ascii_case(id))
        });
        let Some(stage) = stage else {
            self.msg = format!("Netplay: stage not found ({:?})", a.stage);
            return;
        };
        let link = match crate::netlink::NetLink::connect(a) {
            Ok(l) => l,
            Err(e) => {
                self.msg = format!("Netplay: {e}");
                return;
            }
        };
        self.mode = if a.spectate {
            Mode::Watch
        } else {
            Mode::Versus
        };
        self.team = None;
        self.arcade = None;
        self.replaying = None;
        self.desync_reported = false;
        self.sel = (c1, c2, stage);
        let saved = self.settings;
        self.settings = Settings {
            round_time: 0,
            rounds: 1,
            life: 2,
            single_round: false,
            classic_pushbox: true,
            ..saved
        };
        self.start_fight().await;
        self.settings = saved;
        if let Some(s) = self.session.as_mut() {
            s.m.rng = ume_engine::matchs::DEFAULT_SEED;
            s.seed = ume_engine::matchs::DEFAULT_SEED;
            s.net = Some(link);
        }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    fn net_overlay(&self, n: &crate::netlink::NetLink) {
        use crate::netlink::NetStatus;
        let (text, color) = match n.status() {
            NetStatus::Connecting => (crate::i18n::tr3("CONECTANDO AO ADVERSÁRIO…", "CONNECTING TO OPPONENT…", "CONECTANDO CON EL RIVAL…").to_string(), hex("#ffcc33")),
            NetStatus::Waiting => (crate::i18n::tr3("SINCRONIZANDO…", "SYNCING…", "SINCRONIZANDO…").to_string(), hex("#ffcc33")),
            NetStatus::Lost(e) => (format!("{}: {e}", crate::i18n::tr3("CONEXÃO PERDIDA", "CONNECTION LOST", "CONEXIÓN PERDIDA")), hex("#ff4d6d")),
            NetStatus::Running => {
                if n.spectate {
                    (
                        format!(
                            "{} · {} ms · {} {}",
                            crate::i18n::tr3("ESPECTADOR", "SPECTATOR", "ESPECTADOR"),
                            n.ping_ms().unwrap_or(0),
                            n.frames_behind(),
                            crate::i18n::tr3("atrás", "behind", "atrás")
                        ),
                        hex("#a49cc0"),
                    )
                } else {
                    (
                        format!("ONLINE P{} · {} ms", n.local + 1, n.ping_ms().unwrap_or(0)),
                        hex("#a49cc0"),
                    )
                }
            }
        };
        self.ui
            .text(&text, VW / 2.0, VH - 16.0, 18.0, color, Align::Center, true);
        if n.desyncs > 0 {
            self.ui.text(
                &format!("DESSINCRONIZADO ({})", n.desyncs),
                VW / 2.0,
                VH - 40.0,
                18.0,
                hex("#ff4d6d"),
                Align::Center,
                true,
            );
        }
    }

    /// Re-simulate the last finished fight from its recorded inputs (deterministic engine → identical fight).
    pub async fn start_replay(&mut self) {
        let Some(r) = self.last_fight.clone() else {
            return;
        };
        self.mode = r.mode;
        self.team = r.team.clone();
        self.sel = r.sel;
        self.pal = r.pal;
        self.team_pal = r.team_pal.clone();
        self.replaying = Some(r);
        self.start_fight().await;
    }

    fn mugen_screen(&mut self) {
        let Screen::Mugen { view, page } = self.screen else { return };
        self.screen = match self.mugen_catalog(view, page) {
            Some((view, page)) => Screen::Mugen { view, page },
            None => Screen::Menu { cur: 10 },
        };
    }

    // used by the screenshot tour
    pub fn draw_session_only(&mut self) {
        if let Some(s) = self.session.as_mut() {
            s.draw(&mut self.g, &self.ui);
            self.c.touch.draw();
        }
    }
    pub fn preview_opts<'a>() -> DrawOpts<'a> {
        DrawOpts::default()
    }
    pub fn renderer_new(st: &crate::stages::Stage) -> MatchRenderer {
        MatchRenderer::new(st)
    }
}

/// Pause state for this frame and whether the pause menu may act on `nav::BACK`.
///
/// Esc is both the pause key and `nav::BACK`. Without this guard the same key press
/// opens the menu in `fight()` and closes it again in `pause_menu()` on the same frame,
/// so the pause menu never shows. The opening frame therefore consumes that edge.
fn pause_step(was_paused: bool, toggle: bool) -> (bool, bool) {
    (was_paused ^ toggle, was_paused)
}

/// Cycle a palette number (1..=PALETTE_COUNT) wrapping on both ends.
pub(crate) fn cycle_palette(cur: i32, dir: i32) -> i32 {
    let n = crate::settings::PALETTE_COUNT as i32;
    let cur = cur.clamp(1, n);
    ((cur - 1 + dir).rem_euclid(n)) + 1
}

/// Numpad notation (2 = down, 6 = forward, 236 = quarter-circle forward).

#[cfg(test)]
mod tests {
    use super::{cycle_palette, pause_step};

    #[test]
    fn palette_cycling_wraps_on_both_ends() {
        assert_eq!(cycle_palette(1, 1), 2);
        assert_eq!(
            cycle_palette(12, 1),
            1,
            "forward from the last palette wraps to the first"
        );
        assert_eq!(
            cycle_palette(1, -1),
            12,
            "backward from the first palette wraps to the last"
        );
        assert_eq!(
            cycle_palette(7, 0),
            7,
            "no direction keeps the current palette"
        );
        assert_eq!(
            cycle_palette(0, 1),
            2,
            "an invalid palette is clamped before cycling"
        );
        assert_eq!(
            cycle_palette(99, -1),
            11,
            "palettes above the range are clamped"
        );
    }

    #[test]
    fn esc_opening_the_pause_menu_does_not_close_it_on_the_same_frame() {
        // Esc is both the pause key and nav::BACK, so the opening frame consumes the edge.
        assert_eq!(pause_step(false, true), (true, false));
    }

    #[test]
    fn pause_menu_is_navigable_once_open() {
        assert_eq!(pause_step(true, false), (true, true));
        assert_eq!(pause_step(true, true), (false, true));
        assert_eq!(pause_step(false, false), (false, false));
    }

    #[test]
    fn character_story_and_ending_scripts_are_unique_for_each_fighter() {
        let roster = [
            "kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian", "garan",
            "marlow", "kaname", "jin", "daichi", "valdric", "haruki",
        ];
        let mut openings = std::collections::BTreeSet::new();
        let mut prebosses = std::collections::BTreeSet::new();
        let mut awakenings = std::collections::BTreeSet::new();
        let mut endings = std::collections::BTreeSet::new();
        for id in roster {
            let op = super::cutscene::script_for(0, id, id, "", "Mikazuki Oboro", "");
            let pb = super::cutscene::script_for(1, id, id, "", "Mikazuki Oboro", "");
            let aw = super::cutscene::script_for(3, id, id, "", "Mikazuki Oboro", "");
            let en = super::cutscene::script_for(2, id, id, "", "Mikazuki Oboro", "");
            assert!(op.len() >= 4 && pb.len() >= 5 && aw.len() >= 6 && en.len() >= 5);
            assert!(openings.insert(op.last().unwrap().text.clone()));
            assert!(prebosses.insert(pb[3].text.clone()));
            assert!(awakenings.insert(aw[4].text.clone()));
            assert!(endings.insert(en[1].text.clone()));
        }
    }
}
