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
mod results_stats;
mod select_category;
mod select_grid;
mod select_palette; mod select_stats;
mod versus; mod versus_compare;
mod menu_view; mod menu_icons;
mod menu_focus;
mod mode;
mod screen;
mod team;
pub use mode::Mode;
pub use screen::Screen;
pub use team::Team;
mod netplay_session;
mod results_screen;
mod fight_screen;
mod vs_screen;
mod load_plan;
mod stage_select;
mod fighter_draw;
mod menu_screen;
mod title_screen;
use select_category::CATEGORY_COUNT;
#[cfg(target_arch = "wasm32")]
mod web_import;
mod controls_screen;
mod empty_art;
pub(crate) use keyedit::KeyEdit;


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

const VIEW_ROWS: usize = 4;
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
    pub dummy_plan: crate::session::dummy::DummyPlan,
    menu_focus: menu_focus::MenuFocus,
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
    ("menu.story", "desc.story"),
    ("menu.arcade", "desc.arcade"),
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
            dummy_plan: Default::default(),
            menu_focus: Default::default(),
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
        crate::e2e::publish(self);
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
            self.quality_check = 0.0; self.apply_quality();
        }
        if let Some(note) = crate::tweaks::poll(dt) { self.msg = note; }
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

    /// Registry indices visible under the current filter (never empty).
    fn char_view(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self
            .playable()
            .into_iter()
            .filter(|&i| self.category_matches(i))
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
        self.sort_view(&mut v);
        v
    }

    fn menu_text(&self, key: &'static str) -> String {
        if let Some(label) = crate::tweaks::label(key) { return label; }
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
        let cols = self.grid_cols(m);
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
            pos = (pos + cols * self.grid_rows()).min(m - 1);
        }
        if is_key_pressed(KeyCode::PageUp) {
            pos = pos.saturating_sub(cols * self.grid_rows());
        }
        self.handle_select_keys();
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
            self.sel_filter = (self.sel_filter + 1) % CATEGORY_COUNT;
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
        if crow >= self.sel_row + self.grid_rows() {
            self.sel_row = crow + 1 - self.grid_rows();
        }
        self.sel_row = self.sel_row.min(total_rows.saturating_sub(self.grid_rows()));

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
            #[cfg(target_arch = "wasm32")]
            if self.sel_dwell >= 0.25 { let _ = crate::web_sprites::ready(e); }
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
        let gh = self.grid_rows() as f32 * (ch + gap) - gap;
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
        let last = ((self.sel_row + self.grid_rows()) * cols).min(m);
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
            self.draw_tile_force(cx, cy, cw, ch, i);
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
                crate::credit_line::truncate(up, 13)
            } else {
                clip(up.split_whitespace().next().unwrap_or(""), 12).to_string()
            };
            self.ui.text(
                &nm,
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
        if total_rows > self.grid_rows() {
            let bx = gx + gw + 3.0;
            self.ui
                .rect(bx, gy, 5.0, gh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (gh * self.grid_rows() as f32 / total_rows as f32).max(18.0);
            self.ui.rect(
                bx,
                gy + (gh - th) * self.sel_row as f32 / (total_rows - self.grid_rows()) as f32,
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
                self.select_palette_strip(side, pal, team_mode);
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
        let flabel = self.category_label();
        self.sel_buf.clear();
        let _ = write!(
            self.sel_buf,
            "[Tab] {flabel} · {}/{m} · [R] {} · [G] {} · [O] {} · PgUp/PgDn",
            pos + 1,
            crate::i18n::t("select.random").to_lowercase(),
            self.grid_label(),
            self.sort_label()
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
        let by = py + ph - 94.0;
        self.ui
            .rect(px, by, pw, 94.0, Color::new(0.0, 0.0, 0.05, 0.7));
        self.ui.rect(px, by, pw, 2.0, alpha(col, 0.9)); self.select_author(&e.author, cx, by + 88.0); self.draw_stat_gauges(px, by + 20.0, pw, i);
        let nm = self.names_up[i].as_str();
        self.ui.text_bold(
            nm,
            cx,
            by + 50.0,
            if nm.chars().count() > 14 { 19.0 } else { 26.0 },
            if locked { hex("#ffcc33") } else { WHITE },
            BLACK,
            Align::Center,
        );
        self.ui.text(
            &crate::credit_line::truncate(&e.series, 30),
            cx,
            by + 70.0,
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
            for p in &plan { let _ = crate::web_sprites::ready(&self.reg.chars[p.2]); }
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
            let boss_phase = if self.mode == Mode::Arcade {
                self.arcade.as_ref().map(|(l, i)| {
                    let b = self.boss_idx();
                    if Some(l.get(*i).copied().unwrap_or(usize::MAX)) == b {
                        l[..=*i].iter().filter(|&&x| Some(x) == b).count()
                    } else { 0 }
                }).unwrap_or(0)
            } else { 0 };
            let lvl = if let Some(story_lvl) = self.story_level() {
                story_lvl
            } else if self.mode == Mode::Survival {
                self.survival
                    .map(|s| s.ai_level(self.ai_level))
                    .unwrap_or(self.ai_level)
            } else if boss_phase >= 2 {
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
                    (Mode::Training, _) => Pilot::Dummy(self.dummy_plan),
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
            if boss_phase >= 2 {
                sess.m.awaken_boss_phase(boss_phase as u8);
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
mod tests;
