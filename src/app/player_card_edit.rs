//! Card editor modal: name, title, flag and what the player is looking for, saved on confirm.
use super::*;
use crate::flag::catalog::codes;
use crate::i18n::tr3;
use crate::player_card::{seeking_label, Card, SEEKING_COUNT};

const FIELDS: u8 = 4;
const NAME_MAX: usize = 16;
const MODAL_X: f32 = 240.0;
const MODAL_Y: f32 = 150.0;
const MODAL_W: f32 = 800.0;
const MODAL_H: f32 = 420.0;
const ROW_Y: f32 = 246.0;
const ROW_STEP: f32 = 64.0;
const BUTTON_Y: f32 = MODAL_Y + MODAL_H - 72.0;

pub(crate) struct CardEdit {
    pub field: u8,
    pub name: String,
    pub title: String,
    pub flag: String,
    pub seeking: u8,
}

impl CardEdit {
    pub(crate) fn open(card: &Card) -> Self {
        Self {
            field: 0,
            name: card.name.clone(),
            title: card.title.clone(),
            flag: card.flag.clone(),
            seeking: card.seeking,
        }
    }

    fn text_mut(&mut self) -> Option<&mut String> {
        match self.field {
            0 => Some(&mut self.name),
            1 => Some(&mut self.title),
            _ => None,
        }
    }

    fn shift_flag(&mut self, step: isize) {
        let list: Vec<&str> = codes().collect();
        if list.is_empty() {
            return;
        }
        let at = list.iter().position(|c| *c == self.flag.as_str()).unwrap_or(0) as isize;
        self.flag = list[(at + step).rem_euclid(list.len() as isize) as usize].to_string();
    }
}

fn clicked_save(app: &App) -> bool {
    app.clicked_rect(MODAL_X + MODAL_W - 330.0, BUTTON_Y, 150.0, 44.0)
}

fn clicked_cancel(app: &App) -> bool {
    app.clicked_rect(MODAL_X + MODAL_W - 170.0, BUTTON_Y, 140.0, 44.0)
}

impl App {
    pub(super) fn card_edit_step(&mut self) {
        let Some(mut edit) = self.card_edit.take() else {
            return;
        };
        if let Some(text) = edit.text_mut() {
            for ch in typed_text().chars().filter(|c| !c.is_control()) {
                if text.chars().count() < NAME_MAX {
                    text.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                text.pop();
            }
        }
        if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Down) {
            edit.field = (edit.field + 1) % FIELDS;
        }
        if is_key_pressed(KeyCode::Up) {
            edit.field = (edit.field + FIELDS - 1) % FIELDS;
        }
        let right = is_key_pressed(KeyCode::Right);
        if is_key_pressed(KeyCode::Left) || right {
            let step = if right { 1 } else { -1 };
            match edit.field {
                2 => edit.shift_flag(step),
                3 => {
                    edit.seeking = ((edit.seeking as isize + step).rem_euclid(SEEKING_COUNT as isize)) as u8
                }
                _ => {}
            }
        }
        let save = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) || clicked_save(self);
        if save {
            self.player.card = Card {
                name: edit.name.trim().to_string(),
                flag: edit.flag.clone(),
                title: edit.title.trim().to_string(),
                seeking: edit.seeking,
            };
            self.player.save();
            self.audio.sfx("confirm");
            return;
        }
        if is_key_pressed(KeyCode::Escape) || clicked_cancel(self) {
            self.audio.sfx("select");
            return;
        }
        self.card_edit = Some(edit);
    }

    pub(super) fn card_edit_draw(&self) {
        let Some(edit) = self.card_edit.as_ref() else {
            return;
        };
        let ui = &self.ui;
        let (x, y, w, h) = (MODAL_X, MODAL_Y, MODAL_W, MODAL_H);
        ui.adapted_modal(x, y, w, h, VW, VH);
        ui.text(tr3("Editar ficha", "Edit card", "Editar ficha"), x + 40.0, y + 52.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let labels = [
            tr3("Nome", "Name", "Nombre"),
            tr3("Título", "Title", "Título"),
            tr3("Bandeira", "Flag", "Bandera"),
            tr3("Procurando", "Looking for", "Buscando"),
        ];
        let caret = if (ui.t * 2.0) as i32 % 2 == 0 { "|" } else { "" };
        for (i, label) in labels.iter().enumerate() {
            let row = ROW_Y + i as f32 * ROW_STEP;
            let focus = edit.field as usize == i;
            if focus {
                ui.adapted_plate(x + 230.0, row, 500.0, 46.0, Some(crate::tweaks::accent(0.45)));
            } else {
                ui.adapted_field(x + 230.0, row, 500.0, 46.0);
            }
            ui.text(label, x + 40.0, row + 30.0, 22.0, hex("#a49cc0"), Align::Left, false);
            let (tx, ty) = (x + 250.0, row + 31.0);
            match i {
                0 => drop(ui.text(&format!("{}{}", edit.name, if focus { caret } else { "" }), tx, ty, 24.0, WHITE, Align::Left, false)),
                1 => drop(ui.text(&format!("{}{}", edit.title, if focus { caret } else { "" }), tx, ty, 24.0, WHITE, Align::Left, false)),
                2 => {
                    ui.flag(&edit.flag, tx, row + 8.0, 54.0, 1.0);
                    ui.text(&edit.flag, tx + 70.0, ty, 22.0, WHITE, Align::Left, true);
                }
                _ => drop(ui.text(seeking_label(edit.seeking), tx, ty, 22.0, WHITE, Align::Left, false)),
            }
        }
        ui.adapted_plate(x + w - 330.0, y + h - 72.0, 150.0, 44.0, Some(alpha(hex("#2ecc71"), 0.8)));
        ui.text(crate::i18n::t("common.ok"), x + w - 255.0, y + h - 44.0, 20.0, WHITE, Align::Center, true);
        ui.adapted_plate(x + w - 170.0, y + h - 72.0, 140.0, 44.0, Some(alpha(hex("#ff4d6d"), 0.8)));
        ui.text(crate::i18n::t("common.cancel"), x + w - 100.0, y + h - 44.0, 20.0, WHITE, Align::Center, true);
        ui.text(
            tr3("Tab/Cima/Baixo campo · Esq/Dir escolhe · Enter salva · Esc cancela", "Tab/Up/Down field · Left/Right choose · Enter saves · Esc cancels", "Tab/Arriba/Abajo campo · Izq/Der elige · Enter guarda · Esc cancela"),
            x + w / 2.0,
            y + h - 14.0,
            15.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }
}
