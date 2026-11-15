use super::*;
use std::cell::RefCell;

const NATIONALITY_Y: f32 = 668.0;
const BIO_Y: f32 = 684.0;
const BIO_LINE: f32 = 14.0;
const BIO_SIZE: f32 = 13.0;
const BIO_WIDTH: f32 = 1220.0;
const BIO_LINES: usize = 2;

#[derive(Default)]
struct Wrapped {
    key: String,
    lang: &'static str,
    lines: Vec<String>,
}

thread_local! {
    static WRAPPED: RefCell<Wrapped> = RefCell::new(Wrapped::default());
}

impl App {
    pub(crate) fn draw_select_profile(&self, i: usize, accent: Color) {
        let entry = &self.reg.chars[i];
        let ink = Color::new(0.85, 0.82, 0.94, 0.8);
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
        let heading = format!(
            "{}: {}",
            crate::profile::nationality_label(),
            profile.nationality.pick(lang)
        )
        .to_uppercase();
        self.ui.text(
            &heading,
            VW / 2.0,
            NATIONALITY_Y,
            12.0,
            accent,
            Align::Center,
            true,
        );
        WRAPPED.with(|cell| {
            let mut wrapped = cell.borrow_mut();
            if wrapped.key != entry.key || wrapped.lang != lang {
                wrapped.lines = crate::profile::wrap(profile.bio.pick(lang), BIO_WIDTH, |s| {
                    self.ui.measure(s, BIO_SIZE, false)
                });
                wrapped.key.clone_from(&entry.key);
                wrapped.lang = lang;
            }
            for (n, line) in wrapped.lines.iter().take(BIO_LINES).enumerate() {
                self.ui.text(
                    line,
                    VW / 2.0,
                    BIO_Y + n as f32 * BIO_LINE,
                    BIO_SIZE,
                    ink,
                    Align::Center,
                    false,
                );
            }
        });
    }
}
