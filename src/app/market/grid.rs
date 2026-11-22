use super::rows::Row;
use super::*;

pub(super) const CARD_W: f32 = 200.0;
pub(super) const CARD_H: f32 = 186.0;
pub(super) const CARD_GAP: f32 = 8.0;
pub(super) const GRID_COLS: usize = 4;
pub(super) const GRID_ROWS: usize = 2;
const ART_H: f32 = 112.0;

impl App {
    pub(super) fn market_card(&mut self, row: &Row, x: f32, y: f32, focused: bool, hover: bool, downloading: bool) {
        self.ui.rect(x, y, CARD_W, CARD_H, Color::new(0.06, 0.04, 0.12, if focused || hover { 0.95 } else { 0.8 }));
        self.market_draw_portrait(row, x + 4.0, y + 4.0, CARD_W - 8.0, ART_H);
        let marked = self.market_ui.marked.contains(&row.pair());
        self.market_checkbox(x + 10.0, y + 10.0, 22.0, marked);
        if let Some(f) = row.format {
            self.market_format_badge(f, x + CARD_W - 70.0, y + 10.0, 60.0, 20.0);
        }
        self.ui.text(&short(&row.name, 22), x + 10.0, y + ART_H + 26.0, 17.0, WHITE, Align::Left, true);
        self.ui.text(&self.market_kind_label(row), x + 10.0, y + ART_H + 46.0, 12.0, kind_color(row.kind), Align::Left, true);
        self.ui.text(&fmt_bytes(row.bytes), x + CARD_W - 10.0, y + ART_H + 46.0, 13.0, hex("#a49cc0"), Align::Right, false);
        let (label, color) = self.market_status(row, downloading);
        self.ui.text(label, x + 10.0, y + ART_H + 66.0, 14.0, color, Align::Left, true);
        if focused {
            self.ui.rect_lines(x, y, CARD_W, CARD_H, 3.0, crate::tweaks::accent(1.0));
        } else if row.outdated {
            self.ui.rect_lines(x, y, CARD_W, CARD_H, 2.0, hex("#ff9e3d"));
        }
    }
}
