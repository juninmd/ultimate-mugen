use super::*;
use crate::i18n::tr3;
use std::cell::RefCell;

const CHIP: (f32, f32, f32, f32) = (282.0, 63.0, 118.0, 20.0);

thread_local! {
    static AUTHOR: RefCell<Option<String>> = const { RefCell::new(None) };
    static AUTHORS: RefCell<(usize, Vec<(String, usize)>)> = const { RefCell::new((usize::MAX, Vec::new())) };
}

pub(crate) fn author_filter() -> Option<String> {
    AUTHOR.with(|a| a.borrow().clone())
}

impl App {
    pub(crate) fn author_matches(&self, i: usize) -> bool {
        AUTHOR.with(|a| {
            a.borrow()
                .as_ref()
                .is_none_or(|name| self.reg.chars[i].author == *name)
        })
    }

    fn authors(&self) -> Vec<(String, usize)> {
        AUTHORS.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.0 != self.reg.chars.len() {
                let mut counts: Vec<(String, usize)> = Vec::new();
                for i in self.playable() {
                    let name = self.reg.chars[i].author.trim();
                    if name.is_empty() {
                        continue;
                    }
                    match counts.iter_mut().find(|(n, _)| n == name) {
                        Some(entry) => entry.1 += 1,
                        None => counts.push((name.to_string(), 1)),
                    }
                }
                counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                *cache = (self.reg.chars.len(), counts);
            }
            cache.1.clone()
        })
    }

    pub(crate) fn cycle_author(&mut self, forward: bool) {
        let authors = self.authors();
        let now = author_filter();
        let at = now
            .as_ref()
            .and_then(|n| authors.iter().position(|(a, _)| a == n));
        let len = authors.len() + 1;
        let slot = at.map_or(0, |k| k + 1);
        let next = if forward {
            (slot + 1) % len
        } else {
            (slot + len - 1) % len
        };
        AUTHOR.with(|a| *a.borrow_mut() = next.checked_sub(1).map(|k| authors[k].0.clone()));
        self.sel_row = 0;
        self.audio.sfx("select");
    }

    pub(crate) fn set_author_filter(&mut self, name: Option<&str>) {
        AUTHOR.with(|a| *a.borrow_mut() = name.map(String::from));
    }

    pub(crate) fn author_chip(&mut self) -> bool {
        let (x, y, w, h) = CHIP;
        let active = author_filter();
        let hover = self.hover_rect(x, y, w, h);
        let fill = if active.is_some() {
            Color::new(1.0, 0.8, 0.2, 0.85)
        } else if hover {
            Color::new(1.0, 1.0, 1.0, 0.16)
        } else {
            Color::new(1.0, 1.0, 1.0, 0.07)
        };
        self.ui.rect(x, y, w, h, fill);
        let text = match &active {
            Some(name) => {
                let count = self
                    .authors()
                    .iter()
                    .find(|(a, _)| a == name)
                    .map_or(0, |(_, c)| *c);
                format!("{name} ({count})")
            }
            None => format!("[U] {}", tr3("Autor: todos", "Author: all", "Autor: todos")),
        };
        let ink = if active.is_some() {
            BLACK
        } else {
            Color::new(1.0, 1.0, 1.0, 0.6)
        };
        let label = self.fit_label(&text, 13.0, w - 12.0);
        self.ui
            .text(&label, x + 6.0, y + 15.0, 13.0, ink, Align::Left, false);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        if is_key_pressed(KeyCode::U) {
            self.cycle_author(!shift);
            return true;
        }
        if self.clicked_rect(x, y, w, h) {
            self.cycle_author(true);
            return true;
        }
        false
    }
}
