use super::*;
use crate::controls::{face, Glyphs};
use crate::gamepad::PadKind;
use ume_engine::input::k;

impl App {
    pub(super) fn controls_screen(&mut self) {
        if self.key_edit.is_some() {
            self.key_editor();
            return;
        }
        self.ui.background();
        self.ui.text(
            crate::i18n::t("controls.title"),
            60.0,
            70.0,
            34.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        let kl = |i: usize| crate::keybinds::label(self.c.keys.slots[0][i]);
        let touch = self.c.touch.enabled;
        let mut cols = vec![
            Glyphs::Keyboard,
            Glyphs::Pad(PadKind::Xbox),
            Glyphs::Pad(PadKind::PlayStation),
        ];
        if touch {
            cols.insert(0, Glyphs::Touch);
        } else {
            cols.push(Glyphs::Touch);
        }
        let head = |g: Glyphs| match g {
            Glyphs::Keyboard => crate::i18n::t("controls.keyboard"),
            Glyphs::Touch => crate::i18n::t("controls.touch"),
            Glyphs::Pad(p) => p.label(),
        };
        let rows: [(&str, &[u32], &str); 7] = [
            (crate::i18n::t("controls.move"), &[], ""),
            (crate::i18n::t("controls.punch"), &[k::X, k::Y], " "),
            (crate::i18n::t("controls.kick"), &[k::A, k::B], " "),
            (crate::i18n::t("controls.special"), &[k::D2], ""),
            (crate::i18n::t("controls.super"), &[k::D2, k::W], "+"),
            (crate::i18n::t("controls.awaken"), &[k::T], ""),
            (crate::i18n::t("controls.pause"), &[k::S], ""),
        ];
        let move_text = |g: Glyphs| match g {
            Glyphs::Keyboard => format!("{} {} {} {} / setas", kl(0), kl(1), kl(2), kl(3)),
            Glyphs::Touch => "Analógico esquerdo".into(),
            Glyphs::Pad(_) => "Analógico ou direcional".into(),
        };
        let xs = [60.0, 280.0, 520.0, 760.0, 1000.0];
        let (top, rh) = (120.0, 60.0);
        for (j, g) in cols.iter().enumerate() {
            let on = matches!((self.c.glyphs, g), (a, b) if a == *b);
            self.ui.text(
                head(*g),
                xs[j + 1],
                top,
                24.0,
                if on { hex("#ffcc33") } else { hex("#22e8ff") },
                Align::Left,
                true,
            );
        }
        for (i, (act, bits, sep)) in rows.iter().enumerate() {
            let y = top + 18.0 + i as f32 * rh;
            self.ui.rect(
                50.0,
                y,
                1180.0,
                rh - 8.0,
                Color::new(1.0, 1.0, 1.0, if i % 2 == 0 { 0.07 } else { 0.03 }),
            );
            self.ui.text(
                act,
                xs[0],
                y + 36.0,
                26.0,
                hex("#ffcc33"),
                Align::Left,
                true,
            );
            for (j, g) in cols.iter().enumerate() {
                if bits.is_empty() {
                    self.ui.text(
                        &move_text(*g),
                        xs[j + 1],
                        y + 34.0,
                        20.0,
                        WHITE,
                        Align::Left,
                        false,
                    );
                    continue;
                }
                let mut cx = xs[j + 1];
                let bits: &[u32] = if *g == Glyphs::Touch && bits.contains(&k::W) {
                    &[k::W]
                } else {
                    bits
                };
                for (n, b) in bits.iter().enumerate() {
                    if n > 0 {
                        cx += if sep.trim().is_empty() {
                            8.0
                        } else {
                            self.ui.text(
                                sep,
                                cx + 4.0,
                                y + 36.0,
                                24.0,
                                hex("#a49cc0"),
                                Align::Left,
                                true,
                            ) + 10.0
                        };
                    }
                    cx += self.ui.face(face(*g, &self.c.keys, *b), cx, y + 44.0, 36.0);
                }
            }
        }
        let pads = self.c.pad_info.clone();
        let status = if pads.is_empty() {
            crate::i18n::t("controls.none").to_string()
        } else {
            format!(
                "Conectado: {}",
                pads.iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        self.ui.text(
            &status,
            60.0,
            585.0,
            20.0,
            hex("#a49cc0"),
            Align::Left,
            false,
        );
        for (i, p) in pads.iter().enumerate() {
            if p.bits & ume_engine::input::k::A != 0 && self.c.pressed(nav::OK) {
                self.c.rumble(i, 0.8, 0.8, 300);
            }
        }
        self.ui.rect(60.0, 610.0, 200.0, 60.0, hex("#ff4d6d"));
        self.ui.text(
            crate::i18n::t("controls.back"),
            160.0,
            649.0,
            26.0,
            WHITE,
            Align::Center,
            true,
        );
        let back = self.clicked_rect(60.0, 610.0, 200.0, 60.0);
        if !touch {
            self.ui
                .rect(300.0, 610.0, 360.0, 60.0, Color::new(1.0, 1.0, 1.0, 0.1));
            self.ui.text(
                crate::i18n::t("controls.customize"),
                480.0,
                649.0,
                22.0,
                WHITE,
                Align::Center,
                true,
            );
            if is_key_pressed(KeyCode::Tab) || self.clicked_rect(300.0, 610.0, 360.0, 60.0) {
                self.key_edit = Some(keyedit::KeyEdit {
                    slot: 0,
                    cur: 0,
                    waiting: false,
                    msg: String::new(),
                });
            }
        }
        if back || self.c.pressed(nav::BACK) {
            self.screen = match self.settings_back.take() {
                Some((tab, from_fight)) => Screen::Settings {
                    tab,
                    cur: 1,
                    from_fight,
                },
                None => Screen::Menu { cur: 11 },
            };
        }
    }
}