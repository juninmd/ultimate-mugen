use super::*;

const HALO_LAYERS: usize = 4;

impl App {
    pub(super) fn draw_stage_cursor(&self, x: f32, y: f32) {
        let gx = crate::spring::follow("stage.cur.x", x);
        let gy = crate::spring::follow("stage.cur.y", y);
        let pulse = 0.5 + 0.5 * (self.ui.t * 6.0).sin();
        let gold = hex("#ffcc33");
        for k in (1..=HALO_LAYERS).rev() {
            let pad = 3.0 + k as f32 * (3.0 + 2.0 * pulse);
            let fall = 1.0 - k as f32 / (HALO_LAYERS as f32 + 1.0);
            self.ui.rect(
                gx - pad,
                gy - pad,
                TILE_W + pad * 2.0,
                TILE_H + pad * 2.0,
                alpha(gold, 0.06 * fall * (0.6 + 0.4 * pulse)),
            );
        }
        self.ui
            .rect_lines(gx - 3.0, gy - 3.0, TILE_W + 6.0, TILE_H + 6.0, 3.0, gold);
        self.ui.rect_lines(
            gx - 7.0,
            gy - 7.0,
            TILE_W + 14.0,
            TILE_H + 14.0,
            1.0,
            alpha(gold, 0.4 * pulse),
        );
        self.draw_cursor_corners(gx, gy, TILE_W, TILE_H, 3.0 + pulse, 28.0, gold);
    }
}

pub(super) fn pan(focused: bool, t: f32) -> f32 {
    if focused {
        0.5 + 0.5 * (t * 0.4).sin()
    } else {
        0.5
    }
}
