use super::*;
use crate::char_tuning::{self, Tuning, PARAMS};
use crate::controls::nav;
use crate::i18n::tr;

const ROWS: usize = PARAMS.len() + 2;
const RESET_ROW: usize = ROWS - 1;
const ROW_TOP: f32 = 150.0;
const ROW_H: f32 = 44.0;

impl App {
    pub(super) fn char_tuning_screen(&mut self) {
        let Screen::CharTuning { mut ch, mut cur } = self.screen else {
            return;
        };
        let count = self.reg.chars.len().max(1);
        ch = ch.min(count - 1);
        cur = cur.min(RESET_ROW);

        if self.c.pressed(nav::DOWN) {
            cur = (cur + 1) % ROWS;
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::UP) {
            cur = (cur + ROWS - 1) % ROWS;
            self.audio.sfx("select");
        }
        let dir = (self.c.pressed(nav::RIGHT) as i32) - (self.c.pressed(nav::LEFT) as i32);
        let ok = self.c.pressed(nav::OK | nav::START);

        if self.c.pressed(nav::BACK) {
            self.screen = Screen::Settings {
                tab: 3,
                cur: 0,
                from_fight: false,
            };
            return;
        }

        let key = self.reg.chars.get(ch).map(|c| c.key.clone());
        let mut tuning = key.as_deref().map(char_tuning::get).unwrap_or_default();
        let mut changed = false;
        if cur == 0 && dir != 0 {
            ch = (ch as i32 + dir).rem_euclid(count as i32) as usize;
            self.audio.sfx("select");
        } else if (1..RESET_ROW).contains(&cur) && dir != 0 {
            let p = cur - 1;
            let spec = &PARAMS[p];
            let v = tuning.0[p] as i32 + dir * spec.step as i32;
            tuning.0[p] = v.clamp(spec.min as i32, spec.max as i32) as u16;
            changed = true;
        } else if cur == RESET_ROW && ok {
            tuning = Tuning::default();
            changed = true;
        }
        if changed {
            if let (Some(key), Some(entry)) = (key.as_deref(), self.reg.chars.get(ch)) {
                char_tuning::set(key, tuning);
                entry.drop_loaded();
            }
            self.audio.sfx("select");
        }
        self.screen = Screen::CharTuning { ch, cur };

        self.draw_char_tuning(ch, cur, &tuning);
    }

    fn draw_char_tuning(&mut self, ch: usize, cur: usize, tuning: &Tuning) {
        let name = self
            .reg
            .chars
            .get(ch)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "-".into());
        self.ui.background();
        self.ui.adapted_header(tr("AJUSTES DE PERSONAGEM", "CHARACTER TUNING"), &name, VW);
        let mut y = ROW_TOP - ROW_H;
        for (i, label) in std::iter::once(("Personagem", "Character"))
            .chain(PARAMS.iter().map(|p| p.label))
            .chain(std::iter::once(("Restaurar padrão", "Reset to default")))
            .enumerate()
        {
            let focused = i == cur;
            y += ROW_H;
            self.ui.adapted_row(48.0, y - 30.0, VW - 96.0, ROW_H - 6.0, true);
            if focused {
                self.ui.adapted_highlight(("tune.x", "tune.y"), 48.0, y - 30.0, VW - 96.0, ROW_H - 6.0);
            }
            let text = tr(label.0, label.1);
            self.ui.text(text, 80.0, y, 26.0, if focused { WHITE } else { hex("#d8d0f0") }, Align::Left, focused);
            let value = match i {
                0 => format!("‹ {name} ›"),
                _ if i == RESET_ROW => String::new(),
                _ => format!("‹ {}% ›", tuning.0[i - 1]),
            };
            self.ui.text(&value, VW - 80.0, y, 26.0, if focused { WHITE } else { hex("#22e8ff") }, Align::Right, focused);
        }
        self.ui.text(
            tr(
                "Cima/Baixo: navegar · Esq/Dir: ajustar · Enter: restaurar · Esc: voltar",
                "Up/Down: navigate · Left/Right: adjust · Enter: reset · Esc: back",
            ),
            VW / 2.0,
            VH - 18.0,
            18.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }
}
