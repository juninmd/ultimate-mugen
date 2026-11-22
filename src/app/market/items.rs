use super::grid::{CARD_GAP, CARD_H, CARD_W, GRID_COLS, GRID_ROWS};
use super::list::{LIST_ROWS, LIST_ROW_H};
use super::rows::Row;
use super::*;

pub(super) const AREA_TOP: f32 = 88.0;
const DRAG_SLOP: f32 = 12.0;

#[derive(Clone, Copy)]
pub(super) struct Geo {
    pub cols: usize,
    pub vis: usize,
    pub line_h: f32,
}

pub(super) fn geo(grid: bool) -> Geo {
    if grid {
        Geo { cols: GRID_COLS, vis: GRID_ROWS, line_h: CARD_H + CARD_GAP }
    } else {
        Geo { cols: 1, vis: LIST_ROWS, line_h: LIST_ROW_H }
    }
}

impl App {
    pub(super) fn market_geo(&self) -> Geo {
        geo(self.market.state.grid_view)
    }

    fn market_follow_cursor(&mut self, g: Geo, total: usize) {
        let lines = total.div_ceil(g.cols);
        let line = self.market_ui.cur / g.cols;
        if line < self.market_ui.scroll {
            self.market_ui.scroll = line;
        }
        if line >= self.market_ui.scroll + g.vis {
            self.market_ui.scroll = line + 1 - g.vis;
        }
        self.market_ui.scroll = self.market_ui.scroll.min(lines.saturating_sub(g.vis));
    }

    fn market_scroll_by(&mut self, g: Geo, total: usize, delta: i64) {
        let max = total.div_ceil(g.cols).saturating_sub(g.vis) as i64;
        self.market_ui.scroll = (self.market_ui.scroll as i64 + delta).clamp(0, max) as usize;
        let first = self.market_ui.scroll * g.cols;
        let last = ((self.market_ui.scroll + g.vis) * g.cols).min(total).saturating_sub(1);
        self.market_ui.cur = self.market_ui.cur.clamp(first, last.max(first));
    }

    fn market_item_rect(g: Geo, grid: bool, rx: f32, ry: f32, rw: f32, slot: usize) -> (f32, f32, f32, f32) {
        let top = ry + AREA_TOP;
        if grid {
            let (col, line) = (slot % g.cols, slot / g.cols);
            (rx + 12.0 + col as f32 * (CARD_W + CARD_GAP), top + line as f32 * g.line_h, CARD_W, CARD_H)
        } else {
            (rx + 8.0, top + slot as f32 * g.line_h, rw - 16.0, LIST_ROW_H - 2.0)
        }
    }

    pub(super) fn market_items(&mut self, rows: &[Row], rx: f32, ry: f32, rw: f32, modal: bool) {
        let grid = self.market.state.grid_view;
        let g = geo(grid);
        let total = rows.len();
        self.market_follow_cursor(g, total);
        let area_h = g.vis as f32 * g.line_h;
        let in_area = self.hover_rect(rx, ry + AREA_TOP, rw, area_h);
        let wheel = mouse_wheel().1;
        if !modal && wheel != 0.0 && in_area {
            self.market_scroll_by(g, total, -(wheel.signum() as i64) * if grid { 1 } else { 3 });
        }
        let pointer = self.ui.to_virtual(mouse_position().into());
        let moving = mouse_delta_position().length() > 0.0 && self.market_ui.pointer.travelled <= DRAG_SLOP;
        let installing = self.market.current_name().filter(|_| self.market.busy());
        let mut hit: Option<usize> = None;
        let first = self.market_ui.scroll * g.cols;
        for (slot, (vi, row)) in rows.iter().enumerate().skip(first).take(g.cols * g.vis).enumerate() {
            let (x, y, w, h) = Self::market_item_rect(g, grid, rx, ry, rw, slot);
            let hover = pointer.x >= x && pointer.x <= x + w && pointer.y >= y && pointer.y <= y + h;
            if hover && !modal {
                hit = Some(vi);
                if moving {
                    self.market_ui.cur = vi;
                }
            }
            let focused = vi == self.market_ui.cur && self.market_ui.pane == Pane::Items;
            let downloading = installing.as_deref() == Some(row.name.as_str());
            if grid {
                self.market_card(row, x, y, focused, hover && !modal, downloading);
            } else {
                self.market_list_row(row, vi, x, y, w, focused, downloading);
            }
        }
        self.market_pointer(rows, g, hit, in_area, modal);
        self.market_scrollbar(g, total, rx, ry, rw, area_h);
    }

    fn market_pointer(&mut self, rows: &[Row], g: Geo, hit: Option<usize>, in_area: bool, modal: bool) {
        let y = self.ui.to_virtual(mouse_position().into()).y;
        let down = is_mouse_button_down(MouseButton::Left);
        if !modal && in_area && is_mouse_button_pressed(MouseButton::Left) {
            self.market_ui.pointer = Pointer { active: true, hit, last_y: y, ..Default::default() };
        }
        if !self.market_ui.pointer.active {
            return;
        }
        let total = rows.len();
        if down {
            let p = &mut self.market_ui.pointer;
            let dy = y - p.last_y;
            p.last_y = y;
            p.travelled += dy.abs();
            if p.travelled > DRAG_SLOP {
                p.carry += dy;
                let steps = (p.carry / g.line_h).trunc() as i64;
                p.carry -= steps as f32 * g.line_h;
                if steps != 0 {
                    self.market_scroll_by(g, total, -steps);
                }
            }
            return;
        }
        let Pointer { hit, travelled, .. } = std::mem::take(&mut self.market_ui.pointer);
        if let (Some(i), true) = (hit, travelled <= DRAG_SLOP) {
            self.market_ui.cur = i;
            self.market_ui.pane = Pane::Items;
            let key = rows[i].pair();
            if !self.market_ui.marked.remove(&key) {
                self.market_ui.marked.insert(key);
            }
        }
    }

    fn market_scrollbar(&self, g: Geo, total: usize, rx: f32, ry: f32, rw: f32, area_h: f32) {
        let lines = total.div_ceil(g.cols);
        if lines <= g.vis {
            return;
        }
        let frac = self.market_ui.scroll as f32 / (lines - g.vis) as f32;
        let top = ry + AREA_TOP;
        self.ui.rect(rx + rw - 6.0, top, 4.0, area_h, Color::new(1.0, 1.0, 1.0, 0.08));
        self.ui.rect(rx + rw - 6.0, top + frac * (area_h - 40.0), 4.0, 40.0, hex("#ffcc33"));
    }
}
