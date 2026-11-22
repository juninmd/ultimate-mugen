use super::rows::Row;
use super::*;
use ume_market::view::Format;

pub(super) fn format_color(f: Format) -> Color {
    match f {
        Format::Mugen => hex("#6d9bff"),
        Format::Umf => hex("#2ee6a6"),
    }
}

pub(super) fn kind_initials(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric()).take(2).flat_map(char::to_uppercase).collect()
}

impl App {
    pub(super) fn market_format_badge(&self, f: Format, x: f32, y: f32, w: f32, h: f32) {
        self.ui.rect(x, y, w, h, alpha(format_color(f), 0.9));
        self.ui.text(f.label(), x + w / 2.0, y + h - 5.0, 13.0, hex("#0b0716"), Align::Center, true);
    }

    pub(super) fn market_status(&self, row: &Row, downloading: bool) -> (&'static str, Color) {
        if downloading {
            return (mt("baixando…"), hex("#22e8ff"));
        }
        if row.outdated {
            return (mt("DESATUALIZADO"), hex("#ff9e3d"));
        }
        match row.installed {
            Some(true) => (mt("ATIVO"), hex("#6dff9e")),
            Some(false) => (mt("instalado"), hex("#ffcc33")),
            None if !row.installable => (mt("grande demais"), hex("#ff4d6d")),
            None => ("", WHITE),
        }
    }

    pub(super) fn market_checkbox(&self, x: f32, y: f32, size: f32, marked: bool) {
        self.ui.rect(x, y, size, size, Color::new(0.03, 0.02, 0.07, 0.75));
        self.ui.rect_lines(x, y, size, size, 2.0, WHITE);
        if marked {
            self.ui.rect(x + 4.0, y + 4.0, size - 8.0, size - 8.0, hex("#6dff9e"));
        }
    }
}
