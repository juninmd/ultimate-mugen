use crate::gfx::*;
use crate::poly;
use crate::ui::{alpha, hex, Align, Ui};

impl Ui {
    pub fn adapted_meter(&self, x: f32, y: f32, w: f32, frac: f32) {
        let h = 14.0;
        let track = self.pts(poly::chamfer(x, y, w, h, h * 0.45));
        poly::grad_v(
            &track,
            Color::new(0.02, 0.0, 0.05, 0.85),
            Color::new(0.16, 0.08, 0.22, 0.75),
        );
        let fw = w * frac.clamp(0.0, 1.0);
        if fw > 1.0 {
            let fill = self.pts(poly::chamfer(x, y, fw, h, h * 0.45));
            poly::grad_h(&fill, hex("#22e8ff"), hex("#7dffb0"));
            poly::grad_v(
                &self.pts(poly::chamfer(
                    x + 3.0,
                    y + 1.0,
                    (fw - 6.0).max(0.0),
                    h * 0.42,
                    3.0,
                )),
                alpha(WHITE, 0.4),
                alpha(WHITE, 0.0),
            );
        }
        poly::outline(&track, 0.8 * self.s, alpha(WHITE, 0.5));
    }

    pub fn adapted_glass(&self, x: f32, y: f32, w: f32, h: f32, accent: Color, a: f32) {
        let cut = (h * 0.22).min(16.0);
        let body = self.pts(poly::chamfer(x, y, w, h, cut));
        poly::grad_v(
            &body,
            Color::new(0.1, 0.07, 0.2, 0.7 * a),
            Color::new(0.03, 0.02, 0.08, 0.8 * a),
        );
        poly::grad_v(
            &self.pts(poly::chamfer(
                x + 2.0,
                y + 2.0,
                w - 4.0,
                h * 0.45,
                cut * 0.8,
            )),
            alpha(WHITE, 0.1 * a),
            alpha(WHITE, 0.0),
        );
        let lip = self.pts(vec![
            vec2(x + cut, y + h - 2.5),
            vec2(x + w - cut, y + h - 2.5),
            vec2(x + w - cut, y + h),
            vec2(x + cut, y + h),
        ]);
        poly::grad_h(&lip, alpha(accent, 0.0), alpha(accent, a));
        poly::outline(&body, 0.9 * self.s, alpha(WHITE, 0.2 * a));
    }

    pub fn adapted_title(&self, x: f32, y: f32, title: &str) {
        let bar = self.pts(poly::slanted(x, y - 30.0, 6.0, 38.0, 4.0));
        poly::grad_v(
            &bar,
            crate::tweaks::accent(1.0),
            crate::tweaks::accent_deep(1.0),
        );
        self.text(title, x + 18.0, y, 34.0, hex("#ffcc33"), Align::Left, true);
        let w = self.measure(title, 34.0, true) + 60.0;
        let line = self.pts(vec![
            vec2(x, y + 12.0),
            vec2(x + w, y + 12.0),
            vec2(x + w, y + 14.0),
            vec2(x, y + 14.0),
        ]);
        poly::grad_h(
            &line,
            crate::tweaks::accent(0.9),
            crate::tweaks::accent(0.0),
        );
    }
}
