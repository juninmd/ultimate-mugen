use super::*;
use std::cell::RefCell;

const STRIP_Y: f32 = 653.5;
const STRIP_H: f32 = 50.0;
const EDGE: f32 = 40.0;
const FLAG_Y: f32 = 661.0;
const FLAG_W: f32 = 54.0;
const LABEL_Y: f32 = 673.0;
const VALUE_Y: f32 = 693.0;
const VALUE_SIZE: f32 = 16.0;
const VALUE_WIDTH: f32 = 176.0;
const DIVIDER_GAP: f32 = 14.0;
const BIO_Y: f32 = 677.0;
const BIO_LINE: f32 = 17.0;
const BIO_SIZES: [f32; 3] = [15.0, 14.0, 13.0];
const BIO_LINES: usize = 2;
const CARD_FLAG_W: f32 = 42.0;
const TILE_FLAG_RATIO: f32 = 0.2;

#[derive(Default)]
struct Wrapped {
    key: String,
    lang: &'static str,
    size: f32,
    lines: Vec<String>,
}

thread_local! {
    static WRAPPED: RefCell<Wrapped> = RefCell::new(Wrapped::default());
}

impl App {
    fn country(&self, i: usize) -> Option<&str> {
        crate::profile::of(&self.reg.chars[i]).map(|profile| profile.country.as_str())
    }

    pub(crate) fn draw_tile_flag(&self, i: usize, x: f32, y: f32, tile_width: f32, name_top: f32) {
        if let Some(country) = self.country(i) {
            let width = (tile_width * TILE_FLAG_RATIO).clamp(12.0, 20.0);
            let top = name_top - crate::flag::height_for(width) - 4.0;
            self.ui
                .flag(country, x + tile_width - width - 4.0, top, width, 0.95);
        }
    }

    pub(crate) fn draw_card_flag(&self, i: usize, right: f32, band_top: f32) {
        if let Some(country) = self.country(i) {
            let height = crate::flag::height_for(CARD_FLAG_W);
            self.ui.flag(
                country,
                right - CARD_FLAG_W - 12.0,
                band_top - height - 10.0,
                CARD_FLAG_W,
                1.0,
            );
        }
    }

    fn fit_bio(&self, bio: &str, width: f32) -> (f32, Vec<String>) {
        let mut fitted = (BIO_SIZES[0], Vec::new());
        for size in BIO_SIZES {
            fitted = (
                size,
                crate::profile::wrap(bio, width, |s| self.ui.measure(s, size, false)),
            );
            if fitted.1.len() <= BIO_LINES {
                break;
            }
        }
        fitted
    }

    pub(crate) fn draw_select_profile(&self, i: usize, accent: Color) {
        let entry = &self.reg.chars[i];
        let ink = Color::new(0.88, 0.86, 0.96, 0.9);
        let Some(profile) = crate::profile::of(entry) else {
            self.ui.text(
                &entry.description,
                VW / 2.0,
                680.0,
                16.0,
                ink,
                Align::Center,
                false,
            );
            return;
        };
        let lang = crate::profile::lang();
        self.ui.gradient_v(
            0.0,
            STRIP_Y,
            VW,
            STRIP_H,
            alpha(accent, 0.16),
            alpha(accent, 0.0),
        );
        self.ui.rect(0.0, STRIP_Y, 4.0, STRIP_H, accent);
        let has_flag = self.ui.flag(&profile.country, EDGE, FLAG_Y, FLAG_W, 1.0);
        let label_x = if has_flag { EDGE + FLAG_W + 12.0 } else { EDGE };
        let label = crate::profile::nationality_label().to_uppercase();
        self.ui.text(
            &label,
            label_x,
            LABEL_Y,
            11.0,
            Color::new(1.0, 1.0, 1.0, 0.55),
            Align::Left,
            true,
        );
        let value = profile.nationality.pick(lang).to_uppercase();
        let natural = self.ui.measure(&value, VALUE_SIZE, true).max(1.0);
        let value_size = VALUE_SIZE * (VALUE_WIDTH / natural).min(1.0);
        self.ui.text(
            &value,
            label_x,
            VALUE_Y,
            value_size,
            accent,
            Align::Left,
            true,
        );
        let divider_x = label_x + VALUE_WIDTH + DIVIDER_GAP;
        self.ui.rect(
            divider_x,
            FLAG_Y,
            1.0,
            crate::flag::height_for(FLAG_W),
            Color::new(1.0, 1.0, 1.0, 0.18),
        );
        let bio_x = divider_x + DIVIDER_GAP;
        WRAPPED.with(|cell| {
            let mut wrapped = cell.borrow_mut();
            if wrapped.key != entry.key || wrapped.lang != lang {
                let (size, lines) = self.fit_bio(profile.bio.pick(lang), VW - EDGE - bio_x);
                wrapped.size = size;
                wrapped.lines = lines;
                wrapped.key.clone_from(&entry.key);
                wrapped.lang = lang;
            }
            let top = if wrapped.lines.len() < BIO_LINES {
                BIO_Y + BIO_LINE / 2.0
            } else {
                BIO_Y
            };
            for (n, line) in wrapped.lines.iter().take(BIO_LINES).enumerate() {
                self.ui.text(
                    line,
                    bio_x,
                    top + n as f32 * BIO_LINE,
                    wrapped.size,
                    ink,
                    Align::Left,
                    false,
                );
            }
        });
    }
}
