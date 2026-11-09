use super::layout::{Layout, Rect};
use super::skin::{fade, glass, mix, pill};
use super::state::Tab;
use crate::app::*;
use crate::ui::{hex, Face};

pub(super) const INK: &str = "#f2eeff";
pub(super) const MUTED: &str = "#a49cc0";
pub(super) const CYAN: &str = "#22e8ff";
pub(super) const GOLD: &str = "#ffcc33";
pub(super) const PINK: &str = "#ff4d6d";

pub(super) fn tab_label(tab: Tab) -> &'static str {
    match tab {
        Tab::Game => crate::i18n::tr("JOGO", "GAME"),
        Tab::Moves => crate::i18n::t("pause.moves"),
        Tab::System => crate::i18n::tr("SISTEMA", "SYSTEM"),
    }
}

impl App {
    pub(super) fn pause_chrome(&self, lay: &Layout, tab: Tab, fighter: &str, author: &str, a: f32) {
        let ui = &self.ui;
        glass(ui, lay.panel, 22.0, [0.25, 0.32, 0.7], a);
        ui.rect(lay.panel.x, lay.panel.y, lay.panel.w, lay.panel.h, Color::new(0.03, 0.04, 0.1, 0.5 * a));
        ui.rect(lay.panel.x + 24.0, lay.panel.y + 1.0, lay.panel.w - 48.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.22 * a));
        let f = lay.font;
        let w = ui.text(crate::i18n::t("pause.title"), lay.title.0, lay.title.1, 36.0 * f, fade(hex(GOLD), a), Align::Left, true);
        let nw = ui.text(
            &fighter.to_uppercase(),
            lay.title.0 + w + 18.0,
            lay.title.1 - 2.0,
            18.0 * f,
            fade(hex(MUTED), a),
            Align::Left,
            true,
        );
        let room = lay.panel.right() - (lay.title.0 + w + 18.0 + nw + 14.0) - 24.0;
        let max = ((room / (11.0 * f)).max(0.0) as usize).min(28);
        if let Some(by) = crate::credit_line::by_author(author, max).filter(|_| max >= 8) {
            ui.text(&by, lay.title.0 + w + 18.0 + nw + 14.0, lay.title.1 - 2.0, 15.0 * f, fade(hex(CYAN), 0.8 * a), Align::Left, false);
        }
        for (i, t) in Tab::ALL.iter().enumerate() {
            let r = lay.tabs[i];
            let on = *t == tab;
            let hover = self.hover_rect(r.x, r.y, r.w, r.h);
            let (top, bot, edge) = if on {
                (Color::new(1.0, 0.3, 0.43, 0.9), Color::new(0.65, 0.12, 0.3, 0.9), Some(fade(hex("#ffd0da"), 0.7 * a)))
            } else if hover {
                (Color::new(1.0, 1.0, 1.0, 0.16), Color::new(1.0, 1.0, 1.0, 0.08), None)
            } else {
                (Color::new(1.0, 1.0, 1.0, 0.07), Color::new(1.0, 1.0, 1.0, 0.03), None)
            };
            pill(ui, r, 8.0, fade(top, a), fade(bot, a), edge);
            let c = mix(hex(MUTED), hex(INK), if on { 1.0 } else { 0.0 });
            ui.text(tab_label(*t), r.x + r.w / 2.0, r.y + r.h / 2.0 + 7.0 * f, 18.0 * f, fade(c, a), Align::Center, true);
        }
    }

    pub(super) fn pause_footer(&self, r: Rect, lay: &Layout, tab: Tab, a: f32) {
        let ui = &self.ui;
        let muted = fade(hex(MUTED), a);
        let size = 22.0 * lay.font;
        let base = r.y + r.h - 8.0;
        if let Some((msg, t)) = &self.pause.toast {
            ui.text(msg, r.x, base - 4.0, 20.0 * lay.font, fade(hex(CYAN), (t * 3.0).min(1.0) * a), Align::Left, true);
            return;
        }
        if lay.compact {
            let text = match tab {
                Tab::Moves => crate::i18n::tr("Toque num golpe e depois num slot para atribuir", "Tap a move, then a slot to bind it"),
                _ => crate::i18n::tr("Toque numa opcao", "Tap an option"),
            };
            ui.text(text, r.x, base - 4.0, 19.0 * lay.font, muted, Align::Left, false);
            return;
        }
        let mut x = r.x;
        let hints: Vec<(Vec<&'static str>, &str)> = match tab {
            Tab::Moves => vec![
                (vec!["^", "v"], crate::i18n::tr("escolher", "select")),
                (vec!["1", "2", "3", "4"], crate::i18n::tr("atribuir slot", "bind slot")),
                (vec!["<", ">"], crate::i18n::tr("aba", "tab")),
                (vec!["Esc"], crate::i18n::tr("voltar ao jogo", "resume")),
            ],
            _ => vec![
                (vec!["^", "v"], crate::i18n::tr("escolher", "select")),
                (vec!["Enter"], crate::i18n::tr("confirmar", "confirm")),
                (vec!["<", ">"], crate::i18n::tr("aba", "tab")),
                (vec!["Esc"], crate::i18n::tr("voltar ao jogo", "resume")),
            ],
        };
        for (keys, label) in hints {
            for k in keys {
                x += self.hint_key(k, x, base, size) + 4.0;
            }
            x += ui.text(label, x + 4.0, base - 5.0, 18.0, muted, Align::Left, false) + 30.0;
        }
        if tab == Tab::Moves {
            let pad = super::labels::slot_pad_labels(self.c.glyphs).join(" ");
            ui.text(&pad, r.right(), base - 5.0, 16.0, muted, Align::Right, true);
        }
    }

    fn hint_key(&self, k: &'static str, x: f32, base: f32, size: f32) -> f32 {
        let dir = match k {
            "^" => Some((0.0, -1.0)),
            "v" => Some((0.0, 1.0)),
            "<" => Some((-1.0, 0.0)),
            ">" => Some((1.0, 0.0)),
            _ => None,
        };
        match dir {
            Some(d) => {
                self.ui.rect(x, base - size, size, size, Color::new(1.0, 1.0, 1.0, 0.16));
                self.ui.arrow(x + size / 2.0, base - size / 2.0, size * 0.8, d, hex(INK));
                size
            }
            None => self.ui.face(Face::key(k), x, base, size),
        }
    }
}
