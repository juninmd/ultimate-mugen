//! Screens and game flow.
use crate::audio::Audio;
use crate::controls::{nav, Controls};
use crate::registry::Registry;
use crate::render::{Gfx, MatchRenderer, DrawOpts};
use crate::session::{Pilot, Session};
use crate::settings::{Settings, FX_LEVELS, IMPACT, QUALITY, RESOLUTIONS};
use crate::stages::Stage;
use crate::ui::*;
use macroquad::prelude::*;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;

mod cutscene;

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
    Video { cur: usize },
    /// story scene: 0 arcade opening · 1 before the boss · 2 ending
    Cutscene { kind: u8, page: usize, t: f32 },
}

/// Fight assets prepared step by step behind the VS screen.
#[derive(Default)]
pub struct Pending { defs: [Vec<Arc<CharDef>>; 2], stage: Option<Stage>, sess: Option<Session> }

/// Everything needed to re-simulate a finished fight: same fighters, stage and mode, plus the recorded inputs per slot.
#[derive(Clone)]
pub struct Replay { sel: (usize, usize, usize), mode: Mode, team: Option<Team>, rec: Vec<Vec<u32>> }

/// Team battle state: sequence (KOF-style turns, winner keeps its remaining life) or parallel (2v2 simultaneous).
#[derive(Clone)]
pub struct Team { pub members: [Vec<usize>; 2], pub idx: [usize; 2], pub life: [f32; 2], /** KOF XV: the team shares one super meter */ pub power: [f32; 2], pub ko: [Vec<bool>; 2], pub simul: bool }
impl Team {
    pub fn new(a: Vec<usize>, b: Vec<usize>, simul: bool) -> Self { let (la, lb) = (a.len(), b.len()); Team { members: [a, b], idx: [0, 0], life: [1.0, 1.0], power: [0.0, 0.0], ko: [vec![false; la], vec![false; lb]], simul } }
}
const VIEW_ROWS: usize = 4;

pub struct App {
    pub reg: Registry,
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
    stage_row: usize,
    quality_check: f32,
    pub trans: f32,
    screen_t: f32,
    last_disc: Option<std::mem::Discriminant<Screen>>,
}

const MENU: [(&str, &str); 11] = [
    ("Arcade", "Enfrente uma sequência de oponentes"),
    ("Versus CPU", "Você contra o computador"),
    ("2 Jogadores", "Teclado compartilhado ou dois controles"),
    ("Treino", "Boneco de treino, barra infinita, hitboxes"),
    ("CPU vs CPU", "Assista a IA lutar"),
    ("Equipe · Sequência", "Até 5 contra 5, um de cada vez; o vencedor mantém a vida"),
    ("Equipe · Paralelo", "5 contra 5 ao mesmo tempo (2 a 5) · TAB troca de lutador"),
    ("Personagens MUGEN", "Como adicionar personagens e cenários"),
    ("Controles", "Teclado, DualShock/DualSense, toque e vibração"),
    ("Vídeo", "Tela cheia, resolução até 4K e qualidade da arte"),
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
        Self::boot(&mut ui, 0.55, "Pacote da comunidade…").await;
        // Community MUGEN pack (original, freely redistributable characters/stages) loaded at runtime (file or HTTP)
        for path in ["assets/mugen/community.zip", "community.zip"] {
            if let Ok(bytes) = macroquad::file::load_file(path).await {
                let mut vfs = ume_formats::vfs::Vfs::new();
                if ume_formats::vfs::add_zip(&mut vfs, &bytes, "").is_ok() { reg.add_vfs_lazy(vfs, "community"); break; }
            }
        }
        Self::boot(&mut ui, 0.75, "Procurando pasta mugen/…").await;
        #[cfg(not(target_arch = "wasm32"))]
        reg.scan_folders();
        reg.sort_mugen();
        Self::boot(&mut ui, 0.9, "Áudio…").await;
        let audio = Audio::new(audio_on).await;
        Self::boot(&mut ui, 1.0, "Pronto!").await;
        App {
            reg, ui, g: Gfx::new(), c: Controls::new(touch), audio, screen: Screen::Title, mode: Mode::Cpu,
            ai_level: 4, sel: (0, 1, 0), session: None, arcade: None, pause_cur: 0, debug_boxes: false, quit: false, msg: String::new(), preview_t: 0.0,
            settings, pending: Pending::default(), team: None, last_fight: None, replaying: None, team_n: 3, team_pick: vec![], sel_filter: 0, ending_shown: false, char_tint: WHITE, sel_row: 0, sel_last: [usize::MAX; 2], sel_dwell: 0.0, stage_row: 0, quality_check: 0.0, trans: 0.0, screen_t: 0.0, last_disc: None,
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
            Screen::Video { .. } => self.video_screen(),
            Screen::Fight => self.fight(dt).await,
            Screen::Results { .. } => self.results().await,
            Screen::Controls => self.controls_screen(),
            Screen::Mugen => self.mugen_screen(),
            Screen::Cutscene { .. } => self.cutscene(),
        }
        if is_key_pressed(KeyCode::F11) { self.toggle_fullscreen(); }
        if is_key_pressed(KeyCode::F3) { self.settings.fps = !self.settings.fps; self.settings.save(); }
        // auto art quality follows the display (checked outside fights so live sprites are never swapped)
        self.quality_check += dt;
        if self.quality_check > 0.7 && matches!(self.screen, Screen::Menu { .. } | Screen::Title | Screen::Video { .. } | Screen::Select { .. } | Screen::Stages { .. }) { self.quality_check = 0.0; self.apply_quality(); }
        if self.settings.fps { self.ui.text(&format!("{} FPS · {}×{} · arte {:.1}×", get_fps(), (screen_width() * macroquad::miniquad::window::dpi_scale()) as i32, Settings::physical_h() as i32, ume_engine::native::rig::art_res()), 10.0, 20.0, 16.0, hex("#6dff9e"), Align::Left, false); }
        // screen transition: diagonal wipe whenever the screen changes (not into/out of the fight itself)
        let disc = std::mem::discriminant(&self.screen);
        if self.last_disc != Some(disc) {
            if self.last_disc.is_some() && !matches!(self.screen, Screen::Fight) { self.trans = 1.0; }
            self.last_disc = Some(disc);
            self.screen_t = 0.0;
        }
        self.screen_t += dt;
        if self.trans > 0.0 { self.ui.wipe(self.trans); self.trans = (self.trans - dt * 2.6).max(0.0); }
        self.audio.process().await;
        crate::set_import_button(!matches!(self.screen, Screen::Fight | Screen::Vs { .. } | Screen::Results { .. } | Screen::Cutscene { .. }));
        for (i, zip) in crate::take_imports().into_iter().enumerate() {
            let mut vfs = ume_formats::vfs::Vfs::new();
            match ume_formats::vfs::add_zip(&mut vfs, &zip, "") {
                Ok(_) => { let (c, s, e) = self.reg.add_vfs_lazy(vfs, &format!("web{}-{i}", get_time() as u64)); self.reg.sort_mugen(); self.sel = (self.sel.0.min(self.reg.chars.len() - 1), self.sel.1.min(self.reg.chars.len() - 1), self.sel.2.min(self.reg.stages.len() - 1)); self.msg = format!("Importado: {} personagem(ns), {} cenário(s){}", c.len(), s.len(), if e.is_empty() { String::new() } else { format!(" · {} erro(s)", e.len()) }); }
                Err(e) => self.msg = format!("Zip inválido: {e}"),
            }
        }
        if !self.msg.is_empty() {
            self.ui.rect(0.0, VH - 36.0, VW, 36.0, Color::new(0.0, 0.0, 0.0, 0.7));
            self.ui.text(&self.msg, VW / 2.0, VH - 12.0, 18.0, hex("#6dff9e"), Align::Center, false);
            if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Left) { self.msg.clear(); }
        }
    }

    fn go(&mut self, s: Screen) { self.audio.sfx("confirm"); self.screen = s; }

    // ------------------------------------------------------------ title / menu
    fn title(&mut self) {
        self.ui.background();
        self.ui.logo(VW / 2.0, 330.0, 200.0);
        self.ui.text("ENGINE DE LUTA · RUST · DESKTOP · WEB · MOBILE", VW / 2.0, 420.0, 20.0, hex("#a49cc0"), Align::Center, true);
        if (self.ui.t * 2.0) as i32 % 2 == 0 { self.ui.text_bold("PRESSIONE START / ENTER", VW / 2.0, 540.0, 30.0, WHITE, hex("#12051f"), Align::Center); }
        self.ui.text("Compatível com personagens M.U.G.E.N  ·  DualShock 4 / DualSense com vibração", VW / 2.0, 690.0, 18.0, hex("#a49cc0"), Align::Center, false);
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
        self.ui.background();
        self.ui.logo(250.0, 330.0, 120.0);
        for (i, (t, d)) in MENU.iter().enumerate() {
            let k_in = ((self.screen_t - 0.12 - i as f32 * 0.05) / 0.3).clamp(0.0, 1.0);
            let k_in = 1.0 - (1.0 - k_in).powi(3);
            let step_y = 590.0 / MENU.len() as f32;
            let (x, y, w, h) = (460.0 - (1.0 - k_in) * 90.0, 104.0 + i as f32 * step_y, 440.0, step_y - 6.0);
            if self.hover_rect(x, y, w, h) && (mouse_delta_position().length() > 0.0) { cur_v = i; }
            if self.clicked_rect(x, y, w, h) { chosen = Some(i); }
            let f = i == cur_v;
            let off = if f { 14.0 } else { 0.0 };
            let pulse = if f { 0.85 + 0.15 * (self.ui.t * 6.0).sin() } else { 1.0 };
            let mut col = if f { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.06) };
            col.r *= pulse; col.g *= pulse; col.b *= pulse; col.a *= k_in;
            if f { self.ui.quad([vec2(x + off + 8.0, y - 3.0), vec2(x + off + w + 6.0, y - 3.0), vec2(x + off + w - 8.0, y + h + 3.0), vec2(x + off - 6.0, y + h + 3.0)], Color::new(1.0, 0.3, 0.43, 0.22 * k_in)); }
            self.ui.quad([vec2(x + off + 14.0, y), vec2(x + off + w, y), vec2(x + off + w - 14.0, y + h), vec2(x + off, y + h)], col);
            if f { self.ui.quad([vec2(x + off + w - 60.0, y), vec2(x + off + w, y), vec2(x + off + w - 14.0, y + h), vec2(x + off + w - 74.0, y + h)], hex("#8a2be2")); }
            self.ui.text(t, x + off + 26.0, y + h * 0.54, 22.0, WHITE, Align::Left, true);
            self.ui.text(d, x + off + 26.0, y + h * 0.88, 14.0, if f { WHITE } else { hex("#a49cc0") }, Align::Left, false);
        }
        let (nm, nn) = (self.reg.chars.iter().filter(|c| c.mugen).count(), self.playable().len());
        let sx = 960.0;
        self.ui.text(&format!("{nn}"), sx, 200.0, 40.0, hex("#ffcc33"), Align::Left, true);
        self.ui.text("lutadores", sx + 70.0, 196.0, 22.0, hex("#a49cc0"), Align::Left, false);
        self.ui.text(&format!("{}", self.reg.stages.len()), sx, 250.0, 40.0, hex("#ffcc33"), Align::Left, true);
        self.ui.text("cenários", sx + 70.0, 246.0, 22.0, hex("#a49cc0"), Align::Left, false);
        self.ui.text(&format!("{nm}"), sx, 300.0, 40.0, hex("#ffcc33"), Align::Left, true);
        self.ui.text("MUGEN importados", sx + 70.0, 296.0, 22.0, hex("#a49cc0"), Align::Left, false);
        self.ui.text("Dificuldade CPU (esq./dir.)", sx, 370.0, 20.0, hex("#a49cc0"), Align::Left, false);
        for i in 0..8 { self.ui.rect(sx + i as f32 * 26.0, 384.0, 20.0, 14.0, if i < self.ai_level { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.12) }); }
        let pads = &self.c.pad_info;
        self.ui.text("Controles conectados", sx, 450.0, 20.0, hex("#a49cc0"), Align::Left, false);
        if pads.is_empty() { self.ui.text("nenhum (teclado)", sx, 476.0, 18.0, WHITE, Align::Left, false); }
        for (i, p) in pads.iter().enumerate().take(4) { self.ui.text(&format!("P{} · {}{}", i + 1, p.name.chars().take(26).collect::<String>(), if p.sony { " (Sony)" } else { "" }), sx, 476.0 + i as f32 * 24.0, 18.0, WHITE, Align::Left, false); }
        self.screen = Screen::Menu { cur: cur_v };
        if self.c.pressed(nav::BACK) { self.screen = Screen::Title; }
        if let Some(i) = chosen {
            let mode = [Mode::Arcade, Mode::Cpu, Mode::Versus, Mode::Training, Mode::Watch, Mode::TeamSeq, Mode::TeamSimul];
            match i {
                0..=6 => { self.mode = mode[i]; self.team = None; self.team_pick.clear(); if self.mode == Mode::TeamSimul { self.team_n = 5; } else if self.mode == Mode::TeamSeq { self.team_n = 3; } self.go(Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }); }
                7 => self.go(Screen::Mugen),
                8 => self.go(Screen::Controls),
                9 => self.go(Screen::Video { cur: 0 }),
                _ => self.quit = true,
            }
        }
    }

    // ------------------------------------------------------------ select
    pub fn draw_char(&mut self, d: &Arc<CharDef>, anim: i32, t: f32, x: f32, y: f32, height_px: f32, mirror: bool) {
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
        let base_h = (d.c("size.height", 60.0) as f32 * 1.7).max(90.0);
        let k = height_px / base_h * self.ui.s;
        let ks = k * spr.scale * l;
        let flip = mirror ^ fr.flip_h;
        let w = spr.width * ks; let h = spr.height * ks;
        let px = self.ui.x(x) + fr.x * l * k * if mirror { -1.0 } else { 1.0 };
        let py = self.ui.y(y) + fr.y * l * k;
        let left = if flip { px - (spr.width - spr.axis_x) * ks } else { px - spr.axis_x * ks };
        draw_texture_ex(&tex, left, py - spr.axis_y * ks, self.char_tint, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_x: flip, ..Default::default() });
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
        let title = if self.mode.is_team() { format!("{} {}v{}", self.mode.label(), self.team_n, self.team_n) } else { self.mode.label().to_string() };
        self.ui.text(&title, 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let hint = if self.mode.is_team() { "Monte sua equipe (Q/E: tamanho) — o adversário é sorteado" } else if picking == 0 { "JOGADOR 1: escolha seu lutador" } else if two { "JOGADOR 2: escolha" } else { "Escolha o oponente" };
        self.ui.text(hint, VW - 40.0, 56.0, 22.0, hex("#a49cc0"), Align::Right, false);
        let flabel = ["TODOS", "ORIGINAIS", "M.U.G.E.N"][self.sel_filter as usize % 3];
        self.ui.text(&format!("[Tab] {flabel} · {}/{m} · [R] aleatório · PgUp/PgDn", pos + 1), 640.0, 92.0, 15.0, hex("#a49cc0"), Align::Center, false);

        if self.mode.is_team() {
            let n = self.team_n;
            self.ui.text("SUA EQUIPE", 40.0, 90.0, 16.0, hex("#a49cc0"), Align::Left, true);
            for k in 0..n {
                let cw = (470.0 / n as f32).min(100.0) - 6.0;
                let x = 150.0 + k as f32 * (cw + 6.0);
                let filled = self.team_pick.get(k).copied();
                self.ui.rect(x, 74.0, cw, 24.0, if filled.is_some() { alpha(rgb(self.reg.chars[filled.unwrap()].color), 0.9) } else if k == self.team_pick.len() { Color::new(1.0, 0.8, 0.2, 0.25 + 0.15 * (self.ui.t * 6.0).sin()) } else { Color::new(1.0, 1.0, 1.0, 0.08) });
                let label = filled.map(|i| self.reg.chars[i].name.split_whitespace().next().unwrap_or("").to_uppercase()).unwrap_or_else(|| format!("{}º", k + 1));
                self.ui.text(&label, x + cw / 2.0, 92.0, if n > 4 { 12.0 } else { 14.0 }, WHITE, Align::Center, true);
            }
        }
        // cursor dwell: heavy loads (MUGEN) wait until the cursor rests; light previews (UMF) load almost immediately
        let focus = cur[picking];
        if self.sel_last[picking] != focus { self.sel_last[picking] = focus; self.sel_dwell = 0.0; } else { self.sel_dwell += dt; }
        let pal_of = |side: usize, i: usize, picked: [Option<usize>; 2]| if side == 1 && picked[0] == Some(i) { 2 } else { 1 };
        let mut want: Vec<(usize, i32)> = vec![(focus, pal_of(picking, focus, picked))];
        for side in 0..2 { if let Some(i) = picked[side] { want.push((i, pal_of(side, i, picked))); } }
        let mut loaded_one = false;
        for (i, pal) in want {
            let e = &self.reg.chars[i];
            if e.peek(pal).is_some() || e.broken.get() { continue; }
            if self.sel_dwell < if e.mugen { 0.18 } else { 0.03 } { continue; }
            if e.load_preview(pal).is_err() { self.msg = format!("Falha ao carregar {}", e.name); }
            loaded_one = true;
        }

        // grid: only the visible rows are drawn and only they stream their portraits (one UMF preview per frame)
        let gap = 6.0;
        let cw = ((640.0 - (cols as f32 - 1.0) * gap) / cols as f32).min(96.0);
        let ch = cw * 1.17;
        let gx = (VW - (cols as f32 * cw + (cols as f32 - 1.0) * gap)) / 2.0;
        let gy = 110.0;
        let mut click = None;
        let first = self.sel_row * cols;
        let last = ((self.sel_row + VIEW_ROWS) * cols).min(m);
        for k in first..last {
            let i = view[k];
            let (cx, cy) = (gx + (k % cols) as f32 * (cw + gap), gy + ((k / cols) - self.sel_row) as f32 * (ch + gap));
            if self.clicked_rect(cx, cy, cw, ch) { click = Some(i); }
            let (ecol, emugen, ename, broken) = { let e = &self.reg.chars[i]; (e.color, e.mugen, e.name.clone(), e.broken.get()) };
            let col = rgb(ecol);
            self.ui.gradient_v(cx, cy, cw, ch, alpha(col, 0.95), hex("#120a22"));
            let mut got = self.reg.chars[i].peek(1);
            if got.is_none() && !emugen && !loaded_one && !broken { got = self.reg.chars[i].load_preview(1).ok(); loaded_one = got.is_some(); }
            if let Some(d) = got {
                let por = d.sprites.get(&(9000, 1)).or(d.sprites.get(&(9000, 0))).filter(|_| emugen).and_then(|p| p.image.clone().map(|im| (im, p.width, p.height)));
                if let Some((im, pw, ph)) = por {
                    let tex = self.g.texture(&im, true);
                    let kk = ((cw - 8.0) / pw).min((ch - 26.0) / ph);
                    draw_texture_ex(&tex, self.ui.x(cx + (cw - pw * kk) / 2.0), self.ui.y(cy + 4.0), WHITE, DrawTextureParams { dest_size: Some(vec2(pw * kk * self.ui.s, ph * kk * self.ui.s)), ..Default::default() });
                } else {
                    self.ui_clip(cx, cy, cw, ch - 18.0, |app| app.draw_char(&d, 0, 0.0, cx + cw / 2.0, cy + ch * 1.55, 260.0, false));
                }
            } else {
                let init: String = ename.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
                self.ui.text(if broken { "✕" } else { &init }, cx + cw / 2.0, cy + ch * 0.58, cw * 0.55, Color::new(1.0, 1.0, 1.0, if broken { 0.5 } else { 0.28 }), Align::Center, true);
            }
            self.ui.rect(cx, cy + ch - 18.0, cw, 18.0, Color::new(0.0, 0.0, 0.0, 0.65));
            let nm: String = if emugen { ename.chars().take(13).collect() } else { ename.split_whitespace().next().unwrap_or("").chars().take(12).collect() };
            self.ui.text(&nm.to_uppercase(), cx + cw / 2.0, cy + ch - 4.0, (cw * 0.125).min(12.0), WHITE, Align::Center, true);
            if emugen { self.ui.rect(cx + cw - 44.0, cy + 3.0, 41.0, 13.0, hex("#22e8ff")); self.ui.text("MUGEN", cx + cw - 23.5, cy + 14.0, 10.0, BLACK, Align::Center, true); }
            let pulse = 4.0 + 1.5 * (self.ui.t * 8.0).sin();
            let c1 = cur[0] == i && (picking == 0 || picked[0].is_some());
            let c2 = cur[1] == i && picking == 1;
            if c1 { self.ui.rect_lines(cx - 2.0, cy - 2.0, cw + 4.0, ch + 4.0, pulse, hex("#3fa9ff")); }
            if c2 { self.ui.rect_lines(cx - 6.0, cy - 6.0, cw + 12.0, ch + 12.0, pulse, hex("#ff4d6d")); }
        }
        // scrollbar
        if total_rows > VIEW_ROWS {
            let (bx, bh) = (gx + cols as f32 * (cw + gap) + 2.0, VIEW_ROWS as f32 * (ch + gap) - gap);
            self.ui.rect(bx, gy, 5.0, bh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (bh * VIEW_ROWS as f32 / total_rows as f32).max(18.0);
            self.ui.rect(bx, gy + (bh - th) * self.sel_row as f32 / (total_rows - VIEW_ROWS) as f32, 5.0, th, hex("#ff4d6d"));
        }
        if let Some(i) = click { if cur[picking] == i { confirm = true; } else { cur[picking] = i; self.audio.sfx("select"); } }
        // previews
        for side in 0..2 {
            let idx = picked[side].or(if side == picking { Some(cur[side]) } else { None });
            let x = if side == 0 { 170.0 } else { VW - 170.0 };
            let col = if side == 0 { Color::new(0.25, 0.66, 1.0, 0.18) } else { Color::new(1.0, 0.3, 0.43, 0.18) };
            glow(self.ui.x(x), self.ui.y(540.0), 200.0 * self.ui.s, Color::new(col.r, col.g, col.b, 0.9));
            draw_ellipse(self.ui.x(x), self.ui.y(600.0), 80.0 * self.ui.s, 12.0 * self.ui.s, 0.0, Color::new(0.0, 0.0, 0.0, 0.4));
            if let Some(i) = idx {
                if let Some(d) = self.reg.chars[i].peek(pal_of(side, i, picked)) {
                    let anim = if picked[side].is_some() && d.anims.contains_key(&181) { 181 } else { 0 };
                    let t = self.preview_t;
                    self.draw_char(&d, anim, t, x, 600.0, 330.0, side == 1);
                } else if !self.reg.chars[i].broken.get() {
                    self.ui.progress_bar(x - 90.0, 520.0, 180.0, (self.sel_dwell / 0.4).min(0.95), "Carregando…");
                }
                let e = &self.reg.chars[i];
                self.ui.text_bold(&e.name.to_uppercase(), x, 650.0, if e.name.len() > 16 { 20.0 } else { 26.0 }, if picked[side].is_some() { hex("#ffcc33") } else { WHITE }, BLACK, Align::Center);
                self.ui.text(&e.series, x, 676.0, 15.0, hex("#a49cc0"), Align::Center, false);
            } else { self.ui.text_bold("???", x, 650.0, 30.0, hex("#a49cc0"), BLACK, Align::Center); }
        }
        let desc = &self.reg.chars[cur[picking]].description;
        self.ui.text(desc, VW / 2.0, 700.0, 18.0, hex("#d8d0f0"), Align::Center, false);
        // bound memory with hundreds of fighters
        let mut keep: Vec<usize> = cur.to_vec(); keep.extend(picked.iter().flatten());
        self.reg.trim_cache(&keep);
        if confirm {
            if self.reg.chars[cur[picking]].broken.get() { self.msg = "Este personagem não pôde ser carregado.".into(); self.screen = Screen::Select { cur, picking, picked }; return; }
            self.audio.sfx("confirm");
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

    fn ui_clip(&mut self, x: f32, y: f32, w: f32, h: f32, f: impl FnOnce(&mut App)) {
        let (sx, sy, sw, sh) = (self.ui.x(x), self.ui.y(y), w * self.ui.s, h * self.ui.s);
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        gl.quad_gl.scissor(Some((sx as i32, sy as i32, sw as i32, sh as i32)));
        f(self);
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        gl.quad_gl.scissor(None);
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
    async fn load_step(&mut self, step: u8) -> Result<(), String> {
        let plan = self.load_plan();
        let s = self.sel.2;
        let n = plan.len();
        let st = step as usize;
        if st < n {
            if st == 0 { let keep: Vec<usize> = plan.iter().map(|p| p.2).collect(); self.reg.trim_cache(&keep); self.pending = Pending::default(); }
            let (side, k, r) = plan[st];
            // same fighter on both sides / twice in a team → alternate palette
            let dup = (side == 1 && plan.iter().any(|p| p.0 == 0 && p.2 == r)) || plan.iter().any(|p| p.0 == side && p.1 < k && p.2 == r);
            let d = self.reg.chars[r].load(if dup { 2 } else { 1 })?;
            self.pending.defs[side].push(d);
        } else if st == n {
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
            let mut sess = Session::new(teams, stage, pilots, self.mode == Mode::Training, self.mode == Mode::Versus, simul, seq, life);
            sess.team = info;
            sess.is_replay = replay.is_some();
            if seq { if let Some(t) = &self.team { for side in 0..2 { if let Some(f) = sess.m.fighters.iter_mut().find(|f| !f.is_helper && f.side == side) { f.power = t.power[side].min(f.power_max); } } } }
            sess.r.debug = self.debug_boxes;
            if seq && self.team.as_ref().is_some_and(|t| t.idx != [0, 0]) { sess.vfx.banner("PRÓXIMO LUTADOR", &format!("{} vs {}", self.reg.chars[self.sel.0].name, self.reg.chars[self.sel.1].name), [0.4, 0.9, 1.0]); }
            self.pending.sess = Some(sess);
        } else {
            let music = self.pending.sess.as_ref().map(|s| s.stage.music).unwrap_or(0);
            self.audio.prepare_music(music).await;
        }
        Ok(())
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
            if let Err(e) = self.load_step(step_v).await { self.abort_load(e); return; }
            step_now = step_v + 1;
            if let Screen::Vs { step, .. } = &mut self.screen { *step = step_now; }
        }
        let ready = step_now >= total;
        let (a, b) = (self.sel.0, self.sel.1);
        let k = (t / 0.4).min(1.0);
        clear_background(BLACK);
        let ca = rgb(self.reg.chars[a].color); let cb = rgb(self.reg.chars[b].color);
        self.ui.gradient_v(-(1.0 - k) * 640.0, 0.0, 640.0, VH, ca, BLACK);
        self.ui.gradient_v(640.0 + (1.0 - k) * 640.0, 0.0, 640.0, VH, cb, BLACK);
        crate::vfx::speed_lines((screen_width() / 2.0, screen_height() * 0.46), 0.10 + 0.08 * (self.ui.t * 3.0).sin().abs(), [1.0, 0.8, 0.5], (self.ui.t * 12.0) as u32, 40);
        let pt = self.preview_t;
        // previews: whatever is already loaded (full or idle-only), so this screen itself never stalls
        if let Some(d) = self.reg.chars[a].peek(1) { self.draw_char(&d, 0, pt, 320.0 - (1.0 - k) * 640.0, 560.0, 420.0, false); }
        if let Some(d) = self.reg.chars[b].peek(if a == b { 2 } else { 1 }).or_else(|| self.reg.chars[b].load_preview(if a == b { 2 } else { 1 }).ok().filter(|_| !self.reg.chars[b].mugen)) { self.draw_char(&d, 0, pt, 960.0 + (1.0 - k) * 640.0, 560.0, 420.0, true); }
        self.ui.text_bold(&self.reg.chars[a].name.to_uppercase(), 320.0, 630.0, 40.0, WHITE, BLACK, Align::Center);
        self.ui.text_bold(&self.reg.chars[b].name.to_uppercase(), 960.0, 630.0, 40.0, WHITE, BLACK, Align::Center);
        self.ui.text(&self.reg.chars[a].series, 320.0, 662.0, 20.0, hex("#dddddd"), Align::Center, false);
        self.ui.text(&self.reg.chars[b].series, 960.0, 662.0, 20.0, hex("#dddddd"), Align::Center, false);
        let vs_s = 170.0 * (if t < 0.6 { 3.0 - 2.0 * (t / 0.6) } else { 1.0 });
        glow(self.ui.x(640.0), self.ui.y(330.0), 220.0 * self.ui.s, Color::new(1.0, 0.5, 0.1, 0.7));
        self.ui.text_bold("VS", 640.0, 330.0 + vs_s * 0.35, vs_s, hex("#ffcc33"), hex("#5a0a10"), Align::Center);
        let sub = if let Some(t) = &self.team { let (ia, ib) = (t.idx[0] + 1, t.idx[1] + 1); if t.simul { format!("EQUIPE PARALELO · {} + {} vs {} + {}", self.reg.chars[t.members[0][0]].name, self.reg.chars[t.members[0][1.min(t.members[0].len() - 1)]].name, self.reg.chars[t.members[1][0]].name, self.reg.chars[t.members[1][1.min(t.members[1].len() - 1)]].name) } else { format!("EQUIPE SEQUÊNCIA · {}º vs {}º lutador", ia, ib) } } else if let Some((l, i)) = &self.arcade { format!("ARCADE · LUTA {}/{}", i + 1, l.len()) } else { self.reg.stages[self.sel.2].name.clone() };
        self.ui.text(&sub, 640.0, 700.0, 20.0, hex("#a49cc0"), Align::Center, true);
        if !ready {
            self.ui.progress_bar(440.0, 28.0, 400.0, step_now as f32 / total as f32, &self.load_label(step_now));
        } else if t > 1.8 || self.c.pressed(nav::OK | nav::START) || is_mouse_button_pressed(MouseButton::Left) {
            self.finish_fight().await;
        }
    }

    /// Synchronous variant (screenshot tour, rematch): runs every loading step back to back.
    pub async fn start_fight(&mut self) {
        for st in 0..self.load_total() { if let Err(e) = self.load_step(st).await { self.abort_load(e); return; } }
        self.finish_fight().await;
    }

    async fn finish_fight(&mut self) {
        let Some(sess) = self.pending.sess.take() else { self.screen = Screen::Menu { cur: 0 }; return };
        let music = sess.stage.music;
        // DualShock/DualSense lightbars: P1 blue, P2 red
        self.c.pads.set_light(0, [30, 90, 255]);
        self.c.pads.set_light(1, [255, 40, 60]);
        self.session = Some(sess);
        self.g.trim(4000);
        self.screen = Screen::Fight;
        self.audio.play_music(music).await;
    }

    fn video_screen(&mut self) {
        let Screen::Video { mut cur } = self.screen else { return };
        let web = cfg!(target_arch = "wasm32");
        let rows: Vec<usize> = if web { vec![0, 2, 3, 4, 5, 6] } else { vec![0, 1, 2, 3, 4, 5, 6] };
        let mut pos = rows.iter().position(|&r| r == cur).unwrap_or(0);
        if self.c.pressed(nav::DOWN) { pos = (pos + 1) % rows.len(); self.audio.sfx("select"); }
        if self.c.pressed(nav::UP) { pos = (pos + rows.len() - 1) % rows.len(); self.audio.sfx("select"); }
        cur = rows[pos];
        let dir = (self.c.pressed(nav::RIGHT) as i32) - (self.c.pressed(nav::LEFT) as i32);
        let ok = self.c.pressed(nav::OK | nav::START);
        let mut changed = false;
        match cur {
            0 if ok || dir != 0 => { self.settings.fullscreen = !self.settings.fullscreen; changed = true; }
            1 if ok || dir != 0 => { let d = if dir == 0 { 1 } else { dir }; self.settings.res = ((self.settings.res as i32 + d).rem_euclid(RESOLUTIONS.len() as i32)) as usize; self.settings.fullscreen = false; changed = true; }
            2 if ok || dir != 0 => { let d = if dir == 0 { 1 } else { dir }; self.settings.quality = ((self.settings.quality as i32 + d).rem_euclid(QUALITY.len() as i32)) as usize; changed = true; }
            3 if ok || dir != 0 => { let d = if dir == 0 { 1 } else { dir }; self.settings.fx = ((self.settings.fx as i32 + d).rem_euclid(FX_LEVELS.len() as i32)) as usize; changed = true; }
            4 if ok || dir != 0 => { let d = if dir == 0 { 1 } else { dir }; self.settings.impact = ((self.settings.impact as i32 + d).rem_euclid(IMPACT.len() as i32)) as usize; changed = true; }
            5 if ok || dir != 0 => { self.settings.fps = !self.settings.fps; changed = true; }
            6 if ok => { self.screen = Screen::Menu { cur: 9 }; return; }
            _ => {}
        }
        if changed { self.audio.sfx("select"); self.settings.apply_window(); self.settings.save(); self.apply_quality(); }
        self.ui.background();
        self.ui.text("VÍDEO", 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let on = |b: bool| if b { "LIGADO" } else { "DESLIGADO" };
        let q = QUALITY[self.settings.quality.min(QUALITY.len() - 1)];
        let items: [(usize, &str, String, &str); 7] = [
            (0, "Tela cheia", on(self.settings.fullscreen).into(), "Alterna também com F11"),
            (1, "Resolução da janela", RESOLUTIONS[self.settings.res.min(RESOLUTIONS.len() - 1)].2.into(), "Até 5K · o jogo escala a imagem para qualquer tamanho"),
            (2, "Qualidade da arte", q.1.into(), "Rasteriza lutadores e cenários em maior densidade de pixels (4K = Ultra)"),
            (3, "Pós-processamento", FX_LEVELS[self.settings.fx.min(2)].into(), "Bloom, aberração cromática, vinheta, correção de cor e blur radial"),
            (4, "Efeitos de impacto", IMPACT[self.settings.impact.min(IMPACT.len() - 1)].1.into(), "Tremor, zoom, flashes e linhas de velocidade — flashes grandes ficam limitados a 3/s; Mínimo desliga tudo (fotossensibilidade)"),
            (5, "Contador de FPS", on(self.settings.fps).into(), "Alterna também com F3"),
            (6, "Voltar", String::new(), ""),
        ];
        let mut y = 120.0;
        for (id, t, v, d) in items {
            if !rows.contains(&id) { continue; }
            let f = id == cur;
            let (x, w, h) = (160.0, 960.0, 62.0);
            if self.hover_rect(x, y, w, h) && mouse_delta_position().length() > 0.0 { cur = id; }
            if self.clicked_rect(x, y, w, h) { cur = id; self.screen = Screen::Video { cur }; if id == 6 { self.screen = Screen::Menu { cur: 9 }; return; } }
            self.ui.rect(x, y, w, h, if f { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.06) });
            self.ui.text(t, x + 24.0, y + 30.0, 26.0, WHITE, Align::Left, true);
            self.ui.text(d, x + 24.0, y + 52.0, 16.0, if f { WHITE } else { hex("#a49cc0") }, Align::Left, false);
            self.ui.text(&format!("◀ {v} ▶"), x + w - 24.0, y + 40.0, 24.0, hex("#ffcc33"), Align::Right, true);
            y += 72.0;
        }
        let (pw, ph) = ((screen_width() * macroquad::miniquad::window::dpi_scale()) as i32, Settings::physical_h() as i32);
        self.ui.text(&format!("Tela atual: {pw}×{ph} · arte rasterizada a {:.1}× · {} lutadores · {} cenários", ume_engine::native::rig::art_res(), self.reg.chars.len(), self.reg.stages.len()), 160.0, 690.0, 18.0, hex("#a49cc0"), Align::Left, false);
        if self.c.pressed(nav::BACK) { self.screen = Screen::Menu { cur: 9 }; return; }
        self.screen = Screen::Video { cur };
    }

    async fn fight(&mut self, dt: f32) {
        let Some(s) = self.session.as_mut() else { self.screen = Screen::Menu { cur: 0 }; return };
        if self.c.pressed(nav::AUX) { s.r.debug = !s.r.debug; self.debug_boxes = s.r.debug; }
        let pause_btn = self.c.pressed(nav::START) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P);
        if pause_btn { s.paused = !s.paused; self.pause_cur = 0; self.audio.sfx("select"); }
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
        let s = self.session.as_mut().unwrap();
        if s.paused { self.pause_menu().await; return; }
        if let Some(w) = s.over { if s.over_timer > 1.0 { if !self.team_next(w) {
                // keep the fight so it can be replayed (not for replays themselves or sequence team bouts)
                if self.replaying.is_none() && !self.team.as_ref().is_some_and(|t| !t.simul) { if let Some(s) = self.session.as_ref() { self.last_fight = Some(Replay { sel: self.sel, mode: self.mode, team: self.team.clone(), rec: s.rec.clone() }); } }
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

    async fn pause_menu(&mut self) {
        let items = ["Continuar", "Hitboxes", "Reiniciar luta", "Trocar personagens", "Menu principal"];
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
        if f.def.movelist.is_empty() { self.ui.text("Personagem MUGEN: comandos definidos no .cmd do autor", x + 400.0, y + 90.0, 18.0, WHITE, Align::Left, false); }
        for (i, mv) in f.def.movelist.iter().enumerate().take(14) {
            let yy = y + 90.0 + i as f32 * 29.0;
            self.ui.text(&mv.name, x + 400.0, yy, 18.0, WHITE, Align::Left, false);
            self.ui.text(&pretty_input(&mv.input), x + 1010.0, yy, 16.0, hex("#ffcc33"), Align::Right, true);
        }
        if self.c.pressed(nav::BACK) { self.session.as_mut().unwrap().paused = false; return; }
        if ok {
            match self.pause_cur {
                0 => self.session.as_mut().unwrap().paused = false,
                1 => { let s = self.session.as_mut().unwrap(); s.r.debug = !s.r.debug; self.debug_boxes = s.r.debug; }
                2 => { self.reset_team(); self.start_fight().await; }
                3 => { self.audio.stop_music(); self.session = None; self.team = None; self.team_pick.clear(); self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; }
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
        let title = if done { "ARCADE COMPLETO!".to_string() } else if self.team.is_some() { format!("EQUIPE {} VENCE!", e.name.split_whitespace().next().unwrap_or("").to_uppercase()) } else { format!("{} VENCE!", e.name.to_uppercase()) };
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
    pub fn capture_replay(&mut self) { if let Some(s) = self.session.as_ref() { self.last_fight = Some(Replay { sel: self.sel, mode: self.mode, team: self.team.clone(), rec: s.rec.clone() }); } }

    /// Re-simulate the last finished fight from its recorded inputs (deterministic engine → identical fight).
    pub async fn start_replay(&mut self) {
        let Some(r) = self.last_fight.clone() else { return };
        self.mode = r.mode; self.team = r.team.clone(); self.sel = r.sel;
        self.replaying = Some(r);
        self.start_fight().await;
    }

    fn controls_screen(&mut self) {
        self.ui.background();
        self.ui.text("CONTROLES", 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let col = |app: &App, x: f32, title: &str, lines: &[&str]| {
            app.ui.text(title, x, 120.0, 24.0, hex("#22e8ff"), Align::Left, true);
            for (i, l) in lines.iter().enumerate() { app.ui.text(l, x, 156.0 + i as f32 * 28.0, 19.0, WHITE, Align::Left, false); }
        };
        col(self, 40.0, "Teclado", &["Mover: WASD ou setas", "Socos: U fraco · I forte · O", "Chutes: J fraco · K forte · L", "Especial simples: 1 · 2 · 3", "Super simples: 1 + 2", "Pausa: Esc · Hitboxes: F2", "", "2 Jogadores (P2):", "Setas + Num4/5/6 · Num1/2/3"]);
        col(self, 450.0, "DualShock 4 / DualSense", &["□ soco fraco · △ soco forte · R1 soco 3", "✕ chute fraco · ○ chute forte · R2 chute 3", "L1 / L2: especiais simples", "OPTIONS: pausa", "Vibração: golpes recebidos, defesa,", "super e K.O. (USB e Bluetooth)", "Lightbar: P1 azul · P2 vermelho", "Xbox e genéricos também funcionam"]);
        col(self, 880.0, "Celular / Tablet", &["Analógico virtual à esquerda", "LP / HP / LK / HK à direita", "SP1–SP3 e SUPER com um toque", "(funcionam também com MUGEN)", "Vibração do aparelho no navegador"]);
        self.ui.text("Controles detectados:", 40.0, 470.0, 22.0, hex("#ffcc33"), Align::Left, true);
        let pads = self.c.pad_info.clone();
        if pads.is_empty() { self.ui.text("Nenhum controle conectado — conecte um DualShock/DualSense via USB ou Bluetooth.", 40.0, 504.0, 20.0, WHITE, Align::Left, false); }
        for (i, p) in pads.iter().enumerate().take(4) {
            let y = 504.0 + i as f32 * 34.0;
            self.ui.text(&format!("P{}: {} {}", i + 1, p.name, if p.sony { "(Sony)" } else { "" }), 40.0, y, 20.0, WHITE, Align::Left, false);
            if self.clicked_rect(700.0, y - 22.0, 220.0, 30.0) { self.c.rumble(i, 1.0, 1.0, 500); }
            self.ui.rect(700.0, y - 22.0, 220.0, 30.0, hex("#ff4d6d"));
            self.ui.text("Testar vibração", 810.0, y, 18.0, WHITE, Align::Center, true);
        }
        self.ui.text("Pressione ✕/A no controle para testar a vibração · Esc para voltar", 40.0, 690.0, 18.0, hex("#a49cc0"), Align::Left, false);
        for (i, p) in pads.iter().enumerate() { if p.bits & ume_engine::input::k::A != 0 && self.c.pressed(nav::OK) { self.c.rumble(i, 0.8, 0.8, 300); } }
        if self.c.pressed(nav::BACK) { self.screen = Screen::Menu { cur: 8 }; }
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

/// Numpad notation (2 = down, 6 = forward, 236 = quarter-circle forward).
pub fn pretty_input(s: &str) -> String {
    let mut dirs = String::new();
    let mut btn = String::new();
    for t in s.replace(['~', ' ', '/', '$'], "").split(',') {
        let d = match t { "D" => "2", "U" => "8", "F" => "6", "B" => "4", "DF" => "3", "DB" => "1", "UF" => "9", "UB" => "7", _ => "" };
        if !d.is_empty() { dirs.push_str(d); continue; }
        let mut parts = vec![];
        for p in t.split('+') {
            match p { "D" | "U" | "F" | "B" | "DF" | "DB" | "UF" | "UB" => {}, "x" => parts.push("LP"), "y" => parts.push("HP"), "z" => parts.push("P3"), "a" => parts.push("LK"), "b" => parts.push("HK"), "c" => parts.push("K3"), o => parts.push(if o.chars().last().is_some_and(|c| "DUFB".contains(c)) { "" } else { o }) }
        }
        btn = parts.join("+");
    }
    if dirs.is_empty() { btn } else { format!("{dirs} {btn}") }
}
