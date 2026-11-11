use super::select_category::CATEGORY_COUNT;
use super::select_stats::SORT_MODES;
use super::*;

const X0: f32 = 430.0;
const W: f32 = 98.0;
const GAP: f32 = 6.0;
const H: f32 = 26.0;
const FILTER_Y: f32 = 12.0;
const SORT_Y: f32 = 44.0;

impl App {
    fn select_chip(&self, x: f32, y: f32, label: &str, on: bool) -> bool {
        let hover = self.hover_rect(x, y, W, H);
        let fill = if on {
            Color::new(1.0, 0.8, 0.2, 0.9)
        } else if hover {
            Color::new(1.0, 1.0, 1.0, 0.2)
        } else {
            Color::new(1.0, 1.0, 1.0, 0.08)
        };
        self.ui.rect(x, y, W, H, fill);
        let ink = if on { BLACK } else { Color::new(1.0, 1.0, 1.0, 0.75) };
        self.ui.text(label, x + W / 2.0, y + 18.0, 12.0, ink, Align::Center, true);
        self.clicked_rect(x, y, W, H)
    }

    pub(crate) fn select_chips(&mut self) -> bool {
        let Screen::Select { mut cur, picking, picked } = self.screen else {
            return false;
        };
        let filters = [
            crate::i18n::t("select.filter_all"),
            crate::i18n::t("select.filter_originals"),
            crate::i18n::t("select.filter_mugen"),
            crate::i18n::t("select.filter_bosses"),
        ];
        let sorts = [
            crate::i18n::t("select.sort_roster"),
            crate::i18n::t("select.sort_force"),
            crate::i18n::t("select.sort_life"),
            crate::i18n::t("select.sort_speed"),
            crate::i18n::t("select.sort_name"),
        ];
        let mut changed = false;
        for (n, label) in filters.iter().enumerate().take(CATEGORY_COUNT as usize) {
            let x = X0 + n as f32 * (W + GAP);
            if self.select_chip(x, FILTER_Y, label, self.sel_filter as usize == n) && self.sel_filter as usize != n {
                self.sel_filter = n as u8;
                changed = true;
            }
        }
        for (n, label) in sorts.iter().enumerate().take(SORT_MODES) {
            let x = X0 + n as f32 * (W + GAP);
            if self.select_chip(x, SORT_Y, label, self.settings.sort % SORT_MODES == n) && self.settings.sort % SORT_MODES != n {
                self.settings.sort = n;
                self.settings.save();
                changed = true;
            }
        }
        if !changed {
            return false;
        }
        let view = self.char_view();
        if self.sel_filter != 0 && view.len() == self.playable().len() {
            self.sel_filter = 0;
        }
        let p = picking.min(1);
        if !view.contains(&cur[p]) {
            cur[p] = view[0];
        }
        self.sel_row = 0;
        self.audio.sfx("select");
        self.screen = Screen::Select { cur, picking, picked };
        true
    }
}
