use super::holo_loading::Card;
use super::*;
use crate::holo::{self, CardState};
use std::cell::RefCell;

thread_local! {
    static DONE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static LAST: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn track(current: Option<String>, busy: bool) {
    LAST.with(|last| {
        let mut last = last.borrow_mut();
        if *last != current {
            if let Some(prev) = last.take() {
                DONE.with(|d| d.borrow_mut().push(prev));
            }
            *last = current;
        }
    });
    if !busy {
        DONE.with(|d| d.borrow_mut().clear());
        LAST.with(|l| *l.borrow_mut() = None);
    }
}

impl App {
    pub(super) fn market_holo_overlay(&mut self, rx: f32, ry: f32, rw: f32) {
        let busy = self.market.busy();
        track(self.market.current_name(), busy);
        if !busy {
            return;
        }
        let queued: Vec<String> = self
            .market
            .queue
            .iter()
            .filter_map(|(repo, id)| {
                self.market.catalog_cached(repo).and_then(|c| c.find(id)).map(|i| i.name.clone())
            })
            .collect();
        let done: Vec<String> = DONE.with(|d| d.borrow().clone());
        let mut cards: Vec<Card> = vec![];
        for name in &done {
            cards.push(self.holo_card_named(name, CardState::Ready, 0));
        }
        if let Some(name) = self.market.current_name() {
            cards.push(self.holo_card_named(&name, CardState::Loading, 0));
        }
        for name in &queued {
            cards.push(self.holo_card_named(name, CardState::Pending, 1));
        }
        if cards.is_empty() {
            return;
        }
        let (ay, ah) = (ry + 40.0, 436.0);
        self.ui.rect(rx, ay, rw, ah, Color::new(0.03, 0.02, 0.07, 0.92));
        let title = crate::i18n::tr3("BAIXANDO LUTADORES", "DOWNLOADING FIGHTERS", "DESCARGANDO LUCHADORES");
        self.ui.text(title, rx + rw / 2.0, ay + 26.0, 18.0, hex("#7fe9ff"), Align::Center, true);
        let count = format!("{} / {}", done.len(), cards.len());
        self.ui.text(&count, rx + rw - 16.0, ay + 26.0, 15.0, hex("#a49cc0"), Align::Right, false);
        let (pad, top) = (20.0, 40.0);
        let (aw, ah2) = (rw - pad * 2.0, ah - top - pad);
        let m = holo::mosaic(cards.len(), aw, ah2, 150.0);
        for (i, c) in cards.iter().enumerate() {
            let (x, y) = holo::origin(&m, i, cards.len(), rx + pad, ay + top, aw, ah2);
            self.holo_paint(c, x, y, m.w, m.h, m.labels);
        }
    }
}
