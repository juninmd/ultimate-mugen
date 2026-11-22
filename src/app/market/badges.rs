use super::rows::Row;
use super::*;
use ume_market::packinfo::Shape;
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
    pub(super) fn market_kind_label(&self, row: &Row) -> String {
        match row.info.as_ref().map(|i| i.shape()) {
            Some(Shape::Multi(n)) => format!("{} · {n}", mt("PACOTE")),
            Some(Shape::Single(_)) => mt("PERSONAGEM").to_string(),
            Some(Shape::Stages(_)) => mt("CENÁRIO").to_string(),
            _ => kind_label(row.kind).to_string(),
        }
    }

    pub(super) fn market_contents_line(&self, rows: &[Row]) -> Option<String> {
        let row = rows.get(self.market_ui.cur)?;
        match row.info.as_ref()?.shape() {
            Shape::Multi(n) => Some(format!("{} ({n}): {}", mt("Contém"), row.info.as_ref()?.summary(6))),
            Shape::Single(name) => Some(format!("{}: {name}", mt("PERSONAGEM"))),
            _ => None,
        }
    }

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
