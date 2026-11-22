use super::*;
use crate::i18n::tr3;
use crate::ui::{hex, Align, VW};
mod preview;

const TOUCH_TAB: usize = 7;
const TOUCH_SCALE_MIN: f32 = 0.7;
const TOUCH_SCALE_MAX: f32 = 1.5;
const TOUCH_OPACITY_MIN: f32 = 0.2;

impl App {
    pub(super) fn touch_settings_screen(&mut self, from_fight: bool) {
        let Screen::Settings {
            mut tab, mut cur, ..
        } = self.screen
        else {
            return;
        };
        let mut ok = self.c.pressed(nav::OK | nav::START);
        let mut dir = (self.c.pressed(nav::RIGHT) as i32) - (self.c.pressed(nav::LEFT) as i32);
        if self.c.pressed(nav::UP) {
            cur = cur.saturating_sub(1);
            self.audio.sfx("select");
        }
        if self.c.pressed(nav::DOWN) {
            cur = (cur + 1).min(3);
            self.audio.sfx("select");
        }
        if cur == 0 && dir != 0 {
            tab = (tab as i32 + dir).rem_euclid(super::TABS.len() as i32) as usize;
            dir = 0;
            ok = false;
            self.audio.sfx("select");
        }

        crate::screen_art::ScreenArt::Workshop.background(&self.ui);
        self.ui.adapted_header(
            crate::i18n::tr("CONFIGURAÇÕES", "SETTINGS"),
            &format!("Ultimate MUGEN  v{}", crate::update::VERSION),
            VW,
        );
        let tab_w = (VW - 112.0) / super::TABS.len() as f32;
        for (i, (pt, en, es)) in super::TABS.iter().enumerate() {
            let x = 56.0 + i as f32 * tab_w;
            let active = i == tab;
            if self.clicked_rect(x, 80.0, tab_w, 38.0) {
                tab = i;
                cur = 0;
                self.audio.sfx("select");
            }
            self.ui.adapted_tab(x, 80.0, tab_w - 4.0, 38.0, active);
            self.ui.text(
                tr3(pt, en, es),
                x + tab_w / 2.0,
                106.0,
                18.0,
                if active { WHITE } else { hex("#a49cc0") },
                Align::Center,
                true,
            );
        }
        if tab != TOUCH_TAB {
            self.screen = Screen::Settings {
                tab,
                cur: 0,
                from_fight,
            };
            return;
        }

        crate::menu_art::glass(&self.ui, 56.0, 136.0, VW - 112.0, 524.0, 1.0);
        let prefs = crate::tweaks::current();
        let rows = [
            (
                tr3(
                    "Controles para canhotos",
                    "Left-handed controls",
                    "Controles para zurdos",
                ),
                if prefs.touch_left_handed {
                    tr3("Canhoto", "Left-handed", "Zurdo")
                } else {
                    tr3("Padrão", "Default", "Normal")
                },
                tr3(
                    "Move o direcional para a direita",
                    "Moves the stick to the right",
                    "Mueve el direccional a la derecha",
                ),
            ),
            (
                tr3(
                    "Tamanho dos controles",
                    "Control size",
                    "Tamaño de los controles",
                ),
                &format!("{}%", (prefs.touch_scale * 100.0).round() as u32),
                tr3(
                    "Ajuste o tamanho dos botões e do direcional",
                    "Adjust the buttons and stick size",
                    "Ajusta el tamaño de los botones y el direccional",
                ),
            ),
            (
                tr3(
                    "Opacidade dos controles",
                    "Control opacity",
                    "Opacidad de los controles",
                ),
                &format!("{}%", (prefs.touch_opacity * 100.0).round() as u32),
                tr3(
                    "Ajuste a visibilidade durante a luta",
                    "Adjust visibility during a fight",
                    "Ajusta la visibilidad durante el combate",
                ),
            ),
        ];
        for (i, (label, value, hint)) in rows.iter().enumerate() {
            let y = 164.0 + i as f32 * 74.0;
            let focused = cur == i + 1;
            self.ui.adapted_row(88.0, y, VW - 176.0, 60.0, true);
            if focused {
                self.ui.adapted_highlight(
                    ("touch.row.x", "touch.row.y"),
                    88.0,
                    y,
                    VW - 176.0,
                    60.0,
                );
            }
            self.ui
                .text(label, 110.0, y + 25.0, 22.0, WHITE, Align::Left, true);
            self.ui.text(
                hint,
                110.0,
                y + 48.0,
                14.0,
                hex("#a49cc0"),
                Align::Left,
                false,
            );
            self.ui.text(
                value,
                VW - 110.0,
                y + 36.0,
                22.0,
                hex("#ffcc33"),
                Align::Right,
                true,
            );
            if self.clicked_rect(88.0, y, VW - 176.0, 60.0) {
                cur = i + 1;
                let p = self.ui.to_virtual(mouse_position().into());
                let step = if p.x < VW * 0.5 { -1 } else { 1 };
                self.adjust_touch(i, step);
            }
        }
        preview::draw(&self.ui, &prefs);

        if cur > 0 && (dir != 0 || ok) {
            let step = if dir == 0 { 1 } else { dir };
            self.adjust_touch(cur - 1, step);
            self.audio.sfx("select");
        }
        let back = self.c.pressed(nav::BACK);
        let note = if from_fight {
            tr3(
                "As alterações aparecem na luta atual",
                "Changes preview in the current fight",
                "Los cambios se aplican en el combate actual",
            )
        } else {
            crate::i18n::tr(
                "Cima/Baixo: navegar · Esq/Dir: alterar · Enter: confirmar · Esc: voltar",
                "Up/Down: navigate · Left/Right: change · Enter: confirm · Esc: back",
            )
        };
        self.ui.text(
            note,
            VW - 56.0,
            692.0,
            14.0,
            hex("#c9c2e2"),
            Align::Right,
            false,
        );
        if back {
            if from_fight {
                self.screen = Screen::Fight;
            } else {
                self.screen = Screen::Menu { cur: 7 };
            }
        } else {
            self.screen = Screen::Settings {
                tab,
                cur,
                from_fight,
            };
        }
    }

    fn adjust_touch(&mut self, row: usize, step: i32) {
        let mut prefs = crate::tweaks::current();
        match row {
            0 => prefs.touch_left_handed = !prefs.touch_left_handed,
            1 => {
                prefs.touch_scale =
                    (prefs.touch_scale + step as f32 * 0.1).clamp(TOUCH_SCALE_MIN, TOUCH_SCALE_MAX)
            }
            2 => {
                prefs.touch_opacity =
                    (prefs.touch_opacity + step as f32 * 0.1).clamp(TOUCH_OPACITY_MIN, 1.0)
            }
            _ => return,
        }
        crate::tweaks::update_touch(
            prefs.touch_scale,
            prefs.touch_opacity,
            prefs.touch_left_handed,
        );
    }
}
