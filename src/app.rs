//! Screens and game flow.
use crate::audio::Audio;
use crate::controls::{nav, Controls};
use crate::registry::Registry;
use crate::render::{Gfx, MatchRenderer, DrawOpts};
use crate::session::{Pilot, Session};
use crate::ui::*;
use macroquad::prelude::*;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode { Arcade, Cpu, Versus, Training, Watch }
impl Mode { fn label(self) -> &'static str { match self { Mode::Arcade => "ARCADE", Mode::Cpu => "VERSUS CPU", Mode::Versus => "2 JOGADORES", Mode::Training => "TREINO", Mode::Watch => "CPU vs CPU" } } }

pub enum Screen {
    Title,
    Menu { cur: usize },
    Select { cur: [usize; 2], picking: usize, picked: [Option<usize>; 2] },
    Stages { cur: usize },
    Vs { t: f32 },
    Fight,
    Results { winner: usize, cur: usize },
    Controls,
    Mugen,
}

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
}

const MENU: [(&str, &str); 8] = [
    ("Arcade", "Enfrente uma sequência de oponentes"),
    ("Versus CPU", "Você contra o computador"),
    ("2 Jogadores", "Teclado compartilhado ou dois controles"),
    ("Treino", "Boneco de treino, barra infinita, hitboxes"),
    ("CPU vs CPU", "Assista a IA lutar"),
    ("Personagens MUGEN", "Como adicionar personagens e cenários"),
    ("Controles", "Teclado, DualShock/DualSense, toque e vibração"),
    ("Sair", "Fechar o jogo"),
];

impl App {
    pub async fn new(touch: bool, audio_on: bool) -> Self {
        let mut reg = Registry::new();
        #[cfg(not(target_arch = "wasm32"))]
        reg.scan_folders();
        App {
            reg, ui: Ui::new(), g: Gfx::new(), c: Controls::new(touch), audio: Audio::new(audio_on).await, screen: Screen::Title, mode: Mode::Cpu,
            ai_level: 4, sel: (0, 1, 0), session: None, arcade: None, pause_cur: 0, debug_boxes: false, quit: false, msg: String::new(), preview_t: 0.0,
        }
    }

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
        self.ui.begin();
        self.c.update();
        self.preview_t += get_frame_time();
        let dt = get_frame_time();
        match &mut self.screen {
            Screen::Title => self.title(),
            Screen::Menu { .. } => self.menu(),
            Screen::Select { .. } => self.select(),
            Screen::Stages { .. } => self.stages(),
            Screen::Vs { .. } => self.vs(dt).await,
            Screen::Fight => self.fight(dt).await,
            Screen::Results { .. } => self.results().await,
            Screen::Controls => self.controls_screen(),
            Screen::Mugen => self.mugen_screen(),
        }
        self.audio.process().await;
        for (i, zip) in crate::take_imports().into_iter().enumerate() {
            let mut vfs = ume_formats::vfs::Vfs::new();
            match ume_formats::vfs::add_zip(&mut vfs, &zip, "") {
                Ok(_) => { let (c, s, e) = self.reg.add_vfs(vfs, &format!("web{}-{i}", get_time() as u64)); self.msg = format!("Importado: {} personagem(ns), {} cenário(s){}", c.len(), s.len(), if e.is_empty() { String::new() } else { format!(" · {} erro(s)", e.len()) }); }
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
            let (x, y, w, h) = (460.0, 120.0 + i as f32 * 64.0, 440.0, 56.0);
            if self.hover_rect(x, y, w, h) && (mouse_delta_position().length() > 0.0) { cur_v = i; }
            if self.clicked_rect(x, y, w, h) { chosen = Some(i); }
            let f = i == cur_v;
            let off = if f { 14.0 } else { 0.0 };
            let col = if f { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.06) };
            self.ui.quad([vec2(x + off + 14.0, y), vec2(x + off + w, y), vec2(x + off + w - 14.0, y + h), vec2(x + off, y + h)], col);
            if f { self.ui.quad([vec2(x + off + w - 60.0, y), vec2(x + off + w, y), vec2(x + off + w - 14.0, y + h), vec2(x + off + w - 74.0, y + h)], hex("#8a2be2")); }
            self.ui.text(t, x + off + 26.0, y + 30.0, 24.0, WHITE, Align::Left, true);
            self.ui.text(d, x + off + 26.0, y + 49.0, 16.0, if f { WHITE } else { hex("#a49cc0") }, Align::Left, false);
        }
        let (nm, nn) = (self.reg.chars.iter().filter(|c| c.mugen).count(), self.reg.chars.len());
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
            let mode = [Mode::Arcade, Mode::Cpu, Mode::Versus, Mode::Training, Mode::Watch];
            match i {
                0..=4 => { self.mode = mode[i]; self.go(Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }); }
                5 => self.go(Screen::Mugen),
                6 => self.go(Screen::Controls),
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
        draw_texture_ex(&tex, left, py - spr.axis_y * ks, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_x: flip, ..Default::default() });
    }

    fn select(&mut self) {
        let Screen::Select { mut cur, mut picking, mut picked } = self.screen else { return };
        let n = self.reg.chars.len();
        let cols = 6usize;
        let two = self.mode == Mode::Versus;
        let mut confirm = false;
        // input: P1 picks with menu nav; P2 in versus uses pad 2 / arrows
        let p = picking;
        if self.c.pressed(nav::RIGHT) { cur[p] = (cur[p] + 1) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::LEFT) { cur[p] = (cur[p] + n - 1) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::DOWN) { cur[p] = (cur[p] + cols) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::UP) { cur[p] = (cur[p] + n * cols - cols) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::OK | nav::START) { confirm = true; }
        let back = self.c.pressed(nav::BACK);
        self.ui.background();
        self.ui.text(self.mode.label(), 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let hint = if picking == 0 { "JOGADOR 1: escolha seu lutador" } else if two { "JOGADOR 2: escolha" } else { "Escolha o oponente" };
        self.ui.text(hint, VW - 40.0, 56.0, 22.0, hex("#a49cc0"), Align::Right, false);
        // grid
        let (gx, gy, cw, ch, gap) = (330.0, 110.0, 96.0, 112.0, 8.0);
        let mut click = None;
        for i in 0..n {
            let (cx, cy) = (gx + (i % cols) as f32 * (cw + gap), gy + (i / cols) as f32 * (ch + gap));
            if self.clicked_rect(cx, cy, cw, ch) { click = Some(i); }
            let (ecol, emugen, ename) = { let e = &self.reg.chars[i]; (e.color, e.mugen, e.name.clone()) };
            let col = rgb(ecol);
            self.ui.gradient_v(cx, cy, cw, ch, alpha(col, 0.95), hex("#120a22"));
            // portrait: idle frame head crop
            if let Ok(d) = self.reg.chars[i].load(1) {
                let por = d.sprites.get(&(9000, 1)).or(d.sprites.get(&(9000, 0))).filter(|_| emugen).and_then(|p| p.image.clone().map(|im| (im, p.width, p.height)));
                if let Some((im, pw, ph)) = por {
                    let tex = self.g.texture(&im, true);
                    let k = ((cw - 8.0) / pw).min((ch - 26.0) / ph);
                    draw_texture_ex(&tex, self.ui.x(cx + (cw - pw * k) / 2.0), self.ui.y(cy + 4.0), WHITE, DrawTextureParams { dest_size: Some(vec2(pw * k * self.ui.s, ph * k * self.ui.s)), ..Default::default() });
                } else {
                    self.ui_clip(cx, cy, cw, ch - 18.0, |app| app.draw_char(&d, 0, 0.0, cx + cw / 2.0, cy + ch * 1.55, 260.0, false));
                }
            }
            self.ui.rect(cx, cy + ch - 18.0, cw, 18.0, Color::new(0.0, 0.0, 0.0, 0.65));
            let nm: String = ename.split_whitespace().next().unwrap_or("").chars().take(12).collect();
            self.ui.text(&nm.to_uppercase(), cx + cw / 2.0, cy + ch - 4.0, 12.0, WHITE, Align::Center, true);
            if emugen { self.ui.rect(cx + cw - 44.0, cy + 3.0, 41.0, 13.0, hex("#22e8ff")); self.ui.text("MUGEN", cx + cw - 23.5, cy + 14.0, 10.0, BLACK, Align::Center, true); }
            let c1 = cur[0] == i && (picking == 0 || picked[0].is_some());
            let c2 = cur[1] == i && picking == 1;
            if c1 { self.ui.rect_lines(cx - 2.0, cy - 2.0, cw + 4.0, ch + 4.0, 4.0, hex("#3fa9ff")); }
            if c2 { self.ui.rect_lines(cx - 6.0, cy - 6.0, cw + 12.0, ch + 12.0, 4.0, hex("#ff4d6d")); }
        }
        if let Some(i) = click { if cur[picking] == i { confirm = true; } else { cur[picking] = i; self.audio.sfx("select"); } }
        // previews
        for side in 0..2 {
            let idx = picked[side].or(if side == picking { Some(cur[side]) } else { None });
            let x = if side == 0 { 160.0 } else { VW - 160.0 };
            let col = if side == 0 { Color::new(0.25, 0.66, 1.0, 0.18) } else { Color::new(1.0, 0.3, 0.43, 0.18) };
            glow(self.ui.x(x), self.ui.y(540.0), 200.0 * self.ui.s, Color::new(col.r, col.g, col.b, 0.9));
            draw_ellipse(self.ui.x(x), self.ui.y(600.0), 80.0 * self.ui.s, 12.0 * self.ui.s, 0.0, Color::new(0.0, 0.0, 0.0, 0.4));
            if let Some(i) = idx {
                if let Ok(d) = self.reg.chars[i].load(if side == 1 && picked[0] == Some(i) { 2 } else { 1 }) {
                    let anim = if picked[side].is_some() && d.anims.contains_key(&181) { 181 } else { 0 };
                    let t = self.preview_t;
                    self.draw_char(&d, anim, t, x, 600.0, 330.0, side == 1);
                }
                let e = &self.reg.chars[i];
                self.ui.text_bold(&e.name.to_uppercase(), x, 650.0, if e.name.len() > 16 { 20.0 } else { 26.0 }, if picked[side].is_some() { hex("#ffcc33") } else { WHITE }, BLACK, Align::Center);
                self.ui.text(&format!("{} · {}", e.series, e.author), x, 676.0, 16.0, hex("#a49cc0"), Align::Center, false);
            } else { self.ui.text_bold("???", x, 650.0, 30.0, hex("#a49cc0"), BLACK, Align::Center); }
        }
        let desc = &self.reg.chars[cur[picking]].description;
        self.ui.text(desc, VW / 2.0, 700.0, 18.0, hex("#d8d0f0"), Align::Center, false);
        if confirm {
            self.audio.sfx("confirm");
            picked[picking] = Some(cur[picking]);
            if picking == 0 && self.mode == Mode::Arcade {
                let me = cur[0];
                let mut ladder: Vec<usize> = (0..n).filter(|&i| i != me && !self.reg.chars[i].mugen).collect();
                let seed = (get_time() * 1000.0) as usize;
                let ll = ladder.len().max(1);
                ladder.rotate_left(seed % ll);
                ladder.truncate(6);
                self.sel = (me, ladder[0], self.sel.2);
                self.arcade = Some((ladder, 0));
                self.screen = Screen::Stages { cur: 1 };
                return;
            }
            if picking == 0 { picking = 1; } else { self.sel = (picked[0].unwrap(), picked[1].unwrap(), self.sel.2); self.screen = Screen::Stages { cur: self.sel.2 + 1 }; return; }
        }
        if back { if picking == 1 { picking = 0; picked = [None, None]; } else { self.screen = Screen::Menu { cur: 0 }; return; } }
        self.screen = Screen::Select { cur, picking, picked };
    }

    fn ui_clip(&mut self, x: f32, y: f32, w: f32, h: f32, f: impl FnOnce(&mut App)) {
        let (sx, sy, sw, sh) = (self.ui.x(x), self.ui.y(y), w * self.ui.s, h * self.ui.s);
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        gl.quad_gl.scissor(Some((sx as i32, (screen_height() - sy - sh) as i32 * 0 + sy as i32, sw as i32, sh as i32)));
        f(self);
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        gl.quad_gl.scissor(None);
    }

    fn stages(&mut self) {
        let Screen::Stages { mut cur } = self.screen else { return };
        let n = self.reg.stages.len() + 1;
        if self.c.pressed(nav::RIGHT) { cur = (cur + 1) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::LEFT) { cur = (cur + n - 1) % n; self.audio.sfx("select"); }
        if self.c.pressed(nav::DOWN) { cur = (cur + 3).min(n - 1); }
        if self.c.pressed(nav::UP) { cur = cur.saturating_sub(3); }
        let mut ok = self.c.pressed(nav::OK | nav::START);
        self.ui.background();
        self.ui.text("ESCOLHA O CENÁRIO", 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let (cw, ch) = (380.0, 214.0);
        for i in 0..n {
            let (x, y) = (40.0 + (i % 3) as f32 * (cw + 22.0), 100.0 + (i / 3) as f32 * (ch + 22.0));
            if y > VH { continue; }
            if self.clicked_rect(x, y, cw, ch) { if cur == i { ok = true; } cur = i; }
            let (name, th, mugen) = if i == 0 { ("Aleatório".to_string(), [[255, 47, 208], [34, 232, 255], [255, 226, 58]], false) } else { let s = &self.reg.stages[i - 1]; (s.name.clone(), s.thumb, s.mugen) };
            self.ui.gradient_v(x, y, cw, ch * 0.5, rgb(th[0]), rgb(th[1]));
            self.ui.gradient_v(x, y + ch * 0.5, cw, ch * 0.5, rgb(th[1]), rgb(th[2]));
            self.ui.rect(x, y + ch - 34.0, cw, 34.0, Color::new(0.0, 0.0, 0.0, 0.6));
            self.ui.text(&name, x + 12.0, y + ch - 10.0, 18.0, WHITE, Align::Left, true);
            if mugen { self.ui.rect(x + cw - 60.0, y + 6.0, 54.0, 16.0, hex("#22e8ff")); self.ui.text("MUGEN", x + cw - 33.0, y + 19.0, 12.0, BLACK, Align::Center, true); }
            if cur == i { self.ui.rect_lines(x - 3.0, y - 3.0, cw + 6.0, ch + 6.0, 4.0, hex("#ffcc33")); }
        }
        if ok {
            self.sel.2 = if cur == 0 { (get_time() * 7.0) as usize % self.reg.stages.len() } else { cur - 1 };
            self.go(Screen::Vs { t: 0.0 });
            return;
        }
        if self.c.pressed(nav::BACK) { self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; return; }
        self.screen = Screen::Stages { cur };
    }

    async fn vs(&mut self, dt: f32) {
        let Screen::Vs { t } = &mut self.screen else { return };
        *t += dt;
        let t = *t;
        let (a, b) = (self.sel.0, self.sel.1);
        let k = (t / 0.4).min(1.0);
        clear_background(BLACK);
        let ca = rgb(self.reg.chars[a].color); let cb = rgb(self.reg.chars[b].color);
        self.ui.gradient_v(-(1.0 - k) * 640.0, 0.0, 640.0, VH, ca, BLACK);
        self.ui.gradient_v(640.0 + (1.0 - k) * 640.0, 0.0, 640.0, VH, cb, BLACK);
        let pt = self.preview_t;
        if let Ok(d) = self.reg.chars[a].load(1) { self.draw_char(&d, 0, pt, 320.0 - (1.0 - k) * 640.0, 560.0, 420.0, false); }
        if let Ok(d) = self.reg.chars[b].load(if a == b { 2 } else { 1 }) { self.draw_char(&d, 0, pt, 960.0 + (1.0 - k) * 640.0, 560.0, 420.0, true); }
        self.ui.text_bold(&self.reg.chars[a].name.to_uppercase(), 320.0, 630.0, 40.0, WHITE, BLACK, Align::Center);
        self.ui.text_bold(&self.reg.chars[b].name.to_uppercase(), 960.0, 630.0, 40.0, WHITE, BLACK, Align::Center);
        self.ui.text(&self.reg.chars[a].series, 320.0, 662.0, 20.0, hex("#dddddd"), Align::Center, false);
        self.ui.text(&self.reg.chars[b].series, 960.0, 662.0, 20.0, hex("#dddddd"), Align::Center, false);
        let vs_s = 170.0 * (if t < 0.6 { 3.0 - 2.0 * (t / 0.6) } else { 1.0 });
        glow(self.ui.x(640.0), self.ui.y(330.0), 220.0 * self.ui.s, Color::new(1.0, 0.5, 0.1, 0.7));
        self.ui.text_bold("VS", 640.0, 330.0 + vs_s * 0.35, vs_s, hex("#ffcc33"), hex("#5a0a10"), Align::Center);
        let sub = if let Some((l, i)) = &self.arcade { format!("ARCADE · LUTA {}/{}", i + 1, l.len()) } else { self.reg.stages[self.sel.2].name.clone() };
        self.ui.text(&sub, 640.0, 700.0, 20.0, hex("#a49cc0"), Align::Center, true);
        if t > 2.2 || self.c.pressed(nav::OK | nav::START) || is_mouse_button_pressed(MouseButton::Left) { self.start_fight().await; }
    }

    pub async fn start_fight(&mut self) {
        let (a, b, s) = self.sel;
        let d1 = match self.reg.chars[a].load(1) { Ok(d) => d, Err(e) => { self.msg = e; self.screen = Screen::Menu { cur: 0 }; return; } };
        let d2 = match self.reg.chars[b].load(if a == b { 2 } else { 1 }) { Ok(d) => d, Err(e) => { self.msg = e; self.screen = Screen::Menu { cur: 0 }; return; } };
        let stage = self.reg.build_stage(s);
        let music = stage.music;
        let lvl = match &self.arcade { Some((_, i)) if self.mode == Mode::Arcade => (2 + *i as i32 + self.ai_level / 3).min(8), _ => self.ai_level };
        let pilots = match self.mode {
            Mode::Watch => [Pilot::Cpu(Ai::new(lvl, 11)), Pilot::Cpu(Ai::new(lvl, 77))],
            Mode::Versus => [Pilot::Human(0), Pilot::Human(1)],
            Mode::Training => [Pilot::Human(0), Pilot::Cpu(Ai::new(0, 3))],
            _ => [Pilot::Human(0), Pilot::Cpu(Ai::new(lvl, 99))],
        };
        let mut sess = Session::new(d1, d2, stage, pilots, self.mode == Mode::Training, self.mode == Mode::Versus);
        sess.r.debug = self.debug_boxes;
        // DualShock/DualSense lightbars: P1 blue, P2 red
        self.c.pads.set_light(0, [30, 90, 255]);
        self.c.pads.set_light(1, [255, 40, 60]);
        self.session = Some(sess);
        self.g.trim(4000);
        self.screen = Screen::Fight;
        self.audio.play_music(music).await;
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
        s.update(dt, &mut self.c, &mut self.audio);
        s.draw(&mut self.g, &self.ui);
        self.c.touch.draw();
        if self.c.touch.enabled { self.ui.rect(pb.0, pb.1, pb.2, pb.3, Color::new(0.0, 0.0, 0.0, 0.35)); self.ui.text("II", VW / 2.0, pb.1 + 30.0, 20.0, WHITE, Align::Center, true); }
        let s = self.session.as_mut().unwrap();
        if s.paused { self.pause_menu().await; return; }
        if let Some(w) = s.over { if s.over_timer > 1.0 { self.screen = Screen::Results { winner: w, cur: 0 }; } }
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
                2 => { self.start_fight().await; }
                3 => { self.audio.stop_music(); self.session = None; self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; }
                _ => { self.audio.stop_music(); self.session = None; self.screen = Screen::Menu { cur: 0 }; }
            }
        }
    }

    async fn results(&mut self) {
        let Screen::Results { winner, mut cur } = self.screen else { return };
        if let Some(s) = self.session.as_mut() { s.draw(&mut self.g, &self.ui); }
        // arcade progression
        if self.mode == Mode::Arcade && winner == 0 {
            if let Some((ladder, i)) = self.arcade.as_mut() {
                *i += 1;
                if *i < ladder.len() { self.sel.1 = ladder[*i]; self.sel.2 = (*i * 3 + 1) % self.reg.stages.len(); self.audio.stop_music(); self.screen = Screen::Vs { t: 0.0 }; return; }
            }
        }
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.02, 0.01, 0.06, 0.6));
        let w = if winner == 0 { self.sel.0 } else { self.sel.1 };
        let e = &self.reg.chars[w];
        self.ui.gradient_v(160.0, 200.0, 960.0, 300.0, alpha(rgb(e.color), 0.9), Color::new(0.08, 0.05, 0.16, 0.95));
        let done = self.mode == Mode::Arcade && self.arcade.as_ref().is_some_and(|(l, i)| *i >= l.len());
        let title = if done { "ARCADE COMPLETO!".to_string() } else { format!("{} VENCE!", e.name.to_uppercase()) };
        let pal = if winner == 1 && self.sel.0 == self.sel.1 { 2 } else { 1 };
        if let Ok(d) = e.load(pal) { let t = self.preview_t; self.draw_char(&d, if d.anims.contains_key(&181) { 181 } else { 0 }, t, 330.0, 480.0, 260.0, false); }
        self.ui.text_bold(&title, 480.0, 300.0, 44.0, hex("#ffcc33"), BLACK, Align::Left);
        let items = ["Revanche", "Personagens", "Menu"];
        if self.c.pressed(nav::RIGHT) { cur = (cur + 1) % 3; }
        if self.c.pressed(nav::LEFT) { cur = (cur + 2) % 3; }
        let mut ok = self.c.pressed(nav::OK | nav::START);
        for (i, it) in items.iter().enumerate() {
            let (bx, by) = (480.0 + i as f32 * 210.0, 380.0);
            if self.clicked_rect(bx, by, 190.0, 50.0) { cur = i; ok = true; }
            self.ui.rect(bx, by, 190.0, 50.0, if i == cur { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.08) });
            self.ui.text(it, bx + 95.0, by + 33.0, 22.0, WHITE, Align::Center, true);
        }
        self.screen = Screen::Results { winner, cur };
        if ok {
            self.audio.stop_music();
            match cur {
                0 => { if let Some((_, i)) = self.arcade.as_mut() { *i = (*i).min(5); } self.start_fight().await; }
                1 => { self.session = None; self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] }; }
                _ => { self.session = None; self.screen = Screen::Menu { cur: 0 }; }
            }
        }
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
        if self.c.pressed(nav::BACK) { self.screen = Screen::Menu { cur: 6 }; }
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
        if self.c.pressed(nav::BACK | nav::OK) { self.screen = Screen::Menu { cur: 5 }; }
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
