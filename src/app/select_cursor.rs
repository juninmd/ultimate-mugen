use super::*;

const HALO_LAYERS: usize = 4;

impl App {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_select_cursor(
        &self,
        rx: f32,
        ry: f32,
        cw: f32,
        ch: f32,
        a: Color,
        pulse: f32,
        side: usize,
    ) {
        let base = 2.0 + side as f32 * 3.0;
        for k in (1..=HALO_LAYERS).rev() {
            let pad = base + k as f32 * (2.5 + 1.5 * pulse);
            let fall = 1.0 - k as f32 / (HALO_LAYERS as f32 + 1.0);
            self.ui.rect(
                rx - pad,
                ry - pad,
                cw + pad * 2.0,
                ch + pad * 2.0,
                alpha(a, 0.07 * fall * (0.6 + 0.4 * pulse)),
            );
        }
        let pad = base + 1.0 * pulse;
        let (x0, y0, x1, y1) = (
            self.ui.x(rx),
            self.ui.y(ry),
            self.ui.x(rx + cw),
            self.ui.y(ry + ch * 0.45),
        );
        crate::poly::grad_v(
            &[vec2(x0, y0), vec2(x1, y0), vec2(x1, y1), vec2(x0, y1)],
            alpha(WHITE, 0.16 + 0.08 * pulse),
            alpha(WHITE, 0.0),
        );
        self.ui.rect_lines(
            rx - pad,
            ry - pad,
            cw + pad * 2.0,
            ch + pad * 2.0,
            2.0,
            alpha(a, 0.95),
        );
        self.ui.rect_lines(
            rx - pad - 3.0,
            ry - pad - 3.0,
            cw + pad * 2.0 + 6.0,
            ch + pad * 2.0 + 6.0,
            1.0,
            alpha(a, 0.35 * pulse),
        );
        let (bl, bt) = (16.0, 4.0);
        for (ox, oy, dx, dy) in [
            (rx - pad, ry - pad, 1.0, 1.0),
            (rx + cw + pad, ry - pad, -1.0, 1.0),
            (rx - pad, ry + ch + pad, 1.0, -1.0),
            (rx + cw + pad, ry + ch + pad, -1.0, -1.0),
        ] {
            self.ui.rect(
                ox.min(ox + dx * bl),
                oy + if dy > 0.0 { -2.0 } else { 2.0 - bt },
                bl,
                bt,
                WHITE,
            );
            self.ui.rect(
                ox + if dx > 0.0 { -2.0 } else { 2.0 - bt },
                oy.min(oy + dy * bl),
                bt,
                bl,
                WHITE,
            );
        }
    }
}
