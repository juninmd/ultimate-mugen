use super::*;
use crate::i18n::tr3;
use std::cell::{Cell, RefCell};

mod keyboard;
mod page_draw;
mod pill;

use keyboard::Osk;

const MAX_QUERY: usize = 24;
const ROWS: usize = 13;
const ROWS_WITH_KEYBOARD: usize = 6;
const ROW_Y: f32 = 150.0;
const ROW_STEP: f32 = 38.0;
const PILL: (f32, f32, f32, f32) = (40.0, 63.0, 236.0, 20.0);

thread_local! {
    static QUERY: RefCell<String> = const { RefCell::new(String::new()) };
    static OPEN: Cell<bool> = const { Cell::new(false) };
    static PICK: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn query() -> String {
    QUERY.with(|q| q.borrow().to_lowercase())
}

fn clear_query() {
    QUERY.with(|q| q.borrow_mut().clear());
}

impl App {
    pub(crate) fn set_search_query(&mut self, text: &str, open: bool) {
        QUERY.with(|q| *q.borrow_mut() = text.to_string());
        OPEN.with(|o| o.set(open));
        PICK.with(|p| p.set(0));
    }

    pub(crate) fn search_matches(&self, i: usize) -> bool {
        let q = query();
        if q.is_empty() {
            return true;
        }
        let e = &self.reg.chars[i];
        [&e.name, &e.id, &e.author, &e.series]
            .iter()
            .any(|field| field.to_lowercase().contains(&q))
    }

    pub(crate) fn search_label(&self) -> String {
        let q = QUERY.with(|q| q.borrow().clone());
        if q.is_empty() {
            tr3("buscar", "search", "buscar").to_string()
        } else {
            format!("\"{q}\"")
        }
    }

    fn search_hits(&self) -> Vec<usize> {
        let base = self.playable();
        if query().is_empty() {
            return base;
        }
        base.into_iter()
            .filter(|&i| self.search_matches(i))
            .collect()
    }

    fn open_search(&mut self) {
        OPEN.with(|o| o.set(true));
        PICK.with(|p| p.set(0));
        self.audio.sfx("select");
    }

    pub(crate) fn search_page(&mut self) -> bool {
        if !OPEN.with(|o| o.get()) {
            if is_key_pressed(KeyCode::Slash) {
                self.open_search();
                return true;
            }
            return false;
        }
        let before = QUERY.with(|q| q.borrow().clone());
        let osk = self.osk_press();
        QUERY.with(|q| {
            let mut q = q.borrow_mut();
            let typed: String = match osk {
                Some(Osk::Char(c)) => c.to_string(),
                _ => typed_text(),
            };
            for c in typed.chars().filter(|c| !c.is_control()) {
                if q.chars().count() < MAX_QUERY {
                    q.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) || osk == Some(Osk::Back) {
                q.pop();
            }
        });
        if QUERY.with(|q| *q.borrow() != before) {
            PICK.with(|p| p.set(0));
        }
        let hits = self.search_hits();
        let rows = self.search_rows();
        let step = |p: usize, d: isize| {
            (p as isize + d).clamp(0, hits.len().saturating_sub(1) as isize) as usize
        };
        for (key, d) in [
            (KeyCode::Down, 1),
            (KeyCode::Up, -1),
            (KeyCode::PageDown, rows as isize),
            (KeyCode::PageUp, -(rows as isize)),
        ] {
            if is_key_pressed(key) {
                PICK.with(|p| p.set(step(p.get(), d)));
            }
        }
        let tapped = self.search_row_tap(&hits, rows);
        if let Some(k) = tapped {
            PICK.with(|p| p.set(k));
        }
        if is_key_pressed(KeyCode::Escape) || osk == Some(Osk::Close) {
            clear_query();
            OPEN.with(|o| o.set(false));
            self.sel_row = 0;
        } else if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || osk == Some(Osk::Ok)
            || tapped.is_some()
        {
            self.apply_search_pick(&hits);
            return true;
        }
        self.draw_search_page(&hits, rows);
        if self.osk_shown() {
            self.draw_osk();
        }
        true
    }

    fn apply_search_pick(&mut self, hits: &[usize]) {
        OPEN.with(|o| o.set(false));
        self.audio.sfx("confirm");
        self.sel_row = 0;
        let chosen = hits.get(PICK.with(|p| p.get())).copied();
        let Screen::Select {
            mut cur,
            picking,
            picked,
        } = self.screen
        else {
            return;
        };
        let view = self.char_view();
        if let Some(first) = chosen
            .filter(|i| view.contains(i))
            .or_else(|| view.first().copied())
        {
            cur[picking.min(1)] = first;
            self.screen = Screen::Select {
                cur,
                picking,
                picked,
            };
        }
    }

    fn search_rows(&self) -> usize {
        if self.osk_shown() {
            ROWS_WITH_KEYBOARD
        } else {
            ROWS
        }
    }

    fn search_row_tap(&self, hits: &[usize], rows: usize) -> Option<usize> {
        let pick = PICK.with(|p| p.get()).min(hits.len().saturating_sub(1));
        let top = pick.saturating_sub(rows - 1);
        (0..rows.min(hits.len().saturating_sub(top)))
            .find(|&row| self.clicked_rect(40.0, ROW_Y + row as f32 * ROW_STEP, 1200.0, 34.0))
            .map(|row| top + row)
    }
}
