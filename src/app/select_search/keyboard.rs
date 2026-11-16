use super::*;

const ROWS: [&str; 4] = ["1234567890", "QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
const TOP: f32 = 392.0;
const KEY_H: f32 = 56.0;
const GAP: f32 = 6.0;
const LEFT: f32 = 40.0;
const WIDTH: f32 = 1200.0;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Osk {
    Char(char),
    Back,
    Ok,
    Close,
}

fn key_w() -> f32 {
    (WIDTH - 9.0 * GAP) / 10.0
}

fn layout() -> Vec<(f32, f32, f32, Osk)> {
    let w = key_w();
    let mut keys = Vec::new();
    for (r, row) in ROWS.iter().enumerate() {
        let n = row.chars().count() as f32;
        let x0 = LEFT + (WIDTH - (n * w + (n - 1.0) * GAP)) / 2.0;
        let y = TOP + r as f32 * (KEY_H + GAP);
        for (k, c) in row.chars().enumerate() {
            keys.push((
                x0 + k as f32 * (w + GAP),
                y,
                w,
                Osk::Char(c.to_ascii_lowercase()),
            ));
        }
    }
    let y = TOP + ROWS.len() as f32 * (KEY_H + GAP);
    let wide = (WIDTH - 3.0 * GAP) / 6.0;
    keys.push((LEFT, y, wide, Osk::Close));
    keys.push((LEFT + wide + GAP, y, wide * 2.0, Osk::Char(' ')));
    keys.push((LEFT + wide * 3.0 + GAP * 2.0, y, wide * 1.5, Osk::Back));
    keys.push((LEFT + wide * 4.5 + GAP * 3.0, y, wide * 1.5, Osk::Ok));
    keys
}

fn label(key: Osk) -> String {
    match key {
        Osk::Char(' ') => crate::i18n::tr3("ESPAÇO", "SPACE", "ESPACIO").to_string(),
        Osk::Char(c) => c.to_ascii_uppercase().to_string(),
        Osk::Back => crate::i18n::tr3("APAGAR", "DELETE", "BORRAR").to_string(),
        Osk::Ok => "OK".to_string(),
        Osk::Close => crate::i18n::tr3("FECHAR", "CLOSE", "CERRAR").to_string(),
    }
}

impl App {
    pub(super) fn osk_shown(&self) -> bool {
        self.c.touch.enabled || std::env::var("UME_OSK").is_ok_and(|v| v != "0")
    }

    pub(super) fn osk_press(&self) -> Option<Osk> {
        if !self.osk_shown() {
            return None;
        }
        layout()
            .into_iter()
            .find(|&(x, y, w, _)| self.clicked_rect(x, y, w, KEY_H))
            .map(|(_, _, _, key)| key)
    }

    pub(super) fn draw_osk(&self) {
        let pad = 10.0;
        crate::menu_art::glass(
            &self.ui,
            LEFT - pad,
            TOP - pad,
            WIDTH + pad * 2.0,
            5.0 * (KEY_H + GAP) - GAP + pad * 2.0,
            1.0,
        );
        for (x, y, w, key) in layout() {
            let accent = match key {
                Osk::Ok => Some(hex("#3fd17a")),
                Osk::Close => Some(hex("#ff4d6d")),
                Osk::Back => Some(hex("#ffcc33")),
                Osk::Char(_) => None,
            };
            let hover = self.hover_rect(x, y, w, KEY_H);
            let fill = match accent {
                Some(c) => alpha(c, if hover { 0.55 } else { 0.35 }),
                None => Color::new(1.0, 1.0, 1.0, if hover { 0.22 } else { 0.1 }),
            };
            self.ui.rect(x, y, w, KEY_H, fill);
            self.ui
                .rect_lines(x, y, w, KEY_H, 1.0, Color::new(1.0, 1.0, 1.0, 0.18));
            self.ui.text(
                &label(key),
                x + w / 2.0,
                y + KEY_H / 2.0 + 8.0,
                22.0,
                WHITE,
                Align::Center,
                true,
            );
        }
    }
}
