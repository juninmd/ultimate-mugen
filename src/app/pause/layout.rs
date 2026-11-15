#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x < o.right() && o.x < self.right() && self.y < o.bottom() && o.y < self.bottom()
    }
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub compact: bool,
    pub panel: Rect,
    pub title: (f32, f32),
    pub tabs: [Rect; 3],
    pub slots: [Rect; 4],
    pub heading_y: f32,
    pub list: Rect,
    pub footer: Rect,
    pub row_h: f32,
    pub pitch: f32,
    pub font: f32,
}

pub const COMPACT_BELOW: f32 = 0.72;
pub const HUD_TOP: f32 = 100.0;
pub const HUD_BOTTOM: f32 = 650.0;

impl Layout {
    pub fn new(scale: f32) -> Self {
        let compact = scale < COMPACT_BELOW;
        let (panel, header, tab_w, tab_h, slot_h, row_h, footer_h, font) = if compact {
            (Rect::new(12.0, HUD_TOP, 1256.0, HUD_BOTTOM - HUD_TOP), 80.0, 190.0, 54.0, 80.0, 62.0, 46.0, 1.25)
        } else {
            (Rect::new(80.0, HUD_TOP, 1120.0, HUD_BOTTOM - HUD_TOP), 70.0, 150.0, 42.0, 66.0, 44.0, 46.0, 1.0)
        };
        let pad = 24.0;
        let gap = 8.0;
        let tab_total = tab_w * 3.0 + gap * 2.0;
        let tab_x = panel.right() - pad - tab_total;
        let tab_y = panel.y + (header - tab_h) / 2.0;
        let tabs = std::array::from_fn(|i| Rect::new(tab_x + i as f32 * (tab_w + gap), tab_y, tab_w, tab_h));
        let slot_gap = 12.0;
        let slot_w = (panel.w - pad * 2.0 - slot_gap * 3.0) / 4.0;
        let slot_y = panel.y + header + 6.0;
        let slots = std::array::from_fn(|i| Rect::new(panel.x + pad + i as f32 * (slot_w + slot_gap), slot_y, slot_w, slot_h));
        let heading_y = slot_y + slot_h + 28.0;
        let list_y = heading_y + 14.0;
        let footer = Rect::new(panel.x + pad, panel.bottom() - footer_h, panel.w - pad * 2.0, footer_h - 10.0);
        let list = Rect::new(panel.x + pad, list_y, panel.w - pad * 2.0, footer.y - 6.0 - list_y);
        Layout {
            compact,
            panel,
            title: (panel.x + pad, panel.y + header * 0.64),
            tabs,
            slots,
            heading_y,
            list,
            footer,
            row_h,
            pitch: row_h + 4.0,
            font,
        }
    }

    pub fn shifted(&self, dy: f32) -> Layout {
        let mv = |r: Rect| Rect::new(r.x, r.y + dy, r.w, r.h);
        Layout {
            panel: mv(self.panel),
            title: (self.title.0, self.title.1 + dy),
            tabs: self.tabs.map(mv),
            slots: self.slots.map(mv),
            heading_y: self.heading_y + dy,
            list: mv(self.list),
            footer: mv(self.footer),
            ..self.clone()
        }
    }

    pub fn move_row(&self, i: usize, scroll: f32) -> Rect {
        Rect::new(self.list.x, self.list.y + i as f32 * self.pitch - scroll, self.list.w - SCROLLBAR_W - 6.0, self.row_h)
    }

    pub fn action_row(&self, i: usize, count: usize) -> Rect {
        let rows = count.max(3) as f32;
        let h = (self.row_h + 14.0 * self.font).min((self.list.h - 4.0) / rows - 10.0);
        let w = self.list.w.min(640.0);
        let block = rows * (h + 10.0);
        let top = self.list.y + ((self.list.h - block) / 3.0).max(4.0);
        Rect::new(self.list.x + (self.list.w - w) / 2.0, top + i as f32 * (h + 10.0), w, h)
    }

    pub fn visible_rows(&self) -> usize {
        ((self.list.h / self.pitch).floor() as usize).max(1)
    }

    pub fn max_scroll(&self, total: usize) -> f32 {
        (total as f32 * self.pitch - self.list.h).max(0.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Columns {
    pub badge_x: f32,
    pub name_x: f32,
    pub name_w: f32,
    pub input_x: f32,
    pub input_w: f32,
    pub cost_w: f32,
    pub cost_right: f32,
}

pub const SCROLLBAR_W: f32 = 12.0;

impl Columns {
    pub fn new(row_w: f32, compact: bool) -> Self {
        let name_w = if compact { 250.0 } else { 270.0 };
        let cost_w = if compact { 170.0 } else { 150.0 };
        let badge_x = 12.0;
        let name_x = 64.0;
        let input_x = name_x + name_w + 16.0;
        let cost_right = row_w - SCROLLBAR_W - 18.0;
        let input_w = (cost_right - cost_w - 12.0 - input_x).max(0.0);
        Columns { badge_x, name_x, name_w, input_x, input_w, cost_w, cost_right }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALES: [f32; 8] = [0.4, 0.55, 0.625, 0.71, 0.73, 1.0, 1.5, 2.0];

    #[test]
    fn nothing_overlaps_at_any_scale() {
        for s in SCALES {
            let l = Layout::new(s);
            let mut parts = vec![l.list, l.footer];
            parts.extend(l.tabs);
            parts.extend(l.slots);
            for (i, a) in parts.iter().enumerate() {
                assert!(l.panel.contains(a.x, a.y) && l.panel.contains(a.right(), a.bottom()), "{s} {i} inside panel");
                for b in &parts[i + 1..] {
                    assert!(!a.overlaps(b), "{s}: {a:?} overlaps {b:?}");
                }
            }
        }
    }

    #[test]
    fn panel_stays_between_the_top_and_bottom_hud() {
        for s in SCALES {
            let l = Layout::new(s);
            assert!(l.panel.y >= HUD_TOP && l.panel.bottom() <= HUD_BOTTOM, "{s}");
        }
    }

    #[test]
    fn title_stays_left_of_the_tabs() {
        for s in SCALES {
            let l = Layout::new(s);
            assert!(l.title.0 + 300.0 * l.font < l.tabs[0].x, "{s}");
        }
    }

    #[test]
    fn compact_mode_has_touch_sized_rows() {
        let c = Layout::new(0.625);
        assert!(c.compact && c.row_h * 0.625 >= 38.0 && c.tabs[0].h * 0.625 >= 32.0);
        assert!(!Layout::new(1.0).compact);
    }

    #[test]
    fn lists_show_enough_rows() {
        for s in SCALES {
            let min = if s < COMPACT_BELOW { 4 } else { 6 };
            assert!(Layout::new(s).visible_rows() >= min, "{s}");
        }
    }

    #[test]
    fn columns_never_overlap() {
        for compact in [false, true] {
            let l = Layout::new(if compact { 0.5 } else { 1.0 });
            let c = Columns::new(l.list.w, compact);
            assert!(c.badge_x + 44.0 <= c.name_x);
            assert!(c.name_x + c.name_w < c.input_x);
            assert!(c.input_x + c.input_w < c.cost_right - c.cost_w + 1.0);
            assert!(c.input_w > 300.0);
        }
    }

    #[test]
    fn shifting_moves_every_rect_together() {
        let l = Layout::new(1.0);
        let t = l.shifted(10.0);
        assert_eq!(t.panel.y, l.panel.y + 10.0);
        assert_eq!(t.slots[3].y, l.slots[3].y + 10.0);
        assert_eq!(t.list.h, l.list.h);
    }

    #[test]
    fn action_rows_fit_the_list() {
        for s in SCALES {
            let l = Layout::new(s);
            for count in [3, 4] {
                let last = l.action_row(count - 1, count);
                assert!(last.bottom() <= l.list.bottom() && last.h > 0.0, "{s} {count}");
            }
        }
    }

    #[test]
    fn scroll_range_matches_the_overflow() {
        let l = Layout::new(1.0);
        assert_eq!(l.max_scroll(2), 0.0);
        assert!(l.max_scroll(40) > 0.0);
    }
}
