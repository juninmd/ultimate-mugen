use super::*;
use crate::i18n::tr3;
use std::cell::{Cell, RefCell};

const FILTERS: usize = 3;
const MAX_QUERY: usize = 24;
const CHIP_X: f32 = 470.0;
const CHIP_W: f32 = 98.0;
const HEADER_Y: f32 = 34.0;
const BOX: (f32, f32, f32, f32) = (790.0, 34.0, 450.0, 26.0);

type ViewKey = (usize, String, usize);

thread_local! {
    static KIND: Cell<usize> = const { Cell::new(0) };
    static QUERY: RefCell<String> = const { RefCell::new(String::new()) };
    static TYPING: Cell<bool> = const { Cell::new(false) };
    static VIEW: RefCell<Option<(ViewKey, Vec<usize>)>> = const { RefCell::new(None) };
}

fn query() -> String {
    QUERY.with(|q| q.borrow().to_lowercase())
}

impl App {
    pub(crate) fn set_stage_filter(&mut self, kind: usize, text: &str) {
        KIND.with(|k| k.set(kind % FILTERS));
        QUERY.with(|q| *q.borrow_mut() = text.to_string());
        TYPING.with(|t| t.set(false));
    }

    pub(super) fn stage_view(&self) -> Vec<usize> {
        let key = (KIND.with(|k| k.get()), query(), self.reg.stages.len());
        if let Some(v) = VIEW.with(|c| {
            c.borrow()
                .as_ref()
                .filter(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        }) {
            return v;
        }
        let (kind, q, _) = &key;
        let mut v = vec![0];
        v.extend(
            self.reg
                .stages
                .iter()
                .enumerate()
                .filter(|(_, s)| match kind {
                    1 => !s.mugen,
                    2 => s.mugen,
                    _ => true,
                })
                .filter(|(_, s)| q.is_empty() || s.name.to_lowercase().contains(q.as_str()) || crate::i18n::stage_names::shown(&s.name).to_lowercase().contains(q.as_str()))
                .map(|(i, _)| i + 1),
        );
        VIEW.with(|c| *c.borrow_mut() = Some((key, v.clone())));
        v
    }

    pub(super) fn stage_typing(&mut self) -> bool {
        if !TYPING.with(|t| t.get()) {
            if is_key_pressed(KeyCode::Slash) {
                TYPING.with(|t| t.set(true));
                self.audio.sfx("select");
                return true;
            }
            if is_key_pressed(KeyCode::Tab) {
                KIND.with(|k| k.set((k.get() + 1) % FILTERS));
                self.audio.sfx("select");
            }
            if is_key_pressed(KeyCode::Escape) && !query().is_empty() {
                QUERY.with(|q| q.borrow_mut().clear());
                self.audio.sfx("select");
                return true;
            }
            return false;
        }
        QUERY.with(|q| {
            let mut q = q.borrow_mut();
            for c in typed_text()
                .chars()
                .filter(|c| !c.is_control() && *c != '/')
            {
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
            TYPING.with(|t| t.set(false));
        } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            TYPING.with(|t| t.set(false));
            self.audio.sfx("confirm");
        }
        true
    }

    pub(super) fn stage_header(&mut self) {
        let labels = [
            crate::i18n::t("select.filter_all"),
            crate::i18n::t("select.filter_originals"),
            crate::i18n::t("select.filter_mugen"),
        ];
        let kind = KIND.with(|k| k.get());
        for (n, label) in labels.iter().enumerate() {
            let x = CHIP_X + n as f32 * (CHIP_W + 6.0);
            if self.select_chip(x, HEADER_Y, label, kind == n) && kind != n {
                KIND.with(|k| k.set(n));
                self.audio.sfx("select");
            }
        }
        let (x, y, w, h) = BOX;
        let typing = TYPING.with(|t| t.get());
        let q = QUERY.with(|q| q.borrow().clone());
        self.ui.rect(
            x,
            y,
            w,
            h,
            Color::new(1.0, 1.0, 1.0, if typing { 0.16 } else { 0.07 }),
        );
        self.ui.rect_lines(
            x,
            y,
            w,
            h,
            if typing { 2.0 } else { 1.0 },
            if typing {
                hex("#3fa9ff")
            } else {
                Color::new(1.0, 1.0, 1.0, 0.2)
            },
        );
        let blink = if typing && (self.ui.t * 2.0) as i32 % 2 == 0 {
            "_"
        } else {
            ""
        };
        let (shown, ink) = if q.is_empty() && !typing {
            (
                format!(
                    "[/] {}",
                    tr3("Buscar cenário…", "Search stage…", "Buscar escenario…")
                ),
                Color::new(1.0, 1.0, 1.0, 0.55),
            )
        } else {
            (format!("{q}{blink}"), WHITE)
        };
        self.ui
            .text(&shown, x + 10.0, y + 18.0, 14.0, ink, Align::Left, false);
        if self.clicked_rect(x, y, w, h) {
            TYPING.with(|t| t.set(!typing));
        }
    }
}
