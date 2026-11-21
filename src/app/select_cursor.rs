use std::sync::Arc;

use super::*;

const HALO_LAYERS: usize = 4;

thread_local! {
    static CORNER: Option<Texture2D> = load_corner();
}

fn load_corner() -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(include_bytes!("../../assets/ui/select-corner.png")).ok()?;
    let rgba = match image.rgba {
        Some(rgba) => rgba,
        None => ume_formats::png::png_indexed_to_rgba(&image),
    };
    let tex = Texture2D::from_rgba_shared(image.width as u16, image.height as u16, Arc::new(rgba));
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

impl App {
    pub(crate) fn draw_cursor_corners(
        &self,
        rx: f32,
        ry: f32,
        cw: f32,
        ch: f32,
        pad: f32,
        size: f32,
        tint: Color,
    ) -> bool {
        CORNER.with(|corner| {
            let Some(tex) = corner else {
                return false;
            };
            let sz = size * self.ui.s;
            let col = Color::new(
                0.7 + tint.r * 0.3,
                0.7 + tint.g * 0.3,
                0.7 + tint.b * 0.3,
                tint.a,
            );
            let x0 = self.ui.x(rx - pad - 4.0);
            let y0 = self.ui.y(ry - pad - 4.0);
            let x1 = self.ui.x(rx + cw + pad + 4.0) - sz;
            let y1 = self.ui.y(ry + ch + pad + 4.0) - sz;
            for (x, y, flip_x, flip_y) in [
                (x0, y0, false, false),
                (x1, y0, true, false),
                (x0, y1, false, true),
                (x1, y1, true, true),
            ] {
                draw_texture_ex(
                    tex,
                    x,
                    y,
                    col,
                    DrawTextureParams {
                        dest_size: Some(vec2(sz, sz)),
                        flip_x,
                        flip_y,
                        ..Default::default()
                    },
                );
            }
            true
        })
    }

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
        if self.draw_cursor_corners(rx, ry, cw, ch, pad, 24.0, alpha(a, 1.0)) {
            return;
        }
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
