use super::*;

impl App {
    pub(super) fn draw_roster_panel(&self, x: f32, y: f32, w: f32, h: f32, picking: usize) {
        let edge = if picking == 0 {
            hex("#57d4ff")
        } else {
            hex("#ff7794")
        };
        let (left, top, width, height) = (x - 12.0, y - 12.0, w + 24.0, h + 24.0);
        self.ui.rect(
            left,
            top,
            width,
            height,
            Color::new(0.04, 0.035, 0.09, 0.93),
        );
        self.ui
            .rect_lines(left, top, width, height, 1.0, alpha(edge, 0.35));
        self.ui.rect(left, top, width, 2.0, edge);
    }

    pub(super) fn draw_roster_tile(&self, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        self.ui.rect(x, y, w, h, hex("#151022"));
        self.ui
            .gradient_v(x, y, w, h, alpha(tint, 0.44), alpha(tint, 0.08));
        self.ui.rect_lines(x, y, w, h, 1.0, alpha(WHITE, 0.12));
    }
}
