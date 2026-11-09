use super::draw_chrome::{CYAN, GOLD, INK, MUTED};
use super::labels::{slot_pad_labels, truncate_to};
use super::input_fit::fit_input_size;
use super::layout::Layout;
use super::skin::{fade, mix, pill};
use crate::app::*;
use crate::ui::{hex, Face};
use ume_engine::chardef::MoveInfo;

pub(super) struct SlotCard<'a> {
    pub mv: Option<&'a MoveInfo>,
    pub selected: bool,
    pub bindable: bool,
}

const KEY_NAMES: [&str; 4] = ["1", "2", "3", "4"];

impl App {
    pub(super) fn pause_slots(&self, lay: &Layout, cards: &[SlotCard; 4], a: f32) {
        let ui = &self.ui;
        let f = lay.font;
        let pads = slot_pad_labels(self.c.glyphs);
        for (i, card) in cards.iter().enumerate() {
            let r = lay.slots[i];
            let flash = self.pause.flash[i];
            let hover = card.bindable && self.hover_rect(r.x, r.y, r.w, r.h);
            let base = if card.mv.is_some() { 0.1 } else { 0.05 };
            let lift = base + if hover { 0.07 } else { 0.0 } + flash * 0.2;
            let edge = if flash > 0.0 {
                mix(hex(CYAN), hex(GOLD), flash)
            } else if card.selected {
                hex(CYAN)
            } else {
                Color::new(1.0, 1.0, 1.0, 0.16)
            };
            pill(
                ui,
                r,
                8.0,
                Color::new(1.0, 1.0, 1.0, lift + 0.04 * a),
                Color::new(1.0, 1.0, 1.0, lift * 0.5),
                Some(fade(edge, a * if card.selected || flash > 0.0 { 1.0 } else { 0.8 })),
            );
            if card.selected || flash > 0.0 {
                ui.rect(r.x + 10.0, r.bottom() - 3.0, r.w - 20.0, 2.0, fade(edge, a));
            }
            let chip = 30.0 * f;
            let key_x = r.x + 14.0;
            ui.face(Face::key(KEY_NAMES[i]), key_x, r.y + 12.0 + chip, chip);
            ui.text(pads[i], key_x + chip / 2.0, r.bottom() - 10.0, 15.0 * f, fade(hex(MUTED), a), Align::Center, true);
            let tx = key_x + chip + 22.0;
            let tw = r.right() - tx - 12.0;
            match card.mv {
                Some(mv) => {
                    let name = truncate_to(&mv.name, tw, &|s| ui.measure(s, 19.0 * f, false));
                    ui.text(&name, tx, r.y + 27.0 * f, 19.0 * f, fade(hex(INK), a), Align::Left, false);
                    let size = fit_input_size(&mv.input, tw, 17.0 * f, 11.0, &|_, s| s * 1.6);
                    ui.input_icons(&mv.input, tx, r.bottom() - 9.0, size, |b| self.c.face(b));
                }
                None => {
                    let hint = if card.bindable {
                        crate::i18n::tr("Livre - escolha um golpe", "Empty - pick a move")
                    } else {
                        crate::i18n::tr("Livre", "Empty")
                    };
                    ui.text(hint, tx, r.y + r.h / 2.0 + 6.0, 17.0 * f, fade(hex(MUTED), a * 0.8), Align::Left, false);
                }
            }
        }
    }
}
