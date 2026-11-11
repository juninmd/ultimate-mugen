use super::select_stats::fraction;
use super::*;

const PANEL_W: f32 = 340.0;
const PANEL_H: f32 = 98.0;
const PANEL_Y: f32 = 488.0;
const ROW_H: f32 = 22.0;
const TRACK_W: f32 = 112.0;
const TRACK_GAP: f32 = 46.0;
const IMPACT_X: f32 = 640.0;
const IMPACT_Y: f32 = 318.0;

fn settle(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

impl App {
    pub(super) fn draw_versus_compare(&self, time: f32) {
        self.draw_impact_ring(time);
        let appear = settle((time - 0.9) / 0.4);
        if appear <= 0.0 {
            return;
        }
        let (left, right) = (self.char_stats(self.sel.0), self.char_stats(self.sel.1));
        let rows = [
            (crate::i18n::t("select.stat_force"), fraction(left.force, 200.0), fraction(right.force, 200.0), hex("#ffcc33")),
            (crate::i18n::t("select.stat_life"), fraction(left.life, 2000.0), fraction(right.life, 2000.0), hex("#3fd17a")),
            (crate::i18n::t("select.stat_speed"), fraction(left.speed, 6.0), fraction(right.speed, 6.0), hex("#3fa9ff")),
        ];
        let grow = settle((time - 1.2) / 0.5);
        let x = IMPACT_X - PANEL_W / 2.0;
        let y = PANEL_Y + (1.0 - appear) * 30.0;
        self.glass(x, y, PANEL_W, PANEL_H, hex("#ffcc33"), appear);
        self.ui.text(
            crate::i18n::tr3("CONFRONTO DIRETO", "HEAD TO HEAD", "CARA A CARA"),
            IMPACT_X,
            y + 17.0,
            12.0,
            Color::new(1.0, 0.8, 0.2, appear),
            Align::Center,
            true,
        );
        for (i, (label, a, b, color)) in rows.iter().enumerate() {
            let row_y = y + 30.0 + i as f32 * ROW_H;
            self.ui.text(label, IMPACT_X, row_y + 9.0, 11.0, Color::new(1.0, 1.0, 1.0, appear), Align::Center, true);
            let lead = |mine: f32, other: f32| if mine >= other { 1.0 } else { 0.45 };
            let near = IMPACT_X - TRACK_GAP;
            self.ui.rect(near - TRACK_W, row_y + 2.0, TRACK_W, 6.0, Color::new(0.0, 0.0, 0.0, 0.5 * appear));
            self.ui.rect(near - TRACK_W * a * grow, row_y + 2.0, TRACK_W * a * grow, 6.0, Color::new(color.r, color.g, color.b, lead(*a, *b) * appear));
            let far = IMPACT_X + TRACK_GAP;
            self.ui.rect(far, row_y + 2.0, TRACK_W, 6.0, Color::new(0.0, 0.0, 0.0, 0.5 * appear));
            self.ui.rect(far, row_y + 2.0, TRACK_W * b * grow, 6.0, Color::new(color.r, color.g, color.b, lead(*b, *a) * appear));
        }
    }

    fn draw_impact_ring(&self, time: f32) {
        let progress = (time - 0.7) / 0.8;
        if !(0.0..1.0).contains(&progress) {
            return;
        }
        let radius = (100.0 + 380.0 * settle(progress)) * self.ui.s;
        draw_circle_lines(
            self.ui.x(IMPACT_X),
            self.ui.y(IMPACT_Y),
            radius,
            4.0 * self.ui.s,
            Color::new(1.0, 0.8, 0.2, 0.7 * (1.0 - progress)),
        );
    }
}
