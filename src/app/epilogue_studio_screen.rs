use super::*;
use crate::controls::nav;
use crate::i18n::{tr3, Lang};
use crate::story::epilogue::Epilogue;
use crate::story::epilogue_edit::{self as edit, Field};
use crate::story::epilogue_store;

const ROW_TOP: f32 = 142.0;
const ROW_H: f32 = 40.0;
const VALUE_W: f32 = 760.0;
const SHOWN_TAIL: usize = 64;

pub(crate) struct EpiState {
    fighter: usize,
    ids: Vec<String>,
    edit: Option<String>,
    note: String,
    cache: Epilogue,
    edited: bool,
}

impl EpiState {
    pub(crate) fn open() -> Self {
        let mut state = Self { fighter: 0, ids: epilogue_store::ids(), edit: None, note: String::new(), cache: Epilogue::default(), edited: false };
        state.refresh();
        state
    }

    fn refresh(&mut self) {
        self.cache = epilogue_store::table().get(self.id()).cloned().unwrap_or_default();
        self.edited = epilogue_store::has_override(self.id());
    }

    pub(crate) fn editing(mut self, text: &str, id: &str) -> Self {
        self.fighter = self.ids.iter().position(|k| k == id).unwrap_or(0);
        self.edit = Some(text.into());
        self.refresh();
        self
    }

    fn id(&self) -> &str {
        self.ids.get(self.fighter).map(String::as_str).unwrap_or("*")
    }
}

fn field_label(field: Field) -> String {
    match field {
        Field::Title => tr3("Título", "Title", "Título").into(),
        Field::Boss => tr3("Fala do chefe", "Boss line", "Frase del jefe").into(),
        Field::Reply => tr3("Resposta do herói", "Hero reply", "Respuesta del héroe").into(),
        Field::Page(n) => format!("{} {}", tr3("Página", "Page", "Página"), n + 1),
    }
}

impl App {
    pub(super) fn epilogue_studio_screen(&mut self) {
        let Screen::EpilogueStudio { mut cur } = self.screen else { return };
        let mut st = self.epi_studio.take().unwrap_or_else(EpiState::open);
        let lang = crate::i18n::get();
        let fields = edit::fields(&st.cache);
        let last = fields.len() + 1;
        cur = cur.min(last);
        if st.edit.is_some() {
            self.epilogue_type(&mut st, &fields, cur, lang);
        } else {
            cur = self.epilogue_navigate(&mut st, &fields, cur);
        }
        self.draw_epilogue_studio(&st, &fields, cur, lang);
        if matches!(self.screen, Screen::EpilogueStudio { .. }) {
            self.screen = Screen::EpilogueStudio { cur };
            self.epi_studio = Some(st);
        }
    }

    fn epilogue_type(&mut self, st: &mut EpiState, fields: &[Field], cur: usize, lang: Lang) {
        let Some(buf) = st.edit.as_mut() else { return };
        for ch in typed_text().chars().filter(|c| !c.is_control()) {
            if buf.chars().count() < edit::TEXT_MAX_CHARS {
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
            if let Some(field) = cur.checked_sub(1).and_then(|i| fields.get(i)) {
                st.note = match edit::commit(st.id(), *field, lang.code(), &text) {
                    Ok(()) => tr3("Salvo e aplicado na hora", "Saved and applied right away", "Guardado y aplicado al instante").into(),
                    Err(e) => e,
                };
                st.refresh();
                self.audio.sfx("select");
            }
        }
    }

    fn epilogue_navigate(&mut self, st: &mut EpiState, fields: &[Field], mut cur: usize) -> usize {
        let rows = fields.len() + 2;
        if self.c.pressed(nav::DOWN) {
            cur = (cur + 1) % rows;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::UP) {
            cur = (cur + rows - 1) % rows;
            self.audio.sfx("select");
        }
        let dir = (self.c.pressed(nav::RIGHT) as i32) - (self.c.pressed(nav::LEFT) as i32);
        let ok = self.c.pressed(nav::OK | nav::START);
        if self.c.pressed(nav::BACK) {
            self.screen = Screen::StoryStudio { cur: 0 };
            return cur;
        }
        if cur == 0 && dir != 0 {
            let n = st.ids.len().max(1) as i32;
            st.fighter = (st.fighter as i32 + dir).rem_euclid(n) as usize;
            st.refresh();
            self.audio.sfx("select");
            return 0;
        }
        if cur == rows - 1 && ok {
            st.note = match epilogue_store::clear_override(st.id()) {
                Ok(()) => tr3("Epílogo original restaurado", "Original epilogue restored", "Epílogo original restaurado").into(),
                Err(e) => e,
            };
            st.refresh();
            self.audio.sfx("cancel");
        } else if ok && cur >= 1 {
            let lang = crate::i18n::get();
            if let Some(field) = fields.get(cur - 1) {
                st.edit = Some(edit::read(&st.cache, *field, lang.code()));
                self.audio.sfx("confirm");
            }
        }
        cur
    }

    fn draw_epilogue_studio(&mut self, st: &EpiState, fields: &[Field], cur: usize, lang: Lang) {
        crate::screen_art::ScreenArt::Lab.background(&self.ui);
        let mark = if st.edited { "●" } else { "" };
        self.ui.adapted_header(tr3("EDITOR DE EPÍLOGOS", "EPILOGUE STUDIO", "EDITOR DE EPÍLOGOS"), &format!("{} {mark}", st.id()), VW);
        let rows = fields.len() + 2;
        let epilogue = &st.cache;
        for i in 0..rows {
            let y = ROW_TOP + i as f32 * ROW_H;
            let focused = i == cur;
            self.ui.adapted_row(48.0, y - 28.0, VW - 96.0, ROW_H - 4.0, true);
            if focused {
                self.ui.adapted_highlight(("epi.x", "epi.y"), 48.0, y - 28.0, VW - 96.0, ROW_H - 4.0);
            }
            let (label, value) = if i == 0 {
                (tr3("Lutador", "Fighter", "Luchador").to_string(), format!("‹ {} {}/{} ›", st.id(), st.fighter + 1, st.ids.len()))
            } else if i == rows - 1 {
                (tr3("Restaurar epílogo original", "Restore original epilogue", "Restaurar epílogo original").to_string(), String::new())
            } else {
                let field = fields[i - 1];
                let text = match &st.edit {
                    Some(buf) if focused => {
                        let skip = buf.chars().count().saturating_sub(SHOWN_TAIL);
                        format!("{}{}_", if skip > 0 { "…" } else { "" }, buf.chars().skip(skip).collect::<String>())
                    }
                    _ => edit::read(epilogue, field, lang.code()),
                };
                (field_label(field), text)
            };
            let ink = if focused { WHITE } else { hex("#d8d0f0") };
            self.ui.text(&label, 72.0, y, 22.0, ink, Align::Left, focused);
            let value = self.fit_label(&value, 22.0, VALUE_W);
            self.ui.text(&value, VW - 72.0, y, 22.0, if focused { WHITE } else { hex("#22e8ff") }, Align::Right, focused);
        }
        let footer = if st.edit.is_some() {
            tr3("Digite · Enter: salvar · Esc: cancelar", "Type · Enter: save · Esc: cancel", "Escribe · Enter: guardar · Esc: cancelar")
        } else {
            tr3("Esq/Dir: trocar lutador · Enter: editar · Esc: voltar", "Left/Right: switch fighter · Enter: edit · Esc: back", "Izq/Der: cambiar luchador · Enter: editar · Esc: volver")
        };
        self.ui.text(&st.note, VW / 2.0, VH - 46.0, 18.0, hex("#7dffb0"), Align::Center, false);
        self.ui.text(footer, VW / 2.0, VH - 18.0, 18.0, hex("#a49cc0"), Align::Center, false);
    }
}
