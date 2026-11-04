//! Keyboard rebinding screen (reached from CONTROLES): one column per player, pick an action and press the new key.
use super::*;
use crate::keybinds::{self, ACTIONS};

pub struct KeyEdit {
    pub slot: usize,
    pub cur: usize,
    pub waiting: bool,
    pub msg: String,
}

impl App {
    pub(super) fn key_editor(&mut self) {
        let Some(mut ed) = self.key_edit.take() else {
            return;
        };
        self.ui.background();
        self.ui.text(
            "TECLADO · PERSONALIZAR",
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        let n = ACTIONS.len();
        let mut close = false;
        if ed.waiting {
            if is_key_pressed(KeyCode::Escape) {
                ed.waiting = false;
                ed.msg = "Cancelado.".into();
            } else if let Some(kc) = get_last_key_pressed() {
                if keybinds::id(kc).is_some() {
                    self.c.keys.assign(ed.slot, ed.cur, kc);
                    self.c.keys.save();
                    ed.msg = format!("{}: {}", ACTIONS[ed.cur].0, keybinds::label(kc));
                    ed.waiting = false;
                } else {
                    ed.msg = "Essa tecla não pode ser usada. Tente outra (Esc cancela).".into();
                }
            }
        } else {
            if self.c.pressed(nav::DOWN) {
                ed.cur = (ed.cur + 1) % n;
            }
            if self.c.pressed(nav::UP) {
                ed.cur = (ed.cur + n - 1) % n;
            }
            if self.c.pressed(nav::LEFT) || self.c.pressed(nav::RIGHT) {
                ed.slot = 1 - ed.slot;
            }
            if self.c.pressed(nav::OK) {
                ed.waiting = true;
                ed.msg.clear();
                while get_last_key_pressed().is_some() {}
            }
            if is_key_pressed(KeyCode::Delete) {
                self.c.keys.reset_slot(ed.slot);
                self.c.keys.save();
                ed.msg = format!("Jogador {} restaurado ao padrão.", ed.slot + 1);
            }
            if self.c.pressed(nav::BACK) {
                close = true;
            }
        }
        for slot in 0..2 {
            let x = 40.0 + slot as f32 * 620.0;
            self.ui.text(
                &format!("Jogador {}", slot + 1),
                x,
                112.0,
                24.0,
                hex(if slot == 0 { "#22e8ff" } else { "#ff4d6d" }),
                Align::Left,
                true,
            );
            for i in 0..n {
                let y = 130.0 + i as f32 * 33.0;
                let sel = ed.slot == slot && ed.cur == i;
                if !ed.waiting && self.clicked_rect(x, y, 560.0, 30.0) {
                    ed.slot = slot;
                    ed.cur = i;
                    ed.waiting = true;
                    ed.msg.clear();
                    while get_last_key_pressed().is_some() {}
                }
                if sel {
                    self.ui.rect(
                        x - 6.0,
                        y,
                        572.0,
                        30.0,
                        Color::new(1.0, 0.8, 0.2, if ed.waiting { 0.35 } else { 0.18 }),
                    );
                }
                let kc = self.c.keys.slots[slot][i];
                let shown = if sel && ed.waiting {
                    "pressione uma tecla…"
                } else {
                    keybinds::label(kc)
                };
                self.ui
                    .text(ACTIONS[i].0, x, y + 22.0, 19.0, WHITE, Align::Left, false);
                self.ui.text(
                    shown,
                    x + 540.0,
                    y + 22.0,
                    19.0,
                    if sel && ed.waiting {
                        hex("#ffcc33")
                    } else {
                        hex("#22e8ff")
                    },
                    Align::Right,
                    true,
                );
            }
        }
        if !ed.msg.is_empty() {
            self.ui.text(
                &ed.msg,
                40.0,
                640.0,
                20.0,
                hex("#6dff9e"),
                Align::Left,
                false,
            );
        }
        self.ui.text("Cima/Baixo: ação · Esq/Dir: jogador · Enter ou clique: trocar tecla · Del: restaurar padrão do jogador · Esc: voltar", 40.0, 690.0, 18.0, hex("#a49cc0"), Align::Left, false);
        if !close {
            self.key_edit = Some(ed);
        }
    }
}
