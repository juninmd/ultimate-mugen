use super::*;
use crate::ui_tokens::*;

impl App {
    pub(super) fn select_palette_strip(&mut self, side: usize, pal: i32, team_mode: bool) {
        let count = crate::settings::PALETTE_COUNT as i32;
        let px = if side == 0 { 24.0 } else { VW - 304.0 };
        let (x, y, w, h) = (px + 12.0, 138.0, 256.0, 46.0);
        crate::menu_art::glass(&self.ui, x, y, w, h, 1.0);
        let label = format!("{} {pal}/{count}", crate::i18n::t("select.palette"));
        self.ui.text(&label, x + 8.0, y + 15.0, 12.0, WHITE, Align::Left, true);
        self.ui.text(
            crate::i18n::t("select.palette_hint"),
            x + w - 8.0,
            y + 15.0,
            10.0,
            Color::new(1.0, 1.0, 1.0, ALPHA_MUTED),
            Align::Right,
            false,
        );
        for n in 1..=count {
            let bx = x + 2.0 + (n - 1) as f32 * 20.0;
            let by = y + 22.0;
            let on = n == pal;
            if self.clicked_rect(bx, by, 18.0, 20.0) {
                if team_mode {
                    self.pal_pick = n;
                } else {
                    self.pal[side.min(1)] = n;
                }
                self.settings.pal1 = self.side_pal(0) as usize;
                self.settings.pal2 = self.side_pal(1) as usize;
                self.settings.save();
            }
            let fill = if on { hex("#ffcc33") } else { Color::new(1.0, 1.0, 1.0, 0.12) };
            self.ui.rect(bx, by, 18.0, 20.0, fill);
            let ink = if on { BLACK } else { WHITE };
            self.ui.text(&n.to_string(), bx + 9.0, by + 15.0, 11.0, ink, Align::Center, true);
        }
    }
}
