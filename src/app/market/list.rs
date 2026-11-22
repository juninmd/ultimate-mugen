use super::rows::Row;
use super::*;

pub(super) const LIST_ROW_H: f32 = 34.0;
pub(super) const LIST_ROWS: usize = 11;

impl App {
    pub(super) fn market_list_row(&self, row: &Row, vi: usize, x: f32, y: f32, w: f32, focused: bool, downloading: bool) {
        if focused {
            self.ui.adapted_highlight(("mkt.item.x", "mkt.item.y"), x, y, w, LIST_ROW_H - 2.0);
        } else if vi % 2 == 0 {
            self.ui.adapted_row(x, y, w, LIST_ROW_H - 2.0, false);
        }
        if row.outdated {
            self.ui.rect(x, y + 2.0, 4.0, LIST_ROW_H - 6.0, hex("#ff9e3d"));
        }
        let marked = self.market_ui.marked.contains(&row.pair());
        self.market_checkbox(x + 12.0, y + 6.0, 20.0, marked);
        self.ui.rect(x + 44.0, y + 8.0, 96.0, 18.0, alpha(kind_color(row.kind), 0.35));
        self.ui.text(&self.market_kind_label(row), x + 92.0, y + 22.0, 12.0, WHITE, Align::Center, true);
        if let Some(f) = row.format {
            self.market_format_badge(f, x + 148.0, y + 8.0, 60.0, 18.0);
        }
        self.ui.text(&short(&row.name, 32), x + 218.0, y + 23.0, 19.0, WHITE, Align::Left, false);
        self.ui.text(&fmt_bytes(row.bytes), x + w - 190.0, y + 23.0, 16.0, hex("#a49cc0"), Align::Right, false);
        let (label, color) = self.market_status(row, downloading);
        self.ui.text(label, x + w - 20.0, y + 23.0, 16.0, color, Align::Right, true);
    }
}
