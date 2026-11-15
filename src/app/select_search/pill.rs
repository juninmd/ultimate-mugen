use super::*;
use crate::i18n::tr3;

impl App {
    pub(crate) fn search_pill(&mut self) -> bool {
        let (x, y, w, h) = PILL;
        let q = QUERY.with(|q| q.borrow().clone());
        let hover = self.hover_rect(x, y, w, h);
        let active = !q.is_empty();
        let fill = if active {
            Color::new(0.25, 0.66, 1.0, 0.28)
        } else if hover {
            Color::new(1.0, 1.0, 1.0, 0.16)
        } else {
            Color::new(1.0, 1.0, 1.0, 0.07)
        };
        self.ui.rect(x, y, w, h, fill);
        self.ui.rect(
            x,
            y,
            3.0,
            h,
            if active {
                hex("#3fa9ff")
            } else {
                Color::new(1.0, 1.0, 1.0, 0.3)
            },
        );
        let label = if active {
            let count = self.char_view().len();
            format!(
                "\"{q}\" · {count} {} · [Esc]",
                tr3("resultados", "results", "resultados")
            )
        } else {
            format!(
                "[/] {}",
                tr3(
                    "Buscar nome, autor ou série…",
                    "Search name, author or series…",
                    "Buscar nombre, autor o serie…"
                )
            )
        };
        let ink = if active {
            WHITE
        } else {
            Color::new(1.0, 1.0, 1.0, 0.6)
        };
        self.ui
            .text(&label, x + 10.0, y + 15.0, 13.0, ink, Align::Left, false);
        if active && is_key_pressed(KeyCode::Escape) {
            clear_query();
            self.sel_row = 0;
            self.audio.sfx("select");
            return true;
        }
        if self.clicked_rect(x, y, w, h) {
            if active {
                clear_query();
                self.sel_row = 0;
            } else {
                self.open_search();
            }
            return true;
        }
        false
    }
}
