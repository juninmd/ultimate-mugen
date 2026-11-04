use super::*;
use crate::i18n::{tr, Lang, LANGS};
use crate::settings::{FPS_CAPS, LIFE_PERCENT, ROUNDS_TO_WIN, ROUND_SECONDS, VOLUME_STEPS};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Act { Fullscreen, Res, Quality, Fx, Impact, FpsCounter, Vsync, FpsCap, Master, Music, Sfx, Mute, Keys, Guide, Difficulty, RoundTime, Rounds, Life, Language, Reset, Info }

struct Row { act: Act, label: String, value: String, hint: String, level: Option<u8> }

const TABS: [(&str, &str); 6] = [("VÍDEO", "VIDEO"), ("ÁUDIO", "AUDIO"), ("CONTROLES", "CONTROLS"), ("JOGABILIDADE", "GAMEPLAY"), ("IDIOMA", "LANGUAGE"), ("SOBRE", "ABOUT")];

fn row(act: Act, label: &str, value: impl Into<String>, hint: &str) -> Row { Row { act, label: label.into(), value: value.into(), hint: hint.into(), level: None } }
fn cycle(v: usize, d: i32, len: usize) -> usize { (v as i32 + d).rem_euclid(len as i32) as usize }
fn on_off(b: bool) -> &'static str { if b { tr("LIGADO", "ON") } else { tr("DESLIGADO", "OFF") } }

impl App {
    fn settings_rows(&self, tab: usize) -> Vec<Row> {
        let s = &self.settings;
        let web = cfg!(target_arch = "wasm32");
        let mobile = cfg!(any(target_os = "android", target_os = "ios"));
        let level = |act, label: &str, v: u8, hint: &str| Row { act, label: label.into(), value: format!("{}%", v as u32 * 100 / VOLUME_STEPS as u32), hint: hint.into(), level: Some(v) };
        match tab {
            0 => {
                let mut r = vec![row(Act::Fullscreen, tr("Tela cheia", "Fullscreen"), on_off(s.fullscreen), tr("Alterna também com F11", "Also toggled with F11"))];
                if !web && !mobile { r.push(row(Act::Res, tr("Resolução da janela", "Window resolution"), RESOLUTIONS[s.res.min(RESOLUTIONS.len() - 1)].2, tr("Até 5K · a imagem escala para qualquer tamanho", "Up to 5K · the picture scales to any size"))); }
                r.push(row(Act::Quality, tr("Qualidade da arte", "Art quality"), QUALITY[s.quality.min(QUALITY.len() - 1)].1, tr("Densidade de pixels de lutadores e cenários", "Pixel density of fighters and stages")));
                r.push(row(Act::Fx, tr("Pós-processamento", "Post-processing"), FX_LEVELS[s.fx.min(FX_LEVELS.len() - 1)], tr("Bloom, vinheta, correção de cor e blur radial", "Bloom, vignette, colour grading and radial blur")));
                r.push(row(Act::Impact, tr("Efeitos de impacto", "Impact effects"), IMPACT[s.impact.min(IMPACT.len() - 1)].1, tr("Tremor, zoom e flashes · Mínimo desliga tudo", "Shake, zoom and flashes · Minimum turns them off")));
                if !web { r.push(row(Act::Vsync, "V-Sync", on_off(s.vsync), tr("Sincroniza com o monitor e evita rasgos de imagem", "Syncs with the display and avoids tearing"))); }
                if !web { r.push(row(Act::FpsCap, tr("Limite de FPS", "FPS cap"), match s.fps_cap_value() { 0 => tr("Sem limite", "Unlimited").to_string(), n => format!("{n} FPS") }, tr("Útil com V-Sync desligado para poupar bateria e calor", "Useful with V-Sync off to save battery and heat"))); }
                r.push(row(Act::FpsCounter, tr("Contador de FPS", "FPS counter"), on_off(s.fps), tr("Alterna também com F3", "Also toggled with F3")));
                r
            }
            1 => vec![
                level(Act::Master, tr("Volume geral", "Master volume"), s.master, tr("Controla música e efeitos juntos", "Scales music and effects together")),
                level(Act::Music, tr("Música", "Music"), s.music, tr("Trilha dos cenários", "Stage soundtrack")),
                level(Act::Sfx, tr("Efeitos e vozes", "Effects and voices"), s.sfx, tr("Golpes, vozes dos lutadores e menus", "Hits, fighter voices and menus")),
                row(Act::Mute, tr("Mudo", "Mute"), on_off(s.mute), tr("Silencia tudo sem perder os níveis", "Silences everything without losing the levels")),
            ],
            2 => {
                let pads = &self.c.pad_info;
                let mut r = vec![];
                if !self.c.touch.enabled { r.push(row(Act::Keys, tr("Teclado · remapear teclas", "Keyboard · rebind keys"), tr("Abrir", "Open"), tr("Jogador 1 e 2 · padrão: golpes Q W E / A S D, P1 setas, P2 I K J L", "Players 1 and 2 · default: attacks Q W E / A S D, P1 arrows, P2 I K J L"))); }
                r.push(row(Act::Guide, tr("Guia de controles", "Controls guide"), tr("Abrir", "Open"), tr("Teclado, Xbox, PlayStation e toque lado a lado", "Keyboard, Xbox, PlayStation and touch side by side")));
                let pad_line = if pads.is_empty() { tr("nenhum conectado", "none connected").to_string() } else { format!("{}", pads.len()) };
                r.push(row(Act::Info, tr("Controles conectados", "Connected gamepads"), pad_line, tr("O mapeamento dos controles segue o padrão de cada família (Xbox/PlayStation)", "Gamepad mapping follows each family's standard layout (Xbox/PlayStation)")));
                for (i, p) in pads.iter().enumerate().take(2) { r.push(row(Act::Info, &format!("P{}", i + 1), p.name.chars().take(34).collect::<String>(), "")); }
                r
            }
            3 => vec![
                row(Act::Difficulty, tr("Dificuldade da CPU", "CPU difficulty"), format!("{} / 8", s.difficulty), tr("Vale para Arcade, Versus CPU e Equipes", "Applies to Arcade, Versus CPU and Teams")),
                row(Act::RoundTime, tr("Tempo do round", "Round time"), match ROUND_SECONDS[s.round_time.min(ROUND_SECONDS.len() - 1)] { 0 => tr("Infinito", "Infinite").to_string(), n => format!("{n} s") }, tr("Vale a partir da próxima luta", "Takes effect from the next fight")),
                row(Act::Rounds, tr("Rounds para vencer", "Rounds to win"), format!("{} {}", tr("Melhor de", "Best of"), ROUNDS_TO_WIN[s.rounds.min(ROUNDS_TO_WIN.len() - 1)] * 2 - 1), tr("Vitórias necessárias: 1, 2 ou 3 rounds", "Wins needed: 1, 2 or 3 rounds")),
                row(Act::Life, tr("Vida dos lutadores", "Fighter life"), format!("{}%", LIFE_PERCENT[s.life.min(LIFE_PERCENT.len() - 1)]), tr("Multiplica a barra de vida de todos", "Scales everyone's life bar")),
            ],
            4 => vec![row(Act::Language, tr("Idioma", "Language"), LANGS.iter().find(|(l, _)| *l == s.lang).map(|(_, n)| *n).unwrap_or(""), tr("Menus de configurações e tela de versus. O restante é traduzido gradualmente", "Settings and versus screens. The rest is being translated gradually"))],
            _ => vec![row(Act::Reset, tr("Restaurar padrões", "Restore defaults"), tr("Enter", "Enter"), tr("Volta todas as configurações ao original", "Resets every setting to its original value"))],
        }
    }

    fn settings_adjust(&mut self, act: Act, dir: i32, ok: bool) -> bool {
        let step = if dir == 0 { 1 } else { dir };
        let go = ok || dir != 0;
        let s = &mut self.settings;
        let mut changed = true;
        match act {
            Act::Fullscreen if go => s.fullscreen = !s.fullscreen,
            Act::Res if go => { s.res = cycle(s.res, step, RESOLUTIONS.len()); s.fullscreen = false; }
            Act::Quality if go => s.quality = cycle(s.quality, step, QUALITY.len()),
            Act::Fx if go => s.fx = cycle(s.fx, step, FX_LEVELS.len()),
            Act::Impact if go => s.impact = cycle(s.impact, step, IMPACT.len()),
            Act::FpsCounter if go => s.fps = !s.fps,
            Act::Vsync if go => s.vsync = !s.vsync,
            Act::FpsCap if go => s.fps_cap = cycle(s.fps_cap, step, FPS_CAPS.len()),
            Act::Master if go => s.master = cycle(s.master as usize, step, VOLUME_STEPS as usize + 1) as u8,
            Act::Music if go => s.music = cycle(s.music as usize, step, VOLUME_STEPS as usize + 1) as u8,
            Act::Sfx if go => s.sfx = cycle(s.sfx as usize, step, VOLUME_STEPS as usize + 1) as u8,
            Act::Mute if go => s.mute = !s.mute,
            Act::Difficulty if go => { s.difficulty = (s.difficulty + step).clamp(1, 8); self.ai_level = s.difficulty; }
            Act::RoundTime if go => s.round_time = cycle(s.round_time, step, ROUND_SECONDS.len()),
            Act::Rounds if go => s.rounds = cycle(s.rounds, step, ROUNDS_TO_WIN.len()),
            Act::Life if go => s.life = cycle(s.life, step, LIFE_PERCENT.len()),
            Act::Language if go => s.lang = if s.lang == Lang::En { Lang::Pt } else { Lang::En },
            Act::Reset if ok => { *s = Settings::default(); self.ai_level = s.difficulty; self.msg = tr("Configurações restauradas", "Settings restored").into(); }
            Act::Keys if ok => { self.key_edit = Some(keyedit::KeyEdit { slot: 0, cur: 0, waiting: false, msg: String::new() }); self.screen = Screen::Controls; changed = false; }
            Act::Guide if ok => { self.screen = Screen::Controls; changed = false; }
            _ => changed = false,
        }
        changed
    }

    pub(super) fn settings_screen(&mut self) {
        let Screen::Settings { mut tab, mut cur, from_fight } = self.screen else { return };
        let rows = self.settings_rows(tab);
        let usable: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].act != Act::Info).collect();
        let max_cur = usable.len();
        cur = cur.min(max_cur);

        if self.c.pressed(nav::DOWN) { cur = (cur + 1) % (max_cur + 1); self.audio.sfx("select"); }
        if self.c.pressed(nav::UP) { cur = (cur + max_cur) % (max_cur + 1); self.audio.sfx("select"); }
        let dir = (self.c.pressed(nav::RIGHT) as i32) - (self.c.pressed(nav::LEFT) as i32);
        let mut ok = self.c.pressed(nav::OK | nav::START);
        let mut dir_v = dir;

        self.ui.background();
        self.ui.rect(0.0, 0.0, VW, 118.0, Color::new(0.0, 0.0, 0.0, 0.35));
        self.ui.text(tr("CONFIGURAÇÕES", "SETTINGS"), 56.0, 62.0, 38.0, hex("#ffcc33"), Align::Left, true);
        self.ui.text(&format!("Ultimate MUGEN  v{}", crate::update::VERSION), VW - 56.0, 58.0, 20.0, hex("#a49cc0"), Align::Right, false);

        let tab_w = (VW - 112.0) / TABS.len() as f32;
        for (i, (pt, en)) in TABS.iter().enumerate() {
            let x = 56.0 + i as f32 * tab_w;
            let active = i == tab;
            let focused = active && cur == 0;
            if self.clicked_rect(x, 80.0, tab_w, 38.0) { tab = i; cur = 0; self.audio.sfx("select"); }
            let col = if focused { hex("#ff4d6d") } else if active { Color::new(1.0, 0.3, 0.43, 0.38) } else { Color::new(1.0, 1.0, 1.0, 0.06) };
            self.ui.quad([vec2(x + 10.0, 80.0), vec2(x + tab_w, 80.0), vec2(x + tab_w - 10.0, 118.0), vec2(x, 118.0)], col);
            self.ui.text(tr(pt, en), x + tab_w / 2.0, 106.0, 18.0, if active { WHITE } else { hex("#a49cc0") }, Align::Center, true);
        }
        if cur == 0 {
            if dir != 0 { tab = cycle(tab, dir, TABS.len()); self.audio.sfx("select"); dir_v = 0; ok = false; }
        }

        let rows = self.settings_rows(tab);
        let usable: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].act != Act::Info).collect();
        let cur = cur.min(usable.len());
        let focused_row = if cur == 0 { None } else { usable.get(cur - 1).copied() };
        self.ui.rect(120.0, 140.0, VW - 240.0, 520.0, Color::new(0.08, 0.05, 0.16, 0.72));

        let mut act_now: Option<(Act, i32, bool)> = focused_row.map(|i| (rows[i].act, dir_v, ok));
        let (x, w, h) = (150.0, VW - 300.0, 54.0);
        let mut y = 156.0;
        let mut new_cur = cur;
        for (i, r) in rows.iter().enumerate() {
            let f = focused_row == Some(i);
            let interactive = r.act != Act::Info;
            if interactive {
                let pos = usable.iter().position(|&u| u == i).unwrap_or(0) + 1;
                if self.hover_rect(x, y, w, h) && mouse_delta_position().length() > 0.0 { new_cur = pos; }
                if self.clicked_rect(x, y, w, h) {
                    new_cur = pos;
                    let p = self.ui.to_virtual(mouse_position().into());
                    let value_zone = x + w - 360.0;
                    let click_dir = if r.level.is_some() || p.x > value_zone { if p.x < x + w - 180.0 { -1 } else { 1 } } else { 0 };
                    act_now = Some((r.act, click_dir, click_dir == 0));
                }
            }
            self.ui.rect(x, y, w, h, if f { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, if interactive { 0.07 } else { 0.03 }) });
            self.ui.text(&r.label, x + 22.0, y + 25.0, 22.0, if interactive { WHITE } else { hex("#a49cc0") }, Align::Left, true);
            self.ui.text(&r.hint, x + 22.0, y + 45.0, 14.0, if f { WHITE } else { hex("#a49cc0") }, Align::Left, false);
            if let Some(v) = r.level {
                let bx = x + w - 360.0;
                for n in 0..VOLUME_STEPS {
                    let on = n < v && !self.settings.mute;
                    self.ui.rect(bx + n as f32 * 22.0, y + 17.0, 17.0, 20.0, if on { hex("#ffcc33") } else { Color::new(1.0, 1.0, 1.0, 0.14) });
                }
                self.ui.text(&r.value, x + w - 22.0, y + 35.0, 20.0, WHITE, Align::Right, true);
            } else if !r.value.is_empty() {
                let shown = if matches!(r.act, Act::Keys | Act::Guide | Act::Reset) { r.value.clone() } else if interactive { format!("‹ {} ›", r.value) } else { r.value.clone() };
                self.ui.text(&shown, x + w - 22.0, y + 35.0, 22.0, if f { WHITE } else { hex("#ffcc33") }, Align::Right, true);
            }
            y += h + 8.0;
        }

        if tab == 5 {
            let (nm, nn) = (self.reg.chars.iter().filter(|c| c.mugen).count(), self.reg.chars.len());
            let lines = [
                format!("{} v{}", tr("Versão da engine", "Engine version"), crate::update::VERSION),
                format!("{} {} · {} {} · {} MUGEN", nn, tr("lutadores", "fighters"), self.reg.stages.len(), tr("cenários", "stages"), nm),
                format!("{}: {}", tr("Plataforma", "Platform"), if cfg!(target_arch = "wasm32") { "Web (WebGL2)" } else if cfg!(target_os = "android") { "Android" } else if cfg!(target_os = "ios") { "iOS" } else { "Desktop (wgpu)" }),
                tr("Inspirada em M.U.G.E.N (Elecbyte) e Ikemen GO. Código aberto em github.com/juninmd/ultimate-mugen", "Inspired by M.U.G.E.N (Elecbyte) and Ikemen GO. Open source at github.com/juninmd/ultimate-mugen").to_string(),
            ];
            for (i, l) in lines.iter().enumerate() { self.ui.text(l, 150.0 + 22.0, 300.0 + i as f32 * 34.0, if i == 0 { 26.0 } else { 19.0 }, if i == 0 { hex("#ffcc33") } else { WHITE }, Align::Left, i == 0); }
        }

        let back = self.clicked_rect(56.0, 668.0, 150.0, 40.0);
        self.ui.rect(56.0, 668.0, 150.0, 40.0, Color::new(1.0, 1.0, 1.0, 0.1));
        self.ui.text(tr("Voltar", "Back"), 131.0, 696.0, 20.0, WHITE, Align::Center, true);
        let note = if from_fight { tr("Em luta: regras de jogabilidade valem na próxima luta", "In a fight: gameplay rules apply from the next fight") } else { tr("Cima/Baixo: navegar · Esq/Dir: alterar · Enter: confirmar · Esc: voltar", "Up/Down: navigate · Left/Right: change · Enter: confirm · Esc: back") };
        self.ui.text(note, VW - 56.0, 696.0, 16.0, hex("#a49cc0"), Align::Right, false);

        let mut changed = false;
        if let Some((a, d, k)) = act_now { if cur != 0 || new_cur != 0 { changed = self.settings_adjust(a, d, k); } }
        if changed { self.audio.sfx("select"); self.settings.apply_window(); self.settings.save(); if !from_fight { self.apply_quality(); } }

        if matches!(self.screen, Screen::Controls) { self.settings_back = Some((tab, from_fight)); return; }
        if back || self.c.pressed(nav::BACK) { self.screen = if from_fight { Screen::Fight } else { Screen::Menu { cur: 10 } }; return; }
        self.screen = Screen::Settings { tab, cur: new_cur, from_fight };
    }
}
