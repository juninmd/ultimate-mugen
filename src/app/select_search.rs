use super::*;
use crate::i18n::tr3;
use std::cell::{Cell, RefCell};

const MAX_QUERY: usize = 24;
const ROWS: usize = 12;

thread_local! {
    static QUERY: RefCell<String> = const { RefCell::new(String::new()) };
    static OPEN: Cell<bool> = const { Cell::new(false) };
}

fn query() -> String {
    QUERY.with(|q| q.borrow().to_lowercase())
}

impl App {
    pub(crate) fn search_matches(&self, i: usize) -> bool {
        let q = query();
        if q.is_empty() {
            return true;
        }
        let e = &self.reg.chars[i];
        e.name.to_lowercase().contains(&q) || e.id.to_lowercase().contains(&q)
    }

    pub(crate) fn search_label(&self) -> String {
        let q = QUERY.with(|q| q.borrow().clone());
        if q.is_empty() {
            tr3("buscar", "search", "buscar").to_string()
        } else {
            format!("\"{q}\"")
        }
    }

    pub(crate) fn search_page(&mut self) -> bool {
        if !OPEN.with(|o| o.get()) {
            if is_key_pressed(KeyCode::Slash) {
                OPEN.with(|o| o.set(true));
                self.audio.sfx("select");
                return true;
            }
            return false;
        }
        QUERY.with(|q| {
            let mut q = q.borrow_mut();
            for c in typed_text().chars().filter(|c| !c.is_control()) {
                if q.chars().count() < MAX_QUERY {
                    q.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                q.pop();
            }
        });
        if is_key_pressed(KeyCode::Escape) {
            QUERY.with(|q| q.borrow_mut().clear());
            OPEN.with(|o| o.set(false));
        } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            OPEN.with(|o| o.set(false));
            self.audio.sfx("confirm");
            self.sel_row = 0;
            if let Screen::Select { cur, picking, picked } = self.screen {
                if let Some(&first) = self.char_view().first() {
                    let mut cur = cur;
                    cur[picking.min(1)] = first;
                    self.screen = Screen::Select { cur, picking, picked };
                }
            }
            return true;
        }
        self.draw_search_page();
        true
    }

    fn draw_search_page(&self) {
        self.ui.background();
        self.ui.text(tr3("BUSCAR LUTADOR", "SEARCH FIGHTER", "BUSCAR LUCHADOR"), 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        self.ui.rect(40.0, 84.0, 700.0, 50.0, Color::new(1.0, 1.0, 1.0, 0.1));
        self.ui.rect_lines(40.0, 84.0, 700.0, 50.0, 2.0, hex("#3fa9ff"));
        let blink = if (self.ui.t * 2.0) as i32 % 2 == 0 { "_" } else { " " };
        let shown = format!("{}{}", QUERY.with(|q| q.borrow().clone()), blink);
        self.ui.text(&shown, 56.0, 119.0, 26.0, WHITE, Align::Left, true);
        let q = query();
        let hits: Vec<usize> = self.playable().into_iter().filter(|&i| self.search_matches(i)).collect();
        let count = format!("{} / {}", hits.len(), self.playable().len());
        self.ui.text(&count, 740.0, 70.0, 16.0, hex("#a49cc0"), Align::Right, false);
        for (row, &i) in hits.iter().take(ROWS).enumerate() {
            let y = 160.0 + row as f32 * 36.0;
            self.ui.rect(40.0, y, 700.0, 32.0, Color::new(1.0, 1.0, 1.0, if row == 0 { 0.16 } else { 0.06 }));
            let c = rgb(self.reg.chars[i].color);
            self.ui.rect(40.0, y, 6.0, 32.0, c);
            self.ui.text(&self.reg.chars[i].name, 62.0, y + 23.0, 20.0, WHITE, Align::Left, true);
            let s = self.char_stats(i);
            self.ui.text(&format!("{:.0} / {:.0} / {:.1}", s.force, s.life, s.speed), 724.0, y + 22.0, 14.0, hex("#a49cc0"), Align::Right, false);
        }
        if hits.is_empty() && !q.is_empty() {
            self.ui.text(tr3("Nenhum lutador encontrado", "No fighters found", "Ningún luchador encontrado"), 40.0, 190.0, 20.0, hex("#ff4d6d"), Align::Left, true);
        }
        let tip = tr3("Digite para filtrar · Enter aplica · Esc limpa", "Type to filter · Enter applies · Esc clears", "Escribe para filtrar · Enter aplica · Esc limpia");
        self.ui.text(tip, 40.0, 700.0, 15.0, hex("#a79fc4"), Align::Left, false);
    }
}
