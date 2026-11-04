//! Screens and game flow.
use crate::audio::Audio;
use crate::controls::{nav, Controls};
use crate::registry::Registry;
use crate::render::{Gfx, MatchRenderer, DrawOpts};
use crate::session::{Pilot, Session};
use crate::settings::{Rules, Settings, FX_LEVELS, IMPACT, QUALITY, RESOLUTIONS};
use crate::stages::Stage;
use crate::ui::*;
use crate::gfx::*;
use std::fmt::Write;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;

mod cutscene;
mod keyedit;
mod market;
mod settings_screen;
mod versus;
pub(crate) use keyedit::KeyEdit;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode { Arcade, Cpu, Versus, Training, Watch, TeamSeq, TeamSimul }
impl Mode {
    fn label(self) -> &'static str { match self { Mode::Arcade => "ARCADE", Mode::Cpu => "VERSUS CPU", Mode::Versus => "2 JOGADORES", Mode::Training => "TREINO", Mode::Watch => "CPU vs CPU", Mode::TeamSeq => "EQUIPE · SEQUÊNCIA", Mode::TeamSimul => "EQUIPE · PARALELO" } }
    /// Team size (1 for the regular modes).
    pub fn is_team(self) -> bool { matches!(self, Mode::TeamSeq | Mode::TeamSimul) }
}

pub enum Screen {
    Title,
    Menu { cur: usize },
    Select { cur: [usize; 2], picking: usize, picked: [Option<usize>; 2] },
    Stages { cur: usize },
    Vs { t: f32, step: u8 },
    Fight,
    Results { winner: usize, cur: usize },
    Controls,
    Mugen,
    Market,
    Settings { tab: usize, cur: usize, from_fight: bool },
    /// story scene: 0 arcade opening · 1 before the boss · 2 ending
    Cutscene { kind: u8, page: usize, t: f32 },
}

/// Fight assets prepared step by step behind the VS screen.
#[derive(Default)]
pub struct Pending { defs: [Vec<Arc<CharDef>>; 2], stage: Option<Stage>, stage_job: Option<crate::registry::StageJob>, sess: Option<Session>, jobs: Vec<crate::registry::LoadJob> }

/// Everything needed to re-simulate a finished fight: same fighters, stage and mode, plus the recorded inputs per slot.
#[derive(Clone)]
pub struct Replay { sel: (usize, usize, usize), mode: Mode, team: Option<Team>, rec: Vec<Vec<u32>>, rules: Rules }

/// Team battle state: sequence (KOF-style turns, winner keeps its remaining life) or parallel (2v2 simultaneous).
#[derive(Clone)]
pub struct Team { pub members: [Vec<usize>; 2], pub idx: [usize; 2], pub life: [f32; 2], /** KOF XV: the team shares one super meter */ pub power: [f32; 2], pub ko: [Vec<bool>; 2], pub simul: bool }
impl Team {
    pub fn new(a: Vec<usize>, b: Vec<usize>, simul: bool) -> Self { let (la, lb) = (a.len(), b.len()); Team { members: [a, b], idx: [0, 0], life: [1.0, 1.0], power: [0.0, 0.0], ko: [vec![false; la], vec![false; lb]], simul } }
}
const VIEW_ROWS: usize = 4;
const NUMS: [&str; 5] = ["1", "2", "3", "4", "5"];

fn opaque_box(s: &ume_engine::chardef::Sprite) -> (f32, f32, f32, f32) {
    let Some(im) = s.image.as_ref() else { return (0.0, 0.0, s.width, s.height) };
    let (iw, ih) = (im.w as usize, im.h as usize);
    if iw == 0 || ih == 0 { return (0.0, 0.0, s.width, s.height); }
    let (mut x0, mut y0, mut x1, mut y1) = (iw, ih, 0, 0);
    for y in 0..ih {
        let row = &im.rgba[y * iw * 4..(y + 1) * iw * 4];
        for x in 0..iw {
            if row[x * 4 + 3] > 16 { x0 = x0.min(x); x1 = x1.max(x + 1); y0 = y0.min(y); y1 = y1.max(y + 1); }
        }
    }
    if x0 >= x1 || y0 >= y1 { return (0.0, 0.0, s.width, s.height); }
    let (kx, ky) = (s.width / iw as f32, s.height / ih as f32);
    (x0 as f32 * kx, y0 as f32 * ky, (x1 - x0) as f32 * kx, (y1 - y0) as f32 * ky)
}

fn clip(s: &str, n: usize) -> &str { s.char_indices().nth(n).map_or(s, |(i, _)| &s[..i]) }

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
    pub pause_cur: usize,
    pub debug_boxes: bool,
    pub quit: bool,
    pub msg: String,
    preview_t: f32,
    pub settings: Settings,
    pub(crate) key_edit: Option<keyedit::KeyEdit>,
    /** page of the pause-screen move list (14 moves per page) */
    move_page: usize,
    pending: Pending,
    pub team: Option<Team>,
    /// last finished fight (replayable) and the replay being played now
    last_fight: Option<Replay>,
    pub replaying: Option<Replay>,
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
    updater: crate::update::Checker,
    pub trans: f32,
    screen_t: f32,
    menu_art: crate::menu_art::MenuArt,
    last_disc: Option<std::mem::Discriminant<Screen>>,
    applied: Option<Settings>,
    settings_back: Option<(usize, bool)>,
    fight_rules: Rules,
}

const MENU: [(&str, &str); 12] = [
    ("Arcade", "Enfrente uma sequência de oponentes"),
    ("Versus CPU", "Você contra o computador"),
    ("2 Jogadores", "Teclado compartilhado ou dois controles"),
    ("Treino", "Boneco de treino, barra infinita, hitboxes"),
    ("CPU vs CPU", "Assista a IA lutar"),
    ("Equipe · Sequência", "Até 5 contra 5, um de cada vez; o vencedor mantém a vida"),
    ("Equipe · Paralelo", "5 contra 5 ao mesmo tempo (2 a 5) · TAB troca de lutador"),
    ("Personagens MUGEN", "Como adicionar personagens e cenários"),
    ("Mercado GitHub", "Baixe personagens, cenários e músicas de repositórios"),
    ("Controles", "Teclado, Xbox, PlayStation, toque e vibração"),
    ("Configurações", "Vídeo, áudio, controles, jogabilidade e idioma"),
    ("Sair", "Fechar o jogo"),
];

impl App {
    /// One splash frame of the boot loading screen.
    async fn boot(ui: &mut Ui, p: f32, label: &str) { ui.begin(); ui.loading_screen(p, label); next_frame().await; }

    pub async fn new(touch: bool, audio_on: bool) -> Self {
        let mut ui = Ui::new();
        Self::boot(&mut ui, 0.05, "Iniciando…").await;
        let mut settings = Settings::load();
        if touch { settings.fx = settings.fx.min(1); } // phones: no bloom passes
        ume_engine::native::rig::set_art_res(settings.art_res(Settings::physical_h()));
        Self::boot(&mut ui, 0.15, "Lutadores originais…").await;
        let mut reg = Registry::new();
        Self::boot(&mut ui, 0.35, "Pacote Elecbyte (Kung Fu Man)…").await;
        reg.add_bundled_mugen();

        Self::boot(&mut ui, 0.75, "Procurando pasta mugen/…").await;
        #[cfg(not(target_arch = "wasm32"))]
        reg.scan_folders();
        reg.dedupe();
        reg.sort_mugen();
        Self::boot(&mut ui, 0.9, "Áudio…").await;
        let audio = Audio::new(audio_on).await;
        Self::boot(&mut ui, 1.0, "Pronto!").await;
        App {
            reg, market: crate::market::Market::open(), market_ui: market::MarketUi::default(), ui, g: Gfx::new(), c: Controls::new(touch), audio, screen: Screen::Title, mode: Mode::Cpu,
            ai_level: settings.difficulty, sel: (0, 1, 0), session: None, arcade: None, pause_cur: 0, debug_boxes: false, quit: false, msg: String::new(), preview_t: 0.0,
            settings, key_edit: None, move_page: 0, pending: Pending::default(), team: None, last_fight: None, replaying: None, team_n: 3, team_pick: vec![], sel_filter: 0, ending_shown: false, char_tint: WHITE, sel_row: 0, sel_last: [usize::MAX; 2], sel_dwell: 0.0, sel_buf: String::new(), names_up: vec![], fit: Default::default(), sel_cur: [(-1.0, 0.0); 2], sel_flash: (0, 0.0), stage_row: 0, quality_check: 0.0, trans: 0.0, screen_t: 0.0, menu_art: crate::menu_art::MenuArt::new(), last_disc: None, applied: None, settings_back: None, fight_rules: Rules::default(),
            updater: crate::update::Checker::start(&std::env::args().collect::<Vec<_>>()),
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
    fn toggle_fullscreen(&mut self) { self.settings.fullscreen = !self.settings.fullscreen; self.settings.apply_window(); self.settings.save(); }

    fn clicked_rect(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        if !is_mouse_button_pressed(MouseButton::Left) { return false; }
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
        if self.ai_level != self.settings.difficulty { self.settings.difficulty = self.ai_level; self.settings.save(); }
        if self.applied != Some(self.settings) {
            self.settings.apply_runtime();
            self.audio.set_levels(self.settings.master_gain(), self.settings.music_gain(), self.settings.sfx_gain());
            self.applied = Some(self.settings);
        }
        self.g.post.level = self.settings.fx as u8;
        self.g.fx_gain = IMPACT[self.settings.impact.min(IMPACT.len() - 1)].0;
        self.c.update();
        self.preview_t += get_frame_time();
        let dt = get_frame_time();
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
            Screen::Mugen => self.mugen_screen(),
            Screen::Market => self.market_screen(),
            Screen::Cutscene { .. } => self.cutscene(),
        }
        if is_key_pressed(KeyCode::F11) { self.toggle_fullscreen(); }
        if is_key_pressed(KeyCode::F3) { self.settings.fps = !self.settings.fps; self.settings.save(); }
        // auto art quality follows the display (checked outside fights so live sprites are never swapped)
        self.quality_check += dt;
        if self.quality_check > 0.7 && matches!(self.screen, Screen::Menu { .. } | Screen::Title | Screen::Settings { from_fight: false, .. } | Screen::Select { .. } | Screen::Stages { .. }) { self.quality_check = 0.0; self.apply_quality(); }
        if self.settings.fps { self.ui.text(&format!("{} FPS · {}×{} · arte {:.1}×", get_fps(), (screen_width() * dpi_scale()) as i32, Settings::physical_h() as i32, ume_engine::native::rig::art_res()), 10.0, 20.0, 16.0, hex("#6dff9e"), Align::Left, false); }
        // screen transition: diagonal wipe whenever the screen changes (not into/out of the fight itself)
        let disc = std::mem::discriminant(&self.screen);
        if self.last_disc != Some(disc) {
            if self.last_disc.is_some() && !matches!(self.screen, Screen::Fight) { self.trans = 1.0; }
            self.last_disc = Some(disc);
            self.screen_t = 0.0;
        }
        self.screen_t += dt;
        self.menu_art.tick(dt);
        if self.trans > 0.0 { self.ui.wipe(self.trans); self.trans = (self.trans - dt * 2.6).max(0.0); }
        self.audio.process().await;
        self.market.poll();
        if self.market.has_registry_work() && matches!(self.screen, Screen::Title | Screen::Menu { .. } | Screen::Market | Screen::Settings { .. } | Screen::Controls | Screen::Mugen) && self.market.apply_to(&mut self.reg) {
            self.after_registry_change();
        }
        crate::set_import_button(!matches!(self.screen, Screen::Fight | Screen::Vs { .. } | Screen::Results { .. } | Screen::Cutscene { .. }));
        for zip in crate::take_imports() {
            match self.reg.import_zip(&zip) {
                Ok(r) if r.duplicate_pack || (r.chars + r.stages == 0 && r.errors == 0) => self.msg = "Pacote já importado".into(),
                Ok(r) => { self.reg.sort_mugen(); self.sel = (self.sel.0.min(self.reg.chars.len() - 1), self.sel.1.min(self.reg.chars.len() - 1), self.sel.2.min(self.reg.stages.len() - 1)); self.msg = format!("Importado: {} personagem(ns), {} cenário(s){}", r.chars, r.stages, if r.errors == 0 { String::new() } else { format!(" · {} erro(s)", r.errors) }); }
                Err(e) => self.msg = format!("Zip inválido: {e}"),
            }
        }
        if !self.msg.is_empty() {
            self.ui.rect(0.0, VH - 36.0, VW, 36.0, Color::new(0.0, 0.0, 0.0, 0.7));
            self.ui.text(&self.msg, VW / 2.0, VH - 12.0, 18.0, hex("#6dff9e"), Align::Center, false);
            if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Left) { self.msg.clear(); }
        }
    }

    fn after_registry_change(&mut self) {
        let (nc, ns) = (self.reg.chars.len().max(1), self.reg.stages.len().max(1));
        self.sel = (self.sel.0.min(nc - 1), self.sel.1.min(nc - 1), self.sel.2.min(ns - 1));
        self.arcade = None;
        self.team_pick.clear();
        self.last_fight = None;
        self.replaying = None;
        self.sel_last = [usize::MAX; 2];
        self.sel_cur = [(-1.0, 0.0); 2];
        self.sel_row = 0;
        self.stage_row = 0;
    }
    fn menu_art_fade(&self) -> f32 { self.menu_art.fade_in() }

    fn go(&mut self, s: Screen) { self.audio.sfx("confirm"); self.screen = s; }

    // ------------------------------------------------------------ title / menu
    fn title(&mut self) {
        self.menu_art.select(0);
        self.menu_art.draw(&self.ui);
        self.ui.logo(VW / 2.0, 330.0, 200.0);
        self.ui.text("ENGINE DE LUTA · RUST · DESKTOP · WEB · MOBILE", VW / 2.0, 420.0, 20.0, hex("#a49cc0"), Align::Center, true);
        if (self.ui.t * 2.0) as i32 % 2 == 0 { self.ui.text_bold("PRESSIONE START / ENTER", VW / 2.0, 540.0, 30.0, WHITE, hex("#12051f"), Align::Center); }
        self.ui.text("Compatível com personagens M.U.G.E.N  ·  DualShock 4 / DualSense com vibração", VW / 2.0, 690.0, 18.0, hex("#a49cc0"), Align::Center, false);
        let line = match self.updater.poll() {
            Some(u) => format!("{} · nova versão v{} disponível: {}", crate::update::build_label(), u.version, crate::update::RELEASES_URL),
            None => crate::update::build_label(),
        };
        self.ui.text(&line, VW - 14.0, VH - 12.0, 14.0, hex("#a49cc0"), Align::Right, false);
        if self.c.pressed(nav::OK | nav::START) || is_mouse_button_pressed(MouseButton::Left) || !touches().is_empty() && touches()[0].phase == TouchPhase::Started { self.go(Screen::Menu { cur: 0 }); }
    }

    fn menu(&mut self) {
        let Screen::Menu { cur } = &mut self.screen else { return };
        let mut cur_v = *cur;
        if self.c.pressed(nav::DOWN) { cur_v = (cur_v + 1) % MENU.len(); self.audio.sfx("select"); }
        if self.c.pressed(nav::UP) { cur_v = (cur_v + MENU.len() - 1) % MENU.len(); self.audio.sfx("select"); }
        if self.c.pressed(nav::LEFT) { self.ai_level = (self.ai_level - 1).max(1); }
        if self.c.pressed(nav::RIGHT) { self.ai_level = (self.ai_level + 1).min(8); }
        let mut chosen = if self.c.pressed(nav::OK | nav::START) { Some(cur_v) } else { None };
        self.menu_art.select(cur_v);
        self.menu_art.draw(&self.ui);
        let ui = &self.ui;
        let accent = crate::menu_art::accent_of(cur_v);
        crate::menu_art::glass(ui, -400.0, 0.0, 400.0 + 600.0, VH, 1.0);
        ui.rect(-400.0, 0.0, 400.0 + 600.0, VH, Color::new(0.0, 0.0, 0.0, 0.18));
        ui.rect(600.0, 0.0, 3.0, VH, Color::new(accent.r, accent.g, accent.b, 0.8));
        ui.logo(250.0, 118.0, 74.0);
        let n = MENU.len();
        let step = 456.0 / n as f32;
        for (i, (t, _)) in MENU.iter().enumerate() {
            let k_in = ((self.screen_t - 0.05 - i as f32 * 0.04) / 0.28).clamp(0.0, 1.0);
            let k_in = 1.0 - (1.0 - k_in).powi(3);
            let (x, y, w, h) = (40.0 - (1.0 - k_in) * 120.0, 196.0 + i as f32 * step, 520.0, step - 4.0);
            if self.hover_rect(x, y, w, h) && (mouse_delta_position().length() > 0.0) { cur_v = i; }
            if self.clicked_rect(x, y, w, h) { cur_v = i; chosen = Some(i); }
            let f = i == cur_v;
            let off = if f { 22.0 } else { 0.0 };
            if f {
                let pulse = 0.8 + 0.2 * (ui.t * 5.0).sin();
                ui.quad([vec2(x + off + 18.0, y), vec2(x + off + w, y), vec2(x + off + w - 18.0, y + h), vec2(x + off, y + h)], Color::new(accent.r * pulse, accent.g * pulse, accent.b * pulse, 0.92 * k_in));
                ui.quad([vec2(x + off - 14.0, y), vec2(x + off - 4.0, y), vec2(x + off - 18.0, y + h), vec2(x + off - 28.0, y + h)], Color::new(1.0, 1.0, 1.0, k_in));
                ui.text(t, x + off + 30.0, y + h * 0.72, 24.0, hex("#12051f"), Align::Left, true);
            } else {
                ui.quad([vec2(x + 18.0, y), vec2(x + w - 80.0, y), vec2(x + w - 98.0, y + h), vec2(x, y + h)], Color::new(1.0, 1.0, 1.0, 0.07 * k_in));
                ui.text(t, x + 30.0, y + h * 0.72, 22.0, Color::new(1.0, 1.0, 1.0, 0.82 * k_in), Align::Left, true);
            }
        }
        let (title, desc) = MENU[cur_v];
        let ta = ((self.menu_art_fade()) * 1.0).clamp(0.0, 1.0);
        ui.text_bold(&title.to_uppercase(), VW - 60.0, 590.0, if title.chars().count() > 12 { 44.0 } else { 64.0 }, Color::new(1.0, 1.0, 1.0, ta), Color::new(0.05, 0.02, 0.1, ta), Align::Right);
        ui.rect(VW - 460.0, 608.0, 400.0, 4.0, Color::new(accent.r, accent.g, accent.b, ta));
        ui.text(desc, VW - 60.0, 646.0, 22.0, Color::new(1.0, 1.0, 1.0, 0.9 * ta), Align::Right, false);
        let (nm, nn) = (self.reg.chars.iter().filter(|c| c.mugen).count(), self.playable().len());
        let stats = format!("{nn} lutadores · {} cenários · {nm} MUGEN", self.reg.stages.len());
        ui.text(&stats, VW - 40.0, 44.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.85), Align::Right, false);
        if cur_v <= 6 {
            ui.text("Dificuldade CPU (esq./dir.)", VW - 40.0, 86.0, 16.0, hex("#d8d0f0"), Align::Right, false);
            for i in 0..8 { ui.rect(VW - 40.0 - 8.0 * 26.0 + i as f32 * 26.0, 96.0, 20.0, 12.0, if i < self.ai_level { accent } else { Color::new(1.0, 1.0, 1.0, 0.18) }); }
        }
        ui.text(&crate::update::build_label(), 40.0, VH - 20.0, 16.0, hex("#a49cc0"), Align::Left, false);
        let pads = &self.c.pad_info;
        let pad_line = if pads.is_empty() { "teclado".to_string() } else { format!("{} controle(s)", pads.len()) };
        ui.text(&pad_line, 560.0, VH - 20.0, 16.0, hex("#a49cc0"), Align::Right, false);
        self.screen = Screen::Menu { cur: cur_v };
        if self.c.pressed(nav::BACK) { self.screen = Screen::Title; }
        if let Some(i) = chosen {
            let mode = [Mode::Arcade, Mode::Cpu, Mode::Versus, Mode::Training, Mode::Watch, Mode::TeamSeq, Mode::TeamSimul];
            match i {
                0..=6 => { self.mode = mode[i]; self.team = None; self.team_pick.clear(); if self.mode == Mode::TeamSimul { self.team_n = 5; } else if self.mode == Mode::TeamSeq { self.team_n = 3; } self.go(Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }); }
                7 => self.go(Screen::Mugen),
                8 => self.go(Screen::Market),
                9 => { self.settings_back = None; self.go(Screen::Controls) }
                10 => self.go(Screen::Settings { tab: 0, cur: 0, from_fight: false }),
                _ => self.quit = true,
            }
        }
    }

    // ------------------------------------------------------------ select
    pub fn draw_char(&mut self, d: &Arc<CharDef>, anim: i32, t: f32, x: f32, y: f32, height_px: f32, mirror: bool) {
        let base_h = (d.c("size.height", 60.0) as f32 * 1.7).max(90.0);
        self.draw_char_k(d, anim, t, x, y, height_px / base_h, mirror);
    }

    fn draw_char_k(&mut self, d: &Arc<CharDef>, anim: i32, t: f32, x: f32, y: f32, kv: f32, mirror: bool) {
        let Some(a) = d.anims.get(&anim).or(d.anims.get(&0)) else { return };
        if a.frames.is_empty() { return; }
        let tot = a.total_time.max(1);
        let tm = ((t * 60.0) as i32) % tot;
        let (mut acc, mut idx) = (0, 0);
        for (i, f) in a.frames.iter().enumerate() { idx = i; if f.time < 0 || tm < acc + f.time { break; } acc += f.time; }
        let fr = &a.frames[idx];
        let Some(spr) = d.sprites.get(&(fr.group, fr.index)) else { return };
        let Some(img) = spr.image.clone() else { return };
        let tex = self.g.texture(&img, spr.scale >= 0.99);
        let l = 320.0 / d.local_w;
        let k = kv * self.ui.s;
        let ks = k * spr.scale * l;
        let flip = mirror ^ fr.flip_h;
        let w = spr.width * ks; let h = spr.height * ks;
        let px = self.ui.x(x) + fr.x * l * k * if mirror { -1.0 } else { 1.0 };
        let py = self.ui.y(y) + fr.y * l * k;
        let left = if flip { px - (spr.width - spr.axis_x) * ks } else { px - spr.axis_x * ks };
        draw_texture_ex(&tex, left, py - spr.axis_y * ks, self.char_tint, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_x: flip, ..Default::default() });
    }

    fn char_extent(&mut self, d: &Arc<CharDef>) -> [f32; 4] {
        if let Some(e) = self.fit.get(d.id.as_str()) { return *e; }
        let l = 320.0 / d.local_w;
        let mut e = [f32::MAX, f32::MIN, f32::MAX, f32::MIN];
        if let Some(a) = d.anims.get(&0) {
            for fr in &a.frames {
                let Some(s) = d.sprites.get(&(fr.group, fr.index)) else { continue };
                let sc = s.scale * l;
                let (ox, oy, ow, oh) = opaque_box(s);
                let (top, left) = (fr.y * l + (oy - s.axis_y) * sc, if fr.flip_h { fr.x * l + (s.width - ow - ox - s.axis_x) * sc } else { fr.x * l + (ox - s.axis_x) * sc });
                e = [e[0].min(top), e[1].max(top + oh * sc), e[2].min(left), e[3].max(left + ow * sc)];
            }
        }
        if e[0] >= e[1] || e[2] >= e[3] { e = [-150.0, 0.0, -60.0, 60.0]; }
        self.fit.insert(d.id.clone(), e);
        e
    }

    /// Draws a fighter scaled so its idle bounding box fits `fw` x `fh`, feet on `bottom`, centred on `cx`.
    pub fn draw_char_fit(&mut self, d: &Arc<CharDef>, anim: i32, t: f32, cx: f32, bottom: f32, fw: f32, fh: f32, mirror: bool) {
        let e = self.char_extent(d);
        let k = (fh / (e[1] - e[0])).min(fw / (e[3] - e[2])).min(6.0);
        let sgn = if mirror { -1.0 } else { 1.0 };
        self.draw_char_k(d, anim, t, cx - sgn * (e[2] + e[3]) * 0.5 * k, bottom - e[1] * k, k, mirror);
    }

    /// Everyone you can pick (the arcade final boss stays hidden).
    fn playable(&self) -> Vec<usize> { (0..self.reg.chars.len()).filter(|&i| !self.reg.chars[i].boss).collect() }
    /// Registry index of the arcade final boss, if present.
    fn boss_idx(&self) -> Option<usize> { (0..self.reg.chars.len()).find(|&i| self.reg.chars[i].boss && !self.reg.chars[i].broken.get()) }

    /// Registry indices visible under the current filter (never empty).
    fn char_view(&self) -> Vec<usize> {
        let v: Vec<usize> = self.playable().into_iter().filter(|&i| match self.sel_filter { 1 => !self.reg.chars[i].mugen, 2 => self.reg.chars[i].mugen, _ => true }).collect();
        if v.is_empty() { self.playable() } else { v }
    }

    fn select(&mut self) {
        let Screen::Select { mut cur, mut picking, mut picked } = self.screen else { return };
        let dt = get_frame_time();
        let mut view = self.char_view();
        if self.sel_filter != 0 && view.len() == self.playable().len() { self.sel_filter = 0; }
        let m = view.len();
        let pos_of = |view: &[usize], i: usize| view.iter().position(|&v| v == i).unwrap_or(0);
        let cols = m.div_ceil(VIEW_ROWS).clamp(6, 9);
        let total_rows = m.div_ceil(cols);
        let two = self.mode == Mode::Versus;
        let mut confirm = false;
        let p = picking;
        let mut pos = pos_of(&view, cur[p]);
        let old = pos;
        if self.c.pressed(nav::RIGHT) { pos = (pos + 1) % m; }
        if self.c.pressed(nav::LEFT) { pos = (pos + m - 1) % m; }
        if self.c.pressed(nav::DOWN) { pos = if pos + cols < m { pos + cols } else { pos % cols }; }
        if self.c.pressed(nav::UP) { pos = if pos >= cols { pos - cols } else { let mut q = (total_rows - 1) * cols + pos; if q >= m { q -= cols; } q }; }
        if is_key_pressed(KeyCode::PageDown) { pos = (pos + cols * VIEW_ROWS).min(m - 1); }
        if is_key_pressed(KeyCode::PageUp) { pos = pos.saturating_sub(cols * VIEW_ROWS); }
        if is_key_pressed(KeyCode::Home) { pos = 0; }
        if is_key_pressed(KeyCode::End) { pos = m - 1; }
        let wheel = mouse_wheel().1;
        if wheel > 0.0 { pos = pos.saturating_sub(cols); } else if wheel < 0.0 { pos = (pos + cols).min(m - 1); }
        if is_key_pressed(KeyCode::R) { pos = (get_time() * 1000.0) as usize % m; }
        if is_key_pressed(KeyCode::Tab) {
            self.sel_filter = (self.sel_filter + 1) % 3;
            view = self.char_view();
            if view.len() == self.playable().len() { self.sel_filter = 0; }
            pos = pos_of(&view, cur[p]).min(view.len() - 1);
            self.sel_row = 0;
            self.audio.sfx("select");
            // re-enter with the new view next frame (geometry depends on it)
            cur[p] = view[pos];
            self.screen = Screen::Select { cur, picking, picked };
            return;
        }
        if pos != old { cur[p] = view[pos]; self.audio.sfx("select"); }
        if self.c.pressed(nav::OK | nav::START) { confirm = true; }
        let back = self.c.pressed(nav::BACK);
        // keep the cursor row on screen
        let crow = pos / cols;
        if crow < self.sel_row { self.sel_row = crow; }
        if crow >= self.sel_row + VIEW_ROWS { self.sel_row = crow + 1 - VIEW_ROWS; }
        self.sel_row = self.sel_row.min(total_rows.saturating_sub(VIEW_ROWS));

        self.ui.background();
        if self.mode.is_team() {
            if is_key_pressed(KeyCode::E) { self.team_n = (self.team_n + 1).min(5); self.audio.sfx("select"); }
            if is_key_pressed(KeyCode::Q) { self.team_n = (self.team_n - 1).max(2).max(self.team_pick.len() + 1); self.audio.sfx("select"); }
        }
        if self.names_up.len() != self.reg.chars.len() { self.names_up = self.reg.chars.iter().map(|c| c.name.to_uppercase()).collect(); }
        let ns = self.reg.stages.len();
        if ns > 0 {
            if is_key_pressed(KeyCode::RightBracket) || self.clicked_rect(742.0, 10.0, 38.0, 62.0) { self.sel.2 = (self.sel.2 + 1) % ns; self.audio.sfx("select"); }
            if is_key_pressed(KeyCode::LeftBracket) || self.clicked_rect(500.0, 10.0, 38.0, 62.0) { self.sel.2 = (self.sel.2 + ns - 1) % ns; self.audio.sfx("select"); }
            self.sel.2 = self.sel.2.min(ns - 1);
        }
        let accents = [hex("#3fa9ff"), hex("#ff4d6d")];
        let active = [picking == 0 || picked[0].is_some(), picking == 1];
        let s_ui = self.ui.s;
        for side in 0..2 {
            let Some(i) = picked[side].or(if side == picking { Some(cur[side]) } else { None }) else { continue };
            let c = rgb(self.reg.chars[i].color);
            glow(self.ui.x(if side == 0 { 190.0 } else { VW - 190.0 }), self.ui.y(430.0), 420.0 * s_ui, Color::new(c.r, c.g, c.b, 0.5));
        }

        crate::menu_art::glass(&self.ui, 0.0, 0.0, VW, 84.0, 1.0);
        self.ui.rect(0.0, 82.0, VW, 2.0, alpha(accents[picking.min(1)], 0.85));
        let team_mode = self.mode.is_team();
        self.sel_buf.clear();
        if team_mode { let _ = write!(self.sel_buf, "{} {}v{}", self.mode.label(), self.team_n, self.team_n); } else { self.sel_buf.push_str(self.mode.label()); }
        self.ui.text(&self.sel_buf, 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let hint = if team_mode { "Monte sua equipe · Q/E: tamanho" } else if picking == 0 { "JOGADOR 1: escolha seu lutador" } else if two { "JOGADOR 2: escolha" } else { "Escolha o oponente" };
        self.ui.text(hint, VW - 40.0, 56.0, 20.0, hex("#d8d0f0"), Align::Right, false);
        if ns > 0 {
            let (sx, sy, sw, sh) = (500.0, 10.0, 280.0, 62.0);
            crate::menu_art::glass(&self.ui, sx, sy, sw, sh, 1.0);
            self.ui.rect_lines(sx, sy, sw, sh, 1.0, Color::new(1.0, 1.0, 1.0, 0.18));
            if let Some(image) = self.reg.stage_preview(self.sel.2) {
                let texture = self.g.texture(&image, true);
                let (iw, ih) = (image.w as f32, image.h as f32);
                let ch_src = (iw * 50.0 / 72.0).min(ih);
                let cw_src = ch_src * 72.0 / 50.0;
                draw_texture_ex(&texture, self.ui.x(sx + 44.0), self.ui.y(sy + 6.0), WHITE, DrawTextureParams { dest_size: Some(vec2(72.0 * s_ui, 50.0 * s_ui)), source: Some(Rect::new((iw - cw_src) * 0.5, (ih - ch_src) * 0.5, cw_src, ch_src)), ..Default::default() });
            }
            let ar = |this: &Self, x: f32, dir: f32| draw_triangle(vec2(this.ui.x(x + dir * 7.0), this.ui.y(sy + sh * 0.5)), vec2(this.ui.x(x - dir * 5.0), this.ui.y(sy + sh * 0.5 - 9.0)), vec2(this.ui.x(x - dir * 5.0), this.ui.y(sy + sh * 0.5 + 9.0)), hex("#ffcc33"));
            ar(self, sx + 18.0, -1.0);
            ar(self, sx + sw - 18.0, 1.0);
            self.ui.text("CENÁRIO", sx + 126.0, sy + 22.0, 12.0, hex("#a49cc0"), Align::Left, true);
            let stage_name = &self.reg.stages[self.sel.2].name;
            self.ui.text(clip(stage_name, 15), sx + 126.0, sy + 46.0, 15.0, WHITE, Align::Left, true);
        }

        let focus = cur[picking];
        if self.sel_last[picking] != focus { self.sel_last[picking] = focus; self.sel_dwell = 0.0; } else { self.sel_dwell += dt; }
        let pal_of = |side: usize, i: usize, picked: [Option<usize>; 2]| if side == 1 && picked[0] == Some(i) { 2 } else { 1 };
        let mut want = [(focus, pal_of(picking, focus, picked)), (usize::MAX, 0), (usize::MAX, 0)];
        for side in 0..2 { if let Some(i) = picked[side] { want[side + 1] = (i, pal_of(side, i, picked)); } }
        let mut loaded_one = false;
        for (i, pal) in want {
            if i == usize::MAX { continue; }
            let e = &self.reg.chars[i];
            if e.peek(pal).is_some() || e.broken.get() { continue; }
            if self.sel_dwell < if e.mugen { 0.18 } else { 0.03 } { continue; }
            if e.load_preview(pal).is_err() { self.msg = format!("Falha ao carregar {}", self.reg.chars[i].name); }
            loaded_one = true;
        }

        let gap = 6.0;
        let cw = ((640.0 - (cols as f32 - 1.0) * gap) / cols as f32).min(96.0);
        let ch = cw * 1.17;
        let gw = cols as f32 * cw + (cols as f32 - 1.0) * gap;
        let gx = (VW - gw) / 2.0;
        let gy = 110.0;
        let gh = VIEW_ROWS as f32 * (ch + gap) - gap;
        crate::menu_art::glass(&self.ui, gx - 12.0, gy - 12.0, gw + 24.0, gh + 24.0, 1.0);
        self.ui.rect_lines(gx - 12.0, gy - 12.0, gw + 24.0, gh + 24.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
        let mut click = None;
        let first = self.sel_row * cols;
        let last = ((self.sel_row + VIEW_ROWS) * cols).min(m);
        for k in first..last {
            let i = view[k];
            let (cx, cy) = (gx + (k % cols) as f32 * (cw + gap), gy + ((k / cols) - self.sel_row) as f32 * (ch + gap));
            if self.clicked_rect(cx, cy, cw, ch) { click = Some(i); }
            let (ecol, emugen, broken) = { let e = &self.reg.chars[i]; (e.color, e.mugen, e.broken.get()) };
            let col = rgb(ecol);
            self.ui.rect(cx, cy, cw, ch, Color::new(0.06, 0.04, 0.12, 0.92));
            self.ui.rect(cx, cy, cw, ch * 0.62, alpha(col, 0.5));
            self.ui.rect(cx, cy + ch * 0.62, cw, ch * 0.38, alpha(col, 0.18));
            let mut got = self.reg.chars[i].peek(1);
            if got.is_none() && !emugen && !loaded_one && !broken { got = self.reg.chars[i].load_preview(1).ok(); loaded_one = got.is_some(); }
            if emugen && !loaded_one && !broken && self.sel_dwell > 0.15 && !self.reg.chars[i].has_portrait_attempt() { self.reg.chars[i].ensure_portrait(); loaded_one = true; }
            let thumb = if emugen { self.reg.chars[i].portrait() } else { got.as_ref().and_then(|d| d.sprites.get(&(9000, 1)).cloned()) };
            if thumb.is_some() || got.is_some() {
                let por = thumb.and_then(|p| p.image.clone().map(|im| (im, p.width, p.height)));
                if let Some((im, pw, ph)) = por {
                    let tex = self.g.texture(&im, true);
                    let kk = ((cw - 8.0) / pw).min((ch - 26.0) / ph);
                    draw_texture_ex(&tex, self.ui.x(cx + (cw - pw * kk) / 2.0), self.ui.y(cy + 4.0), WHITE, DrawTextureParams { dest_size: Some(vec2(pw * kk * self.ui.s, ph * kk * self.ui.s)), ..Default::default() });
                } else if let Some(d) = got {
                    self.ui_clip(cx, cy, cw, ch - 18.0, |app| app.draw_char_fit(&d, 0, 0.0, cx + cw / 2.0, cy + ch - 20.0, cw - 8.0, ch - 30.0, false));
                }
            } else {
                let init = self.names_up[i].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
                self.ui.text(if broken { "X" } else { &self.names_up[i][..init] }, cx + cw / 2.0, cy + ch * 0.58, cw * 0.55, Color::new(1.0, 1.0, 1.0, if broken { 0.5 } else { 0.28 }), Align::Center, true);
            }
            self.ui.rect(cx, cy + ch - 18.0, cw, 18.0, Color::new(0.0, 0.0, 0.0, 0.7));
            let up = self.names_up[i].as_str();
            let nm = if emugen { clip(up, 13) } else { clip(up.split_whitespace().next().unwrap_or(""), 12) };
            self.ui.text(nm, cx + cw / 2.0, cy + ch - 4.0, (cw * 0.125).min(12.0), WHITE, Align::Center, true);
            if emugen { self.ui.rect(cx + cw - 44.0, cy + 3.0, 41.0, 13.0, hex("#22e8ff")); self.ui.text("MUGEN", cx + cw - 23.5, cy + 14.0, 10.0, BLACK, Align::Center, true); }
            let cursored = (cur[0] == i && active[0]) || (cur[1] == i && active[1]);
            if !cursored { self.ui.rect(cx, cy, cw, ch, Color::new(0.0, 0.0, 0.08, 0.25)); }
            for side in 0..2 {
                if picked[side] == Some(i) { self.ui.rect(cx, cy, 24.0, 14.0, accents[side]); self.ui.text(if side == 0 { "P1" } else { "P2" }, cx + 12.0, cy + 12.0, 11.0, BLACK, Align::Center, true); }
            }
            if let Some(n) = self.team_pick.iter().position(|&t| t == i) {
                self.ui.rect(cx, cy, 20.0, 18.0, hex("#ffcc33"));
                self.ui.text(NUMS[n.min(4)], cx + 10.0, cy + 14.0, 14.0, BLACK, Align::Center, true);
            }
        }
        if total_rows > VIEW_ROWS {
            let bx = gx + gw + 3.0;
            self.ui.rect(bx, gy, 5.0, gh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (gh * VIEW_ROWS as f32 / total_rows as f32).max(18.0);
            self.ui.rect(bx, gy + (gh - th) * self.sel_row as f32 / (total_rows - VIEW_ROWS) as f32, 5.0, th, hex("#ff4d6d"));
        }
        let pulse = 0.5 + 0.5 * (self.ui.t * 7.0).sin();
        for side in 0..2 {
            if !active[side] { continue; }
            let kpos = pos_of(&view, cur[side]);
            if kpos < first || kpos >= last { continue; }
            let target = (gx + (kpos % cols) as f32 * (cw + gap), gy + ((kpos / cols) - self.sel_row) as f32 * (ch + gap));
            let c = &mut self.sel_cur[side];
            if c.0 < 0.0 { *c = target; } else { let f = 1.0 - (-dt * 24.0).exp(); c.0 += (target.0 - c.0) * f; c.1 += (target.1 - c.1) * f; }
            let (rx, ry) = (self.sel_cur[side].0, self.sel_cur[side].1);
            let a = accents[side];
            let pad = 2.0 + 2.0 * pulse + side as f32 * 3.0;
            self.ui.rect(rx - pad, ry - pad, cw + pad * 2.0, ch + pad * 2.0, alpha(a, 0.10 + 0.08 * pulse));
            self.ui.rect_lines(rx - pad, ry - pad, cw + pad * 2.0, ch + pad * 2.0, 2.0, alpha(a, 0.9));
            let (bl, bt) = (16.0, 4.0);
            for (ox, oy, dx, dy) in [(rx - pad, ry - pad, 1.0, 1.0), (rx + cw + pad, ry - pad, -1.0, 1.0), (rx - pad, ry + ch + pad, 1.0, -1.0), (rx + cw + pad, ry + ch + pad, -1.0, -1.0)] {
                self.ui.rect(ox.min(ox + dx * bl), oy + if dy > 0.0 { -2.0 } else { 2.0 - bt }, bl, bt, WHITE);
                self.ui.rect(ox + if dx > 0.0 { -2.0 } else { 2.0 - bt }, oy.min(oy + dy * bl), bt, bl, WHITE);
            }
            if picked[side].is_none() { self.ui.rect(rx - pad, ry - pad - 15.0, 26.0, 15.0, a);
            self.ui.text(if side == 0 { "P1" } else if two { "P2" } else { "CPU" }, rx - pad + 13.0, ry - pad - 3.0, 12.0, BLACK, Align::Center, true); }
        }
        if let Some(i) = click { if cur[picking] == i { confirm = true; } else { cur[picking] = i; self.audio.sfx("select"); } }

        for side in 0..2 {
            let idx = picked[side].or(if side == picking { Some(cur[side]) } else { None });
            let label = if side == 0 { "P1" } else if two { "P2" } else { "CPU" };
            let pal = idx.map(|i| pal_of(side, i, picked)).unwrap_or(1);
            self.select_side(side, idx, picked[side].is_some(), side == picking, pal, label);
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
                        self.ui.rect(x, y, sw, 46.0, alpha(rgb(self.reg.chars[i].color), 0.5));
                        self.portrait_fit(i, x + 2.0, y + 2.0, 42.0, 42.0);
                        self.ui.text(clip(self.names_up[i].split_whitespace().next().unwrap_or(""), 7), x + 46.0, y + 28.0, if n > 4 { 11.0 } else { 13.0 }, WHITE, Align::Left, true);
                    }
                    None => {
                        let now = k == self.team_pick.len();
                        if now { self.ui.rect(x, y, sw, 46.0, Color::new(1.0, 0.8, 0.2, 0.14 + 0.12 * pulse)); self.ui.rect_lines(x, y, sw, 46.0, 2.0, hex("#ffcc33")); }
                        self.ui.text(NUMS[k.min(4)], x + sw / 2.0, y + 31.0, 20.0, Color::new(1.0, 1.0, 1.0, if now { 0.9 } else { 0.3 }), Align::Center, true);
                    }
                }
            }
        }

        crate::menu_art::glass(&self.ui, 0.0, 652.0, VW, 68.0, 1.0);
        self.ui.rect(0.0, 652.0, VW, 1.5, alpha(accents[picking.min(1)], 0.7));
        let desc = &self.reg.chars[cur[picking]].description;
        self.ui.text(desc, VW / 2.0, 680.0, 18.0, hex("#d8d0f0"), Align::Center, false);
        let flabel = ["TODOS", "ORIGINAIS", "M.U.G.E.N"][self.sel_filter as usize % 3];
        self.sel_buf.clear();
        let _ = write!(self.sel_buf, "[Tab] {flabel} · {}/{m} · [R] aleatório · [ ] cenário · PgUp/PgDn", pos + 1);
        self.ui.text(&self.sel_buf, VW / 2.0, 706.0, 14.0, hex("#a49cc0"), Align::Center, false);
        // bound memory with hundreds of fighters
        let mut keep: Vec<usize> = cur.to_vec(); keep.extend(picked.iter().flatten());
        self.reg.trim_cache(&keep);
        if confirm {
            if self.reg.chars[cur[picking]].broken.get() { self.msg = "Este personagem não pôde ser carregado.".into(); self.screen = Screen::Select { cur, picking, picked }; return; }
            self.audio.sfx("confirm");
            self.sel_flash = (picking, 1.0);
            if self.mode.is_team() {
                let me = cur[0];
                if self.team_pick.contains(&me) { self.msg = "Este lutador já está na equipe.".into(); self.screen = Screen::Select { cur, picking: 0, picked: [None, None] }; return; }
                self.team_pick.push(me);
                let n = self.team_n;
                if self.team_pick.len() < n { self.screen = Screen::Select { cur, picking: 0, picked: [None, None] }; return; }
                // opponents: random distinct fighters (originals first), then straight to the stage
                let mut pool: Vec<usize> = (0..self.reg.chars.len()).filter(|i| !self.team_pick.contains(i) && !self.reg.chars[*i].broken.get() && !self.reg.chars[*i].boss).collect();
                let seed = (get_time() * 1000.0) as usize;
                if !pool.is_empty() { let l = pool.len(); pool.rotate_left(seed % l); }
                pool.sort_by_key(|&i| self.reg.chars[i].mugen as u8);
                let mut foes: Vec<usize> = pool.into_iter().take(n).collect();
                while foes.len() < n { foes.push(self.team_pick[foes.len() % self.team_pick.len()]); }
                let t = Team::new(self.team_pick.clone(), foes, self.mode == Mode::TeamSimul);
                self.sel = (t.members[0][0], t.members[1][0], self.sel.2);
                self.team = Some(t);
                self.screen = Screen::Stages { cur: 1 };
                return;
            }
            picked[picking] = Some(cur[picking]);
            if picking == 0 && self.mode == Mode::Arcade {
                let me = cur[0];
                let n = self.reg.chars.len();
                let mut ladder: Vec<usize> = (0..n).filter(|&i| i != me && !self.reg.chars[i].broken.get() && !self.reg.chars[i].boss).collect();
                let seed = (get_time() * 1000.0) as usize;
                let ll = ladder.len().max(1);
                ladder.rotate_left(seed % ll);
                // originals first (guaranteed to work), MUGEN fighters mixed in later rounds
                ladder.sort_by_key(|&i| self.reg.chars[i].mugen as u8);
                ladder.truncate(8);
                if ladder.is_empty() { ladder.push(me); }
                // the ladder ends with the hidden final boss
                let has_boss = self.boss_idx().filter(|&b| b != me).map(|b| ladder.push(b)).is_some();
                self.sel = (me, ladder[0], self.sel.2);
                self.arcade = Some((ladder, 0));
                self.ending_shown = false;
                self.screen = if has_boss { Screen::Cutscene { kind: 0, page: 0, t: 0.0 } } else { Screen::Stages { cur: 1 } };
                return;
            }
            if picking == 0 { picking = 1; } else { self.sel = (picked[0].unwrap(), picked[1].unwrap(), self.sel.2); self.screen = Screen::Stages { cur: self.sel.2 + 1 }; return; }
        }
        if back {
            if self.mode.is_team() && !self.team_pick.is_empty() { self.team_pick.pop(); self.screen = Screen::Select { cur, picking: 0, picked: [None, None] }; return; }
            if picking == 1 { picking = 0; picked = [None, None]; } else { self.screen = Screen::Menu { cur: 0 }; return; }
        }
        self.screen = Screen::Select { cur, picking, picked };
    }

    fn portrait_fit(&mut self, i: usize, x: f32, y: f32, w: f32, h: f32) {
        let e = &self.reg.chars[i];
        let por = if e.mugen { e.portrait() } else { e.peek(1).and_then(|d| d.sprites.get(&(9000, 1)).cloned()) };
        let Some((im, pw, ph)) = por.and_then(|p| p.image.clone().map(|im| (im, p.width, p.height))) else { return };
        let tex = self.g.texture(&im, true);
        let kk = (w / pw).min(h / ph);
        draw_texture_ex(&tex, self.ui.x(x + (w - pw * kk) / 2.0), self.ui.y(y + (h - ph * kk) / 2.0), WHITE, DrawTextureParams { dest_size: Some(vec2(pw * kk * self.ui.s, ph * kk * self.ui.s)), ..Default::default() });
    }

    fn select_side(&mut self, side: usize, idx: Option<usize>, locked: bool, hovering: bool, pal: i32, label: &str) {
        let acc = if side == 0 { hex("#3fa9ff") } else { hex("#ff4d6d") };
        let (px, py, pw, ph) = (if side == 0 { 24.0 } else { VW - 304.0 }, 100.0, 280.0, 540.0);
        let cx = px + pw / 2.0;
        crate::menu_art::glass(&self.ui, px, py, pw, ph, 1.0);
        self.ui.rect(px, py, pw, 3.0, acc);
        self.ui.rect(if side == 0 { px } else { px + pw - 3.0 }, py, 3.0, ph, alpha(acc, 0.55));
        self.ui.rect_lines(px, py, pw, ph, 1.0, Color::new(1.0, 1.0, 1.0, 0.14));
        let s = self.ui.s;
        let Some(i) = idx else {
            self.ui.text("?", cx, py + 340.0, 220.0, Color::new(1.0, 1.0, 1.0, 0.07), Align::Center, true);
            self.ui.text(if self.mode.is_team() { "ADVERSÁRIOS SORTEADOS" } else { "AGUARDANDO" }, cx, py + ph - 24.0, 15.0, hex("#a49cc0"), Align::Center, true);
            self.ui.rect(px + 12.0, py + 12.0, 44.0, 20.0, acc);
            self.ui.text(label, px + 34.0, py + 28.0, 15.0, BLACK, Align::Center, true);
            return;
        };
        let col = rgb(self.reg.chars[i].color);
        let t = self.preview_t;
        let ease = if hovering && !locked { 1.0 - (1.0 - (self.sel_dwell / 0.3).min(1.0)).powi(3) } else { 1.0 };
        glow(self.ui.x(cx), self.ui.y(py + 340.0), 230.0 * s, alpha(col, 0.55));
        draw_ellipse(self.ui.x(cx), self.ui.y(py + 488.0), 105.0 * s, 13.0 * s, 0.0, Color::new(0.0, 0.0, 0.0, 0.5));
        draw_ellipse(self.ui.x(cx), self.ui.y(py + 488.0), 105.0 * s, 13.0 * s, 0.0, alpha(acc, 0.12));
        if let Some(d) = self.reg.chars[i].peek(pal) {
            let anim = if locked && d.anims.contains_key(&181) { 181 } else { 0 };
            self.char_tint = Color::new(1.0, 1.0, 1.0, ease);
            let slide = (1.0 - ease) * if side == 0 { -60.0 } else { 60.0 };
            self.draw_char_fit(&d, anim, t, cx + slide, py + 478.0, 270.0, 450.0, side == 1);
            self.char_tint = WHITE;
        } else if !self.reg.chars[i].broken.get() {
            self.ui.progress_bar(cx - 90.0, py + 300.0, 180.0, (self.sel_dwell / 0.4).min(0.95), "Carregando…");
        }
        if self.sel_flash.0 == side && self.sel_flash.1 > 0.0 {
            let f = self.sel_flash.1;
            self.ui.rect(px, py, pw, ph, Color::new(1.0, 1.0, 1.0, 0.45 * f * f));
            draw_circle_lines(self.ui.x(cx), self.ui.y(py + 300.0), (1.0 - f) * 320.0 * s, 4.0 * s, alpha(acc, f));
        }
        let e = &self.reg.chars[i];
        let by = py + ph - 56.0;
        self.ui.rect(px, by, pw, 56.0, Color::new(0.0, 0.0, 0.05, 0.7));
        self.ui.rect(px, by, pw, 2.0, alpha(col, 0.9));
        let nm = self.names_up[i].as_str();
        self.ui.text_bold(nm, cx, by + 28.0, if nm.chars().count() > 14 { 19.0 } else { 26.0 }, if locked { hex("#ffcc33") } else { WHITE }, BLACK, Align::Center);
        self.ui.text(&e.series, cx, by + 48.0, 14.0, hex("#a49cc0"), Align::Center, false);
        self.ui.rect(px + 12.0, py + 12.0, 44.0, 20.0, acc);
        self.ui.text(label, px + 34.0, py + 28.0, 15.0, BLACK, Align::Center, true);
        if locked { self.ui.rect(px + pw - 82.0, py + 12.0, 70.0, 20.0, hex("#ffcc33")); self.ui.text("PRONTO", px + pw - 47.0, py + 28.0, 14.0, BLACK, Align::Center, true); }
        else if e.mugen { self.ui.rect(px + pw - 62.0, py + 12.0, 50.0, 16.0, hex("#22e8ff")); self.ui.text("MUGEN", px + pw - 37.0, py + 25.0, 12.0, BLACK, Align::Center, true); }
    }

    fn ui_clip(&mut self, x: f32, y: f32, w: f32, h: f32, f: impl FnOnce(&mut App)) {
        let (sx, sy, sw, sh) = (self.ui.x(x), self.ui.y(y), w * self.ui.s, h * self.ui.s);
        set_scissor(Some((sx as i32, sy as i32, sw as i32, sh as i32)));
        f(self);
        set_scissor(None);
    }

    fn stages(&mut self) {
        let Screen::Stages { mut cur } = self.screen else { return };
        let n = self.reg.stages.len() + 1;
        let cols = 4usize;
        let total_rows = n.div_ceil(cols);
        if self.c.pressed(nav::RIGHT) { cur = (cur + 1) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::LEFT) { cur = (cur + n - 1) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::DOWN) { cur = (cur + cols).min(n - 1); }
        if self.c.pressed(nav::UP) { cur = cur.saturating_sub(cols); }
        if is_key_pressed(KeyCode::PageDown) { cur = (cur + cols * VIEW_ROWS).min(n - 1); }
        if is_key_pressed(KeyCode::PageUp) { cur = cur.saturating_sub(cols * VIEW_ROWS); }
        let wheel = mouse_wheel().1;
        if wheel > 0.0 { cur = cur.saturating_sub(cols); } else if wheel < 0.0 { cur = (cur + cols).min(n - 1); }
        let crow = cur / cols;
        if crow < self.stage_row { self.stage_row = crow; }
        if crow >= self.stage_row + VIEW_ROWS { self.stage_row = crow + 1 - VIEW_ROWS; }
        self.stage_row = self.stage_row.min(total_rows.saturating_sub(VIEW_ROWS));
        let mut ok = self.c.pressed(nav::OK | nav::START);
        self.ui.background();
        self.ui.text("ESCOLHA O CENÁRIO", 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        self.ui.text(&format!("{}/{} · PgUp/PgDn · roda do mouse", cur + 1, n), VW - 40.0, 56.0, 18.0, hex("#a49cc0"), Align::Right, false);
        let (cw, ch) = (290.0, 134.0);
        for i in (self.stage_row * cols)..((self.stage_row + VIEW_ROWS) * cols).min(n) {
            let (x, y) = (40.0 + (i % cols) as f32 * (cw + 13.0), 88.0 + ((i / cols) - self.stage_row) as f32 * (ch + 14.0));
            if self.clicked_rect(x, y, cw, ch) { if cur == i { ok = true; } cur = i; }
            let (name, th, mugen) = if i == 0 { ("Aleatório".to_string(), [[255, 47, 208], [34, 232, 255], [255, 226, 58]], false) } else { let s = &self.reg.stages[i - 1]; (s.name.clone(), s.thumb, s.mugen) };
            self.ui.gradient_v(x, y, cw, ch * 0.5, rgb(th[0]), rgb(th[1]));
            self.ui.gradient_v(x, y + ch * 0.5, cw, ch * 0.5, rgb(th[1]), rgb(th[2]));
            if let Some(image) = i.checked_sub(1).and_then(|index| self.reg.stage_preview(index)) {
                let texture = self.g.texture(&image, true);
                draw_texture_ex(&texture, self.ui.x(x), self.ui.y(y), WHITE, DrawTextureParams { dest_size: Some(vec2(cw * self.ui.s, ch * self.ui.s)), source: Some(Rect::new(0.0, (image.h as f32 - image.w as f32 * ch / cw) * 0.5, image.w as f32, image.w as f32 * ch / cw)), ..Default::default() });
            }
            self.ui.rect(x, y + ch - 30.0, cw, 30.0, Color::new(0.0, 0.0, 0.0, 0.6));
            self.ui.text(&name.chars().take(28).collect::<String>(), x + 10.0, y + ch - 9.0, 16.0, WHITE, Align::Left, true);
            if mugen { self.ui.rect(x + cw - 60.0, y + 6.0, 54.0, 16.0, hex("#22e8ff")); self.ui.text("MUGEN", x + cw - 33.0, y + 19.0, 12.0, BLACK, Align::Center, true); }
            if cur == i { self.ui.rect_lines(x - 3.0, y - 3.0, cw + 6.0, ch + 6.0, 4.0, hex("#ffcc33")); }
        }
        if total_rows > VIEW_ROWS {
            let bh = VIEW_ROWS as f32 * (ch + 14.0) - 14.0;
            self.ui.rect(VW - 22.0, 88.0, 6.0, bh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (bh * VIEW_ROWS as f32 / total_rows as f32).max(18.0);
            self.ui.rect(VW - 22.0, 88.0 + (bh - th) * self.stage_row as f32 / (total_rows - VIEW_ROWS) as f32, 6.0, th, hex("#ff4d6d"));
        }
        if ok {
            self.sel.2 = if cur == 0 { (get_time() * 7.0) as usize % self.reg.stages.len() } else { cur - 1 };
            self.go(Screen::Vs { t: 0.0, step: 0 });
            return;
        }
        if self.c.pressed(nav::BACK) { self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; return; }
        self.screen = Screen::Stages { cur };
    }

    /// Fighters to load for the current bout as (side, slot, registry index): leaders only, or whole teams in parallel mode.
    fn load_plan(&self) -> Vec<(usize, usize, usize)> {
        let mut v = vec![];
        match &self.team {
            Some(t) if t.simul => { for k in 0..t.members[0].len().max(t.members[1].len()) { for side in 0..2 { if let Some(&r) = t.members[side].get(k) { v.push((side, k, r)); } } } }
            _ => { v.push((0, 0, self.sel.0)); v.push((1, 0, self.sel.1)); }
        }
        v
    }
    fn load_total(&self) -> u8 { (self.load_plan().len() + 3) as u8 }
    fn load_label(&self, step: u8) -> String {
        let plan = self.load_plan();
        match plan.get(step as usize) {
            Some((_, _, r)) => format!("Carregando {}…", self.reg.chars[*r].name),
            None => ["Montando o cenário…", "Preparando a luta…", "Sintetizando a música…"][(step as usize - plan.len()).min(2)].to_string(),
        }
    }

    /// One unit of fight preparation (runs behind the VS screen, one per frame, so the bar animates and the UI never freezes).
    fn pal_for(plan: &[(usize, usize, usize)], st: usize) -> i32 {
        let (side, k, r) = plan[st];
        let dup = (side == 1 && plan.iter().any(|p| p.0 == 0 && p.2 == r)) || plan.iter().any(|p| p.0 == side && p.1 < k && p.2 == r);
        if dup { 2 } else { 1 }
    }

    async fn load_step(&mut self, step: u8, wait: bool) -> Result<bool, String> {
        let plan = self.load_plan();
        let s = self.sel.2;
        let n = plan.len();
        let st = step as usize;
        if st < n {
            if st == 0 && self.pending.jobs.is_empty() {
                let keep: Vec<usize> = plan.iter().map(|p| p.2).collect();
                self.reg.trim_cache(&keep);
                self.pending = Pending::default();
                let mut seen: Vec<(usize, i32)> = vec![];
                for k in 0..n {
                    let key = (plan[k].2, Self::pal_for(&plan, k));
                    self.pending.jobs.push(if seen.contains(&key) { None } else { seen.push(key); self.reg.chars[key.0].spawn_load(key.1) });
                }
            }
            if !wait && self.pending.jobs.get(st).is_some_and(|j| j.as_ref().is_some_and(|h| !h.is_finished())) { return Ok(false); }
            let (side, _, r) = plan[st];
            let job = self.pending.jobs.get_mut(st).and_then(|j| j.take());
            let d = self.reg.chars[r].finish_load(Self::pal_for(&plan, st), job)?;
            self.pending.defs[side].push(d);
        } else if st == n {
            self.pending.jobs.clear();
            let job = self.pending.stage_job.get_or_insert_with(|| self.reg.prepare_stage(s));
            match job.as_mut().poll(&mut std::task::Context::from_waker(std::task::Waker::noop())) {
                std::task::Poll::Pending => return Ok(false),
                std::task::Poll::Ready(result) => result?,
            }
            self.pending.stage_job = None;
            self.pending.stage = Some(self.reg.build_stage(s));
        } else if st == n + 1 {
            let stage = self.pending.stage.take().ok_or("stage")?;
            let teams = std::mem::take(&mut self.pending.defs);
            let lvl = match &self.arcade { Some((_, i)) if self.mode == Mode::Arcade => (2 + *i as i32 + self.ai_level / 3).min(8), _ => self.ai_level };
            let leader = |side: usize| -> Pilot { match (self.mode, side) {
                (Mode::Watch, 0) => Pilot::Cpu(Ai::new(lvl, 11)), (Mode::Watch, _) => Pilot::Cpu(Ai::new(lvl, 77)),
                (Mode::Versus, 0) => Pilot::Human(0), (Mode::Versus, _) => Pilot::Human(1),
                (Mode::Training, 0) => Pilot::Human(0), (Mode::Training, _) => Pilot::Dummy,
                (_, 0) => Pilot::Human(0), _ => Pilot::Cpu(Ai::new(lvl, 99)),
            } };
            let mut pilots = vec![];
            let replay = self.replaying.clone();
            for k in 0..teams[0].len().max(teams[1].len()) { for side in 0..2 { if teams[side].get(k).is_some() { pilots.push(if k == 0 { leader(side) } else { Pilot::Cpu(Ai::new((lvl - 1).max(1), 31 + (side * 7 + k) as u32)) }); } } }
            if let Some(r) = &replay { for (slot, p) in pilots.iter_mut().enumerate() { *p = Pilot::Replay(r.rec.iter().map(|b| b.get(slot).copied().unwrap_or(0)).collect(), 0); } }
            let (simul, seq) = (self.team.as_ref().is_some_and(|t| t.simul), self.team.as_ref().is_some_and(|t| !t.simul));
            let life = self.team.as_ref().filter(|t| !t.simul).map(|t| t.life).unwrap_or([1.0, 1.0]);
            let info = self.team.as_ref().map(|t| crate::session::TeamInfo {
                names: [0, 1].map(|sd| t.members[sd].iter().map(|&r| (self.reg.chars[r].name.split_whitespace().next().unwrap_or("").to_uppercase(), self.reg.chars[r].color)).collect()),
                idx: t.idx, ko: t.ko.clone(), simul: t.simul,
            });
            self.c.clear_macros();
            let mut sess = Session::new(teams, stage, pilots, self.mode == Mode::Training, self.mode == Mode::Versus, simul, seq, life);
            if let Some(t) = &info { sess.m.team_lead = [0, 1].map(|sd| t.names[sd].first().map(|n| n.0.clone()).unwrap_or_default()); }
            sess.team = info;
            sess.is_replay = replay.is_some();
            self.fight_rules = replay.as_ref().map(|r| r.rules).unwrap_or_else(|| self.settings.rules());
            sess.apply_rules(self.fight_rules);
            if seq { if let Some(t) = &self.team { for side in 0..2 { if let Some(f) = sess.m.fighters.iter_mut().find(|f| !f.is_helper && f.side == side) { f.power = t.power[side].min(f.power_max); } } } }
            sess.r.debug = self.debug_boxes;
            if seq && self.team.as_ref().is_some_and(|t| t.idx != [0, 0]) { sess.vfx.banner("PRÓXIMO LUTADOR", &format!("{} vs {}", self.reg.chars[self.sel.0].name, self.reg.chars[self.sel.1].name), [0.4, 0.9, 1.0]); }
            self.pending.sess = Some(sess);
        } else {
            let music = self.pending.sess.as_ref().map(|s| s.stage.music).unwrap_or(0);
            if let Some(sess) = &self.pending.sess {
                let mut chars: Vec<crate::audio::CharSounds> = vec![];
                for f in &sess.m.fighters { if !chars.iter().any(|(id, _)| *id == f.def.id) { chars.push((&f.def.id, &f.def.sounds)); } }
                self.audio.preload_match(&chars);
            }
            self.audio.prepare_music(music).await;
        }
        Ok(true)
    }

    fn abort_load(&mut self, e: String) { self.msg = e; self.pending = Pending::default(); self.screen = Screen::Menu { cur: 0 }; }

    async fn vs(&mut self, dt: f32) {
        let Screen::Vs { t, step } = &mut self.screen else { return };
        *t += dt;
        let (t, step_v) = (*t, *step);
        // work for the step labelled on the previous frame (that frame is still on screen while we block)
        let mut step_now = step_v;
        let total = self.load_total();
        if t > dt * 1.5 && step_v < total {
            match self.load_step(step_v, false).await {
                Err(e) => { self.abort_load(e); return; }
                Ok(true) => { step_now = step_v + 1; if let Screen::Vs { step, .. } = &mut self.screen { *step = step_now; } }
                Ok(false) => {}
            }
        }
        let ready = step_now >= total;
        self.draw_versus(t);
        if !ready {
            self.ui.progress_bar(440.0, 84.0, 400.0, step_now as f32 / total as f32, &self.load_label(step_now));
        } else if t > 1.8 || self.c.pressed(nav::OK | nav::START) || is_mouse_button_pressed(MouseButton::Left) {
            self.finish_fight().await;
        }
    }

    /// Synchronous variant (screenshot tour, rematch): runs every loading step back to back.
    pub async fn start_fight(&mut self) {
        for st in 0..self.load_total() { if let Err(e) = self.load_step(st, true).await { self.abort_load(e); return; } }
        self.finish_fight().await;
    }

    async fn finish_fight(&mut self) {
        let Some(sess) = self.pending.sess.take() else { self.screen = Screen::Menu { cur: 0 }; return };
        let music = sess.stage.music;
        let bgm = sess.stage.bgm.clone().or_else(|| self.market.pick_track());
        // DualShock/DualSense lightbars: P1 blue, P2 red
        self.c.pads.set_light(0, [30, 90, 255]);
        self.c.pads.set_light(1, [255, 40, 60]);
        self.g.trim(4000);
        let mut warmed: Vec<&str> = vec![];
        for f in &sess.m.fighters {
            if warmed.contains(&f.def.id.as_str()) { continue; }
            warmed.push(&f.def.id);
            if !self.g.prefetch(&f.def, 0.5) { break; }
        }
        self.session = Some(sess);
        self.screen = Screen::Fight;
        self.audio.reset_bgm();
        if let Some(b) = bgm.as_ref() { self.audio.register_music_file("stage", b.0.clone(), b.1); }
        let custom = match bgm { Some(b) => self.audio.play_music_file(&b.0, b.1).await, None => false };
        if !custom { self.audio.play_music(music).await; }
    }

    async fn fight(&mut self, dt: f32) {
        let Some(s) = self.session.as_mut() else { self.screen = Screen::Menu { cur: 0 }; return };
        if self.c.pressed(nav::AUX) { s.r.debug = !s.r.debug; self.debug_boxes = s.r.debug; }
        let pause_btn = self.c.pressed(nav::START) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P);
        let (paused, accept_back) = pause_step(s.paused, pause_btn);
        if pause_btn { self.pause_cur = 0; self.move_page = 0; self.audio.sfx("select"); }
        s.paused = paused;
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if s.net.is_some() { s.paused = false; }
        // pause button (touch) top-center
        let pb = (VW / 2.0 - 22.0, 92.0, 44.0, 44.0);
        if !s.paused && self.c.touch.enabled && is_mouse_button_pressed(MouseButton::Left) {
            let p = self.ui.to_virtual(mouse_position().into());
            if p.x > pb.0 && p.x < pb.0 + pb.2 && p.y > pb.1 && p.y < pb.1 + pb.3 { s.paused = true; }
        }
        if is_key_pressed(KeyCode::Tab) && !s.paused { s.switch_control(); }
        s.update(dt, &mut self.c, &mut self.audio);
        s.draw(&mut self.g, &self.ui);
        self.c.touch.draw();
        if self.c.touch.enabled { self.ui.rect(pb.0, pb.1, pb.2, pb.3, Color::new(0.0, 0.0, 0.0, 0.35)); self.ui.text("II", VW / 2.0, pb.1 + 30.0, 20.0, WHITE, Align::Center, true); }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(n) = self.session.as_ref().and_then(|s| s.net.as_ref()) { self.net_overlay(n); }
        let s = self.session.as_mut().unwrap();
        if s.paused { self.pause_menu(accept_back).await; return; }
        if let Some(w) = s.over { if s.over_timer > 1.0 { if !self.team_next(w) {
                // keep the fight so it can be replayed (not for replays themselves or sequence team bouts)
                if self.replaying.is_none() && !self.team.as_ref().is_some_and(|t| !t.simul) { if let Some(s) = self.session.as_ref() { self.last_fight = Some(Replay { sel: self.sel, mode: self.mode, team: self.team.clone(), rec: s.rec.clone(), rules: self.fight_rules }); } }
                self.screen = Screen::Results { winner: w, cur: 0 };
            } } }
    }

    /// Sequence mode: after a bout the winner keeps (most of) its life and the loser's next teammate steps in.
    /// Returns true when another bout was queued (false → the team match is over / not a sequence).
    fn team_next(&mut self, w: usize) -> bool {
        let Some(t) = self.team.as_mut() else { return false };
        if t.simul { return false; }
        let Some(s) = self.session.as_ref() else { return false };
        let ratio = s.m.roots().find(|f| f.side == w).map(|f| (f.life / f.life_max).clamp(0.0, 1.0)).unwrap_or(1.0);
        // shared team meter carries over to the next fighter
        for side in 0..2 { if let Some(f) = s.m.roots().find(|f| f.side == side) { t.power[side] = f.power; } }
        let lose = 1 - w;
        if let Some(k) = t.ko[lose].get_mut(t.idx[lose]) { *k = true; }
        t.idx[lose] += 1;
        t.life[lose] = 1.0;
        t.life[w] = (ratio + 0.12).min(1.0);
        if t.idx[lose] >= t.members[lose].len() { return false; }
        self.sel = (t.members[0][t.idx[0]], t.members[1][t.idx[1]], self.sel.2);
        self.audio.stop_music();
        self.session = None;
        self.screen = Screen::Vs { t: 0.0, step: 0 };
        true
    }
    fn reset_team(&mut self) {
        if let Some(t) = self.team.as_mut() { *t = Team::new(t.members[0].clone(), t.members[1].clone(), t.simul); self.sel = (t.members[0][0], t.members[1][0], self.sel.2); }
    }

    async fn pause_menu(&mut self, accept_back: bool) {
        let items = ["Continuar", "Hitboxes", "Reiniciar luta", "Trocar personagens", "Configurações", "Menu principal"];
        if self.c.pressed(nav::DOWN) { self.pause_cur = (self.pause_cur + 1) % items.len(); }
        if self.c.pressed(nav::UP) { self.pause_cur = (self.pause_cur + items.len() - 1) % items.len(); }
        let mut ok = self.c.pressed(nav::OK);
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.02, 0.01, 0.06, 0.72));
        let (x, y) = (120.0, 110.0);
        self.ui.rect(x, y, 1040.0, 520.0, Color::new(0.08, 0.05, 0.16, 0.92));
        self.ui.text("PAUSA", x + 30.0, y + 52.0, 34.0, hex("#ffcc33"), Align::Left, true);
        let s = self.session.as_ref().unwrap();
        for (i, it) in items.iter().enumerate() {
            let (bx, by) = (x + 30.0, y + 80.0 + i as f32 * 56.0);
            if self.clicked_rect(bx, by, 330.0, 48.0) { self.pause_cur = i; ok = true; }
            self.ui.rect(bx, by, 330.0, 48.0, if i == self.pause_cur { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.06) });
            let label = if i == 1 { format!("Hitboxes: {}", if s.r.debug { "ON" } else { "OFF" }) } else { it.to_string() };
            self.ui.text(&label, bx + 16.0, by + 32.0, 22.0, WHITE, Align::Left, false);
        }
        let f = &s.m.fighters[0];
        self.ui.text(&format!("GOLPES — {}", f.def.display_name.to_uppercase()), x + 400.0, y + 52.0, 22.0, hex("#22e8ff"), Align::Left, true);
        const PER_PAGE: usize = 9;
        let total = f.def.movelist.len();
        let pages = total.div_ceil(PER_PAGE).max(1);
        if self.c.pressed(nav::RIGHT) { self.move_page = (self.move_page + 1) % pages; }
        if self.c.pressed(nav::LEFT) { self.move_page = (self.move_page + pages - 1) % pages; }
        self.move_page = self.move_page.min(pages - 1);
        if total == 0 { self.ui.text("Este personagem não define comandos no .cmd", x + 400.0, y + 90.0, 18.0, WHITE, Align::Left, false); }
        for (i, mv) in f.def.movelist.iter().enumerate().skip(self.move_page * PER_PAGE).take(PER_PAGE) {
            let yy = y + 78.0 + (i - self.move_page * PER_PAGE) as f32 * 46.0;
            self.ui.rect(x + 390.0, yy, 630.0, 40.0, Color::new(1.0, 1.0, 1.0, if i % 2 == 0 { 0.06 } else { 0.03 }));
            self.ui.text(&mv.name.chars().take(26).collect::<String>(), x + 402.0, yy + 27.0, 19.0, WHITE, Align::Left, false);
            let c = &self.c;
            self.ui.input_icons(&mv.input, x + 690.0, yy + 34.0, 28.0, |b| c.face(b));
        }
        if pages > 1 {
            self.ui.rect(x + 880.0, y + 20.0, 60.0, 44.0, Color::new(1.0, 1.0, 1.0, 0.12));
            self.ui.rect(x + 950.0, y + 20.0, 60.0, 44.0, Color::new(1.0, 1.0, 1.0, 0.12));
            self.ui.arrow(x + 910.0, y + 42.0, 26.0, (-1.0, 0.0), WHITE);
            self.ui.arrow(x + 980.0, y + 42.0, 26.0, (1.0, 0.0), WHITE);
            if self.clicked_rect(x + 880.0, y + 20.0, 60.0, 44.0) { self.move_page = (self.move_page + pages - 1) % pages; }
            if self.clicked_rect(x + 950.0, y + 20.0, 60.0, 44.0) { self.move_page = (self.move_page + 1) % pages; }
            self.ui.text(&format!("página {}/{}", self.move_page + 1, pages), x + 400.0, y + 505.0, 16.0, hex("#a49cc0"), Align::Left, false);
        }
        if accept_back && self.c.pressed(nav::BACK) { self.session.as_mut().unwrap().paused = false; return; }
        if ok {
            match self.pause_cur {
                0 => self.session.as_mut().unwrap().paused = false,
                1 => { let s = self.session.as_mut().unwrap(); s.r.debug = !s.r.debug; self.debug_boxes = s.r.debug; }
                2 => { self.reset_team(); self.start_fight().await; }
                3 => { self.audio.stop_music(); self.session = None; self.team = None; self.team_pick.clear(); self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; }
                4 => self.screen = Screen::Settings { tab: 0, cur: 0, from_fight: true },
                _ => { self.audio.stop_music(); self.session = None; self.team = None; self.screen = Screen::Menu { cur: 0 }; }
            }
        }
    }

    async fn results(&mut self) {
        let Screen::Results { winner, mut cur } = self.screen else { return };
        if let Some(s) = self.session.as_mut() { s.draw(&mut self.g, &self.ui); }
        // arcade progression
        if self.mode == Mode::Arcade && winner == 0 {
            let boss = self.boss_idx();
            if let Some((ladder, i)) = self.arcade.as_mut() {
                *i += 1;
                if *i < ladder.len() {
                    self.sel.1 = ladder[*i]; self.sel.2 = (*i * 3 + 1) % self.reg.stages.len(); self.audio.stop_music();
                    self.screen = if Some(ladder[*i]) == boss { Screen::Cutscene { kind: 1, page: 0, t: 0.0 } } else { Screen::Vs { t: 0.0, step: 0 } };
                    return;
                }
                if *i == ladder.len() && !self.ending_shown && boss.is_some_and(|b| ladder.last() == Some(&b)) { self.ending_shown = true; self.audio.stop_music(); self.screen = Screen::Cutscene { kind: 2, page: 0, t: 0.0 }; return; }
            }
        }
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.02, 0.01, 0.06, 0.6));
        self.ui.motes(70, 1.6);
        let w = if winner == 0 { self.sel.0 } else { self.sel.1 };
        let e = &self.reg.chars[w];
        self.ui.gradient_v(160.0, 200.0, 960.0, 300.0, alpha(rgb(e.color), 0.9), Color::new(0.08, 0.05, 0.16, 0.95));
        let done = self.mode == Mode::Arcade && self.arcade.as_ref().is_some_and(|(l, i)| *i >= l.len());
        let title = if done { "ARCADE COMPLETO!".to_string() } else if self.team.is_some() { { let lead = self.team.as_ref().and_then(|t| t.members[winner.min(1)].first().copied()).unwrap_or(w); format!("TIME {} VENCEU!", self.reg.chars[lead].name.split_whitespace().next().unwrap_or("").to_uppercase()) } } else { format!("{} VENCE!", e.name.to_uppercase()) };
        let pal = if winner == 1 && self.sel.0 == self.sel.1 { 2 } else { 1 };
        if let Ok(d) = e.load(pal) { let t = self.preview_t; self.draw_char(&d, if d.anims.contains_key(&181) { 181 } else { 0 }, t, 330.0, 480.0, 260.0, false); }
        self.ui.text_bold(&title, 480.0, 300.0, 44.0, hex("#ffcc33"), BLACK, Align::Left);
        let can_replay = self.last_fight.is_some();
        let items: Vec<&str> = if can_replay { vec!["Revanche", "Replay", "Personagens", "Menu"] } else { vec!["Revanche", "Personagens", "Menu"] };
        let nitems = items.len();
        if self.c.pressed(nav::RIGHT) { cur = (cur + 1) % nitems; }
        if self.c.pressed(nav::LEFT) { cur = (cur + nitems - 1) % nitems; }
        let mut ok = self.c.pressed(nav::OK | nav::START);
        for (i, it) in items.iter().enumerate() {
            let (bx, by) = (480.0 + i as f32 * 160.0, 380.0);
            if self.clicked_rect(bx, by, 150.0, 50.0) { cur = i; ok = true; }
            self.ui.rect(bx, by, 150.0, 50.0, if i == cur { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.08) });
            self.ui.text(it, bx + 75.0, by + 33.0, 20.0, WHITE, Align::Center, true);
        }
        self.screen = Screen::Results { winner, cur };
        if ok {
            self.audio.stop_music();
            // with a replay available the menu is Revanche / Replay / Personagens / Menu; without it Revanche / Personagens / Menu
            let choice = if can_replay { cur } else if cur >= 1 { cur + 1 } else { 0 };
            match choice {
                0 => { self.replaying = None; if let Some((_, i)) = self.arcade.as_mut() { *i = (*i).min(5); } self.reset_team(); self.start_fight().await; }
                1 => { self.start_replay().await; }
                2 => { self.replaying = None; self.session = None; self.team = None; self.team_pick.clear(); self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; }
                _ => { self.replaying = None; self.session = None; self.team = None; self.screen = Screen::Menu { cur: 0 }; }
            }
        }
    }

    /// Snapshot the running session as the replayable "last fight" (used by the screenshot tour's determinism check).
    pub fn capture_replay(&mut self) { if let Some(s) = self.session.as_ref() { self.last_fight = Some(Replay { sel: self.sel, mode: self.mode, team: self.team.clone(), rec: s.rec.clone(), rules: self.fight_rules }); } }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub async fn start_netplay(&mut self, a: &crate::netlink::NetArgs) {
        let find_char = |id: &str| self.reg.chars.iter().position(|c| c.id.eq_ignore_ascii_case(id) || c.name.eq_ignore_ascii_case(id));
        let (c1, c2) = match &a.chars { Some((x, y)) => (find_char(x), find_char(y)), None => (Some(0), Some(1.min(self.reg.chars.len() - 1))) };
        let (Some(c1), Some(c2)) = (c1, c2) else { self.msg = format!("Netplay: personagem não encontrado ({:?})", a.chars); return };
        let stage = a.stage.as_ref().map_or(Some(0), |id| self.reg.stages.iter().position(|s| s.id.eq_ignore_ascii_case(id) || s.name.eq_ignore_ascii_case(id)));
        let Some(stage) = stage else { self.msg = format!("Netplay: cenário não encontrado ({:?})", a.stage); return };
        let link = match crate::netlink::NetLink::connect(a) { Ok(l) => l, Err(e) => { self.msg = format!("Netplay: {e}"); return } };
        self.mode = Mode::Versus; self.team = None; self.arcade = None; self.replaying = None;
        self.sel = (c1, c2, stage);
        let saved = self.settings;
        self.settings = Settings { round_time: 0, rounds: 1, life: 2, ..saved };
        self.start_fight().await;
        self.settings = saved;
        if let Some(s) = self.session.as_mut() { s.net = Some(link); }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    fn net_overlay(&self, n: &crate::netlink::NetLink) {
        use crate::netlink::NetStatus;
        let (text, color) = match n.status() {
            NetStatus::Connecting => ("CONECTANDO AO ADVERSÁRIO…".to_string(), hex("#ffcc33")),
            NetStatus::Waiting => ("SINCRONIZANDO…".to_string(), hex("#ffcc33")),
            NetStatus::Lost(e) => (format!("CONEXÃO PERDIDA: {e}"), hex("#ff4d6d")),
            NetStatus::Running => (format!("ONLINE P{} · {} ms", n.local + 1, n.ping_ms().unwrap_or(0)), hex("#a49cc0")),
        };
        self.ui.text(&text, VW / 2.0, VH - 16.0, 18.0, color, Align::Center, true);
        if n.desyncs > 0 { self.ui.text(&format!("DESSINCRONIZADO ({})", n.desyncs), VW / 2.0, VH - 40.0, 18.0, hex("#ff4d6d"), Align::Center, true); }
    }

    /// Re-simulate the last finished fight from its recorded inputs (deterministic engine → identical fight).
    pub async fn start_replay(&mut self) {
        let Some(r) = self.last_fight.clone() else { return };
        self.mode = r.mode; self.team = r.team.clone(); self.sel = r.sel;
        self.replaying = Some(r);
        self.start_fight().await;
    }

    fn controls_screen(&mut self) {
        if self.key_edit.is_some() { self.key_editor(); return; }
        self.ui.background();
        self.ui.text("CONTROLES", 60.0, 70.0, 34.0, hex("#ffcc33"), Align::Left, true);
        use crate::controls::{face, Glyphs};
        use crate::gamepad::PadKind;
        use ume_engine::input::k;
        let kl = |i: usize| crate::keybinds::label(self.c.keys.slots[0][i]);
        let touch = self.c.touch.enabled;
        let mut cols = vec![Glyphs::Keyboard, Glyphs::Pad(PadKind::Xbox), Glyphs::Pad(PadKind::PlayStation)];
        if touch { cols.insert(0, Glyphs::Touch); } else { cols.push(Glyphs::Touch); }
        let head = |g: Glyphs| match g { Glyphs::Keyboard => "Teclado", Glyphs::Touch => "Toque", Glyphs::Pad(p) => p.label() };
        let rows: [(&str, &[u32], &str); 7] = [
            ("Mover", &[], ""), ("Soco", &[k::X, k::Y], " "), ("Chute", &[k::A, k::B], " "), ("Especial", &[k::D2], ""),
            ("Super", &[k::D2, k::W], "+"), ("Despertar", &[k::T], ""), ("Pausa", &[k::S], ""),
        ];
        let move_text = |g: Glyphs| match g { Glyphs::Keyboard => format!("{} {} {} {} / setas", kl(0), kl(1), kl(2), kl(3)), Glyphs::Touch => "Analógico esquerdo".into(), Glyphs::Pad(_) => "Analógico ou direcional".into() };
        let xs = [60.0, 280.0, 520.0, 760.0, 1000.0];
        let (top, rh) = (120.0, 60.0);
        for (j, g) in cols.iter().enumerate() {
            let on = matches!((self.c.glyphs, g), (a, b) if a == *b);
            self.ui.text(head(*g), xs[j + 1], top, 24.0, if on { hex("#ffcc33") } else { hex("#22e8ff") }, Align::Left, true);
        }
        for (i, (act, bits, sep)) in rows.iter().enumerate() {
            let y = top + 18.0 + i as f32 * rh;
            self.ui.rect(50.0, y, 1180.0, rh - 8.0, Color::new(1.0, 1.0, 1.0, if i % 2 == 0 { 0.07 } else { 0.03 }));
            self.ui.text(act, xs[0], y + 36.0, 26.0, hex("#ffcc33"), Align::Left, true);
            for (j, g) in cols.iter().enumerate() {
                if bits.is_empty() { self.ui.text(&move_text(*g), xs[j + 1], y + 34.0, 20.0, WHITE, Align::Left, false); continue; }
                let mut cx = xs[j + 1];
                let bits: &[u32] = if *g == Glyphs::Touch && bits.contains(&k::W) { &[k::W] } else { bits };
                for (n, b) in bits.iter().enumerate() {
                    if n > 0 { cx += if sep.trim().is_empty() { 8.0 } else { self.ui.text(sep, cx + 4.0, y + 36.0, 24.0, hex("#a49cc0"), Align::Left, true) + 10.0 }; }
                    cx += self.ui.face(face(*g, &self.c.keys, *b), cx, y + 44.0, 36.0);
                }
            }
        }
        let pads = self.c.pad_info.clone();
        let status = if pads.is_empty() { "Nenhum controle conectado".to_string() } else { format!("Conectado: {}", pads.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")) };
        self.ui.text(&status, 60.0, 585.0, 20.0, hex("#a49cc0"), Align::Left, false);
        for (i, p) in pads.iter().enumerate() { if p.bits & ume_engine::input::k::A != 0 && self.c.pressed(nav::OK) { self.c.rumble(i, 0.8, 0.8, 300); } }
        self.ui.rect(60.0, 610.0, 200.0, 60.0, hex("#ff4d6d"));
        self.ui.text("Voltar", 160.0, 649.0, 26.0, WHITE, Align::Center, true);
        let back = self.clicked_rect(60.0, 610.0, 200.0, 60.0);
        if !touch {
            self.ui.rect(300.0, 610.0, 360.0, 60.0, Color::new(1.0, 1.0, 1.0, 0.1));
            self.ui.text("Personalizar teclado (Tab)", 480.0, 649.0, 22.0, WHITE, Align::Center, true);
            if is_key_pressed(KeyCode::Tab) || self.clicked_rect(300.0, 610.0, 360.0, 60.0) { self.key_edit = Some(keyedit::KeyEdit { slot: 0, cur: 0, waiting: false, msg: String::new() }); }
        }
        if back || self.c.pressed(nav::BACK) { self.screen = match self.settings_back.take() { Some((tab, from_fight)) => Screen::Settings { tab, cur: 1, from_fight }, None => Screen::Menu { cur: 9 } }; }
    }

    fn mugen_screen(&mut self) {
        self.ui.background();
        self.ui.text("PERSONAGENS E CENÁRIOS M.U.G.E.N", 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let lines = [
            "O motor lê os formatos originais do M.U.G.E.N 1.0/1.1 e do Ikemen GO:",
            "  .def · .sff (v1 PCX e v2 RLE8/RLE5/LZ5/PNG) · .air (Clsn1/Clsn2) · .cmd · .cns · .snd · .act · stages",
            "  Triggers, redirecionamentos, ~90 state controllers, HitDef completo, helpers, projéteis, explods, super pause.",
            "",
            "Como adicionar (desktop): coloque pastas ou arquivos .zip dentro da pasta  mugen/  ao lado do jogo",
            "  ex.: mugen/chars/kfm/kfm.def   ·   mugen/stages/meu_cenario.def   ·   mugen/goku.zip",
            "  (ou defina a variável de ambiente UME_MUGEN_DIR). Eles aparecem automaticamente na seleção.",
            "Versão web: botão \"Importar\" da página (arraste .zip).",
            "",
            "Personagens de anime/jogos (Dragon Ball, Bleach, Naruto…) feitos por fãs pertencem a seus autores",
            "e detentores dos direitos — não são distribuídos com o jogo. Veja docs/ROSTER.md.",
        ];
        for (i, l) in lines.iter().enumerate() { self.ui.text(l, 40.0, 120.0 + i as f32 * 32.0, 20.0, if l.starts_with("  ") { hex("#c9f7ff") } else { WHITE }, Align::Left, false); }
        let found: Vec<String> = self.reg.chars.iter().filter(|c| c.mugen).map(|c| format!("{} ({})", c.name, c.author)).collect();
        self.ui.text(&format!("Encontrados: {} personagens MUGEN, {} cenários MUGEN", found.len(), self.reg.stages.iter().filter(|s| s.mugen).count()), 40.0, 520.0, 22.0, hex("#ffcc33"), Align::Left, true);
        for (i, f) in found.iter().enumerate().take(4) { self.ui.text(f, 60.0, 556.0 + i as f32 * 26.0, 19.0, WHITE, Align::Left, false); }
        if self.c.pressed(nav::BACK | nav::OK) { self.screen = Screen::Menu { cur: 7 }; }
    }

    // used by the screenshot tour
    pub fn draw_session_only(&mut self) { if let Some(s) = self.session.as_mut() { s.draw(&mut self.g, &self.ui); self.c.touch.draw(); } }
    pub fn preview_opts() -> DrawOpts { DrawOpts::default() }
    pub fn renderer_new(st: &crate::stages::Stage) -> MatchRenderer { MatchRenderer::new(st) }
}

/// Pause state for this frame and whether the pause menu may act on `nav::BACK`.
///
/// Esc is both the pause key and `nav::BACK`. Without this guard the same key press
/// opens the menu in `fight()` and closes it again in `pause_menu()` on the same frame,
/// so the pause menu never shows. The opening frame therefore consumes that edge.
fn pause_step(was_paused: bool, toggle: bool) -> (bool, bool) {
    (was_paused ^ toggle, was_paused)
}

/// Numpad notation (2 = down, 6 = forward, 236 = quarter-circle forward).

#[cfg(test)]
mod tests {
    use super::pause_step;

    #[test]
    fn esc_opening_the_pause_menu_does_not_close_it_on_the_same_frame() {
        // Esc is both the pause key and nav::BACK, so the opening frame consumes the edge.
        assert_eq!(pause_step(false, true), (true, false));
    }

    #[test]
    fn pause_menu_is_navigable_once_open() {
        assert_eq!(pause_step(true, false), (true, true)); // paused, no toggle: BACK accepted
        assert_eq!(pause_step(true, true), (false, true)); // Esc while paused closes the menu
        assert_eq!(pause_step(false, false), (false, false)); // idle
    }
}
