use super::*;
use crate::controls::nav;
use crate::i18n::{tr3, Lang};
use crate::story::script::StoryScript;
use crate::story::studio::{self, BEATS, RULES};

const RULE_ROWS: usize = RULES.len();
const STAGE_ROW: usize = RULE_ROWS;
const TITLE_ROW: usize = STAGE_ROW + 1;
const BOSS_ROW: usize = STAGE_ROW + 2;
const FOES_ROW: usize = STAGE_ROW + 3;
const BEAT_ROW: usize = STAGE_ROW + 4;
const RESET_ROW: usize = BEAT_ROW + BEATS;
const EPILOGUE_ROW: usize = RESET_ROW + 1;
const ROWS: usize = EPILOGUE_ROW + 1;
const ROW_TOP: f32 = 142.0;
const ROW_H: f32 = 33.0;
const VALUE_W: f32 = 700.0;

pub(crate) struct StudioState {
    script: StoryScript,
    stage: usize,
    edit: Option<String>,
    note: String,
}

impl StudioState {
    pub(crate) fn open() -> Self {
        Self {
            script: crate::story::active_script().clone(),
            stage: 0,
            edit: None,
            note: String::new(),
        }
    }
}

impl StudioState {
    pub(crate) fn editing(mut self, line: &str, stage: usize) -> Self {
        self.edit = Some(line.into());
        self.stage = stage;
        self
    }
}

fn saved_note() -> String {
    tr3("Salvo e aplicado na hora", "Saved and applied right away", "Guardado y aplicado al instante").into()
}

impl App {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn studio_e2e_open(&mut self) {
        thread_local!(static OPENED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
        if matches!(self.screen, Screen::Menu { .. }) && !OPENED.replace(true) {
            self.studio = None;
            self.screen = Screen::StoryStudio { cur: 0 };
        }
    }

    pub(super) fn story_studio_screen(&mut self) {
        let Screen::StoryStudio { mut cur } = self.screen else { return };
        let mut st = self.studio.take().unwrap_or_else(StudioState::open);
        cur = cur.min(ROWS - 1);
        let lang = crate::i18n::get();
        if st.edit.is_some() {
            self.studio_type(&mut st, cur, lang);
        } else {
            cur = self.studio_navigate(&mut st, cur, lang);
        }
        self.draw_story_studio(&st, cur, lang);
        if matches!(self.screen, Screen::StoryStudio { .. }) {
            self.screen = Screen::StoryStudio { cur };
            self.studio = Some(st);
        }
    }

    fn studio_commit(&mut self, st: &mut StudioState) {
        st.note = match studio::save(&st.script) {
            Ok(()) => saved_note(),
            Err(e) => e,
        };
        self.audio.sfx("select");
    }

    fn studio_type(&mut self, st: &mut StudioState, cur: usize, lang: Lang) {
        let Some(buf) = st.edit.as_mut() else { return };
        for ch in typed_text().chars().filter(|c| !c.is_control()) {
            if buf.chars().count() < if cur == TITLE_ROW { studio::TITLE_MAX_CHARS } else { studio::BEAT_MAX_CHARS } {
                buf.push(ch);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            buf.pop();
        }
        if self.c.pressed(nav::BACK) {
            st.edit = None;
        } else if is_key_pressed(KeyCode::Enter) {
            let text = st.edit.take().unwrap_or_default();
            if cur == TITLE_ROW {
                studio::set_title(&mut st.script, st.stage, lang, &text);
            } else {
                studio::set_beat(&mut st.script, st.stage, cur - BEAT_ROW, lang, &text);
            }
            self.studio_commit(st);
        }
    }

    fn studio_navigate(&mut self, st: &mut StudioState, mut cur: usize, lang: Lang) -> usize {
        if self.c.pressed(nav::DOWN) {
            cur = (cur + 1) % ROWS;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::UP) {
            cur = (cur + ROWS - 1) % ROWS;
            self.audio.sfx("select");
        }
        let dir = (self.c.pressed(nav::RIGHT) as i32) - (self.c.pressed(nav::LEFT) as i32);
        let ok = self.c.pressed(nav::OK | nav::START);
        if self.c.pressed(nav::BACK) {
            self.screen = Screen::Settings { tab: 3, cur: 0, from_fight: false };
            return cur;
        }
        let stages = st.script.stages.len().max(1);
        match cur {
            c if c < RULE_ROWS && dir != 0 => {
                RULES[c].step(&mut st.script.campaign, dir);
                self.studio_commit(st);
            }
            STAGE_ROW if dir != 0 => {
                st.stage = (st.stage as i32 + dir).rem_euclid(stages as i32) as usize;
                self.audio.sfx("select");
            }
            TITLE_ROW if ok => {
                st.edit = Some(st.script.stage_title(st.stage, lang).to_string());
                self.audio.sfx("confirm");
            }
            BOSS_ROW if dir != 0 => {
                let roster: Vec<String> = self
                    .reg
                    .chars
                    .iter()
                    .filter(|c| !c.broken.get() && !c.id.starts_with("npc_"))
                    .map(|c| c.id.strip_prefix("umf:").unwrap_or(&c.id).to_string())
                    .collect();
                studio::cycle_boss(&mut st.script, st.stage, &roster, dir);
                self.studio_commit(st);
            }
            FOES_ROW if dir != 0 => {
                studio::adjust_foes(&mut st.script, st.stage, dir);
                self.studio_commit(st);
            }
            c if (BEAT_ROW..RESET_ROW).contains(&c) && ok => {
                st.edit = Some(st.script.stage_beat(st.stage, c - BEAT_ROW, lang).to_string());
                self.audio.sfx("confirm");
            }
            RESET_ROW if ok => {
                st.script = studio::reset();
                st.note = tr3("Roteiro original restaurado", "Original script restored", "Guion original restaurado").into();
                self.audio.sfx("cancel");
            }
            EPILOGUE_ROW if ok => {
                self.epi_studio = None;
                self.screen = Screen::EpilogueStudio { cur: 0 };
            }
            _ => {}
        }
        cur
    }

    fn draw_story_studio(&mut self, st: &StudioState, cur: usize, lang: Lang) {
        crate::screen_art::ScreenArt::Lab.background(&self.ui);
        let title = st.script.title(lang).to_string();
        self.ui.adapted_header(tr3("EDITOR DA HISTÓRIA", "STORY STUDIO", "EDITOR DE HISTORIA"), &title, VW);
        for i in 0..ROWS {
            let y = ROW_TOP + i as f32 * ROW_H;
            let focused = i == cur;
            self.ui.adapted_row(48.0, y - 28.0, VW - 96.0, ROW_H - 4.0, true);
            if focused {
                self.ui.adapted_highlight(("studio.x", "studio.y"), 48.0, y - 28.0, VW - 96.0, ROW_H - 4.0);
            }
            let (label, value) = self.studio_row(st, i, cur, lang);
            let ink = if focused { WHITE } else { hex("#d8d0f0") };
            self.ui.text(&label, 72.0, y, 22.0, ink, Align::Left, focused);
            let value = self.fit_label(&value, 22.0, VALUE_W);
            self.ui.text(&value, VW - 72.0, y, 22.0, if focused { WHITE } else { hex("#22e8ff") }, Align::Right, focused);
        }
        let footer = if st.edit.is_some() {
            tr3("Digite · Enter: salvar · Esc: cancelar", "Type · Enter: save · Esc: cancel", "Escribe · Enter: guardar · Esc: cancelar")
        } else {
            tr3(
                "Cima/Baixo: navegar · Esq/Dir: ajustar · Enter: editar fala · Esc: voltar",
                "Up/Down: navigate · Left/Right: adjust · Enter: edit line · Esc: back",
                "Arriba/Abajo: navegar · Izq/Der: ajustar · Enter: editar línea · Esc: volver",
            )
        };
        self.ui.text(&st.note, VW / 2.0, VH - 46.0, 18.0, hex("#7dffb0"), Align::Center, false);
        self.ui.text(footer, VW / 2.0, VH - 18.0, 18.0, hex("#a49cc0"), Align::Center, false);
    }

    fn studio_row(&self, st: &StudioState, i: usize, cur: usize, lang: Lang) -> (String, String) {
        match i {
            c if c < RULE_ROWS => {
                let (pt, en, es) = RULES[c].label();
                (tr3(pt, en, es).into(), format!("‹ {} ›", RULES[c].display(&st.script.campaign)))
            }
            STAGE_ROW => {
                let name = st.script.stage_title(st.stage, lang);
                (tr3("Fase", "Stage", "Fase").into(), format!("‹ {}/{} · {} ›", st.stage + 1, st.script.stages.len().max(1), name))
            }
            TITLE_ROW => {
                let text = match &st.edit {
                    Some(buf) if i == cur => format!("{buf}_"),
                    _ => st.script.stage_title(st.stage, lang).to_string(),
                };
                (tr3("Nome da fase", "Stage name", "Nombre de la fase").into(), text)
            }
            BOSS_ROW => {
                let boss = st.script.stage(st.stage).map(|s| s.boss.as_str()).unwrap_or("-");
                (tr3("Chefe da fase", "Stage boss", "Jefe de la fase").into(), format!("‹ {boss} ›"))
            }
            FOES_ROW => {
                let (lo, hi) = studio::foe_range(&st.script, st.stage);
                let shown = if lo == hi { lo.to_string() } else { format!("{lo}-{hi}") };
                (tr3("Inimigos por onda", "Foes per wave", "Enemigos por oleada").into(), format!("‹ {shown} ›"))
            }
            EPILOGUE_ROW => (tr3("Epílogos…", "Epilogues…", "Epílogos…").into(), "›".into()),
            RESET_ROW => (tr3("Restaurar roteiro original", "Restore original script", "Restaurar guion original").into(), String::new()),
            c => {
                let n = c - BEAT_ROW;
                let text = match &st.edit {
                    Some(buf) if c == cur => format!("{buf}_"),
                    _ => st.script.stage_beat(st.stage, n, lang).to_string(),
                };
                (format!("{} {}", tr3("Fala", "Line", "Línea"), n + 1), text)
            }
        }
    }
}
