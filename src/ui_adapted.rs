use crate::gfx::*;
use crate::poly;
use crate::ui::{alpha, hex, Align, Ui};

impl Ui {
    pub(crate) fn pts(&self, v: Vec<Vec2>) -> Vec<Vec2> {
        v.into_iter()
            .map(|q| vec2(self.x(q.x), self.y(q.y)))
            .collect()
    }

    pub fn adapted_header(&self, title: &str, sub: &str, w: f32) {
        self.rect(0.0, 0.0, w, 118.0, Color::new(0.0, 0.0, 0.0, 0.35));
        let bar = self.pts(poly::slanted(56.0, 30.0, 6.0, 44.0, 4.0));
        poly::grad_v(
            &bar,
            crate::tweaks::accent(1.0),
            crate::tweaks::accent_deep(1.0),
        );
        self.text(title, 74.0, 66.0, 38.0, hex("#ffcc33"), Align::Left, true);
        self.text(
            sub,
            w - 56.0,
            58.0,
            20.0,
            hex("#a49cc0"),
            Align::Right,
            false,
        );
        let line = self.pts(vec![
            vec2(56.0, 116.0),
            vec2(w * 0.5, 116.0),
            vec2(w * 0.5, 118.0),
            vec2(56.0, 118.0),
        ]);
        poly::grad_h(
            &line,
            crate::tweaks::accent(0.9),
            crate::tweaks::accent(0.0),
        );
    }

    pub fn adapted_highlight(
        &self,
        key: (&'static str, &'static str),
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let (x, y) = (
            crate::spring::follow(key.0, x),
            crate::spring::follow(key.1, y),
        );
        let pulse = 0.75 + 0.25 * (self.t * 4.0).sin();
        let cut = h * 0.3;
        poly::grad_v(
            &self.pts(poly::chamfer(x - 4.0, y - 4.0, w + 8.0, h + 8.0, cut + 3.0)),
            crate::tweaks::accent(0.22 * pulse),
            crate::tweaks::accent(0.06),
        );
        let main = self.pts(poly::chamfer(x, y, w, h, cut));
        poly::grad_v(
            &main,
            crate::tweaks::accent(0.9),
            crate::tweaks::accent_deep(0.75),
        );
        poly::grad_v(
            &self.pts(poly::chamfer(
                x + 6.0,
                y + 1.0,
                w - 12.0,
                h * 0.42,
                cut * 0.5,
            )),
            alpha(WHITE, 0.22),
            alpha(WHITE, 0.0),
        );
        poly::outline(&main, 1.2 * self.s, Color::new(1.0, 0.85, 0.9, 0.7));
    }

    pub fn adapted_row(&self, x: f32, y: f32, w: f32, h: f32, interactive: bool) {
        let a = if interactive { 1.0 } else { 0.45 };
        let body = self.pts(poly::chamfer(x, y, w, h, h * 0.3));
        poly::grad_v(
            &body,
            Color::new(1.0, 1.0, 1.0, 0.08 * a),
            Color::new(1.0, 1.0, 1.0, 0.025 * a),
        );
        poly::outline(&body, 0.8 * self.s, Color::new(1.0, 1.0, 1.0, 0.1 * a));
    }

    pub fn adapted_tab(&self, x: f32, y: f32, w: f32, h: f32, active: bool) {
        let shape = self.pts(poly::slanted(x, y, w, h, 10.0));
        if active {
            poly::grad_v(
                &shape,
                crate::tweaks::accent(0.55),
                crate::tweaks::accent_deep(0.35),
            );
            let lip = self.pts(poly::slanted(x + 4.0, y + h - 3.0, w - 8.0, 3.0, 0.5));
            poly::solid(&lip, crate::tweaks::accent(1.0));
        } else {
            poly::grad_v(&shape, alpha(WHITE, 0.08), alpha(WHITE, 0.03));
        }
    }

    pub fn adapted_plate(&self, x: f32, y: f32, w: f32, h: f32, tint: Option<Color>) {
        let shape = self.pts(poly::chamfer(x, y, w, h, h * 0.35));
        match tint {
            Some(c) => poly::grad_v(&shape, c, poly::shade(c, 0.6)),
            None => poly::grad_v(&shape, alpha(WHITE, 0.14), alpha(WHITE, 0.05)),
        }
        poly::outline(&shape, 0.8 * self.s, alpha(WHITE, 0.25));
    }

    pub fn adapted_button(&self, x: f32, y: f32, w: f32, h: f32, label: &str, hot: bool) {
        self.adapted_plate(x, y, w, h, hot.then(|| crate::tweaks::accent(0.9)));
        self.text(
            label,
            x + w / 2.0,
            y + h * 0.7,
            20.0,
            WHITE,
            Align::Center,
            true,
        );
    }

    pub fn adapted_modal(&self, x: f32, y: f32, w: f32, h: f32, vw: f32, vh: f32) {
        self.rect(0.0, 0.0, vw, vh, Color::new(0.0, 0.0, 0.0, 0.7));
        let shape = self.pts(poly::chamfer(x, y, w, h, 18.0));
        poly::grad_v(&shape, hex("#2a1846"), hex("#120a24"));
        poly::grad_v(
            &self.pts(poly::chamfer(x + 2.0, y + 2.0, w - 4.0, 60.0, 16.0)),
            alpha(WHITE, 0.08),
            alpha(WHITE, 0.0),
        );
        poly::outline(&shape, 1.6 * self.s, crate::tweaks::accent(0.85));
    }

    pub fn adapted_field(&self, x: f32, y: f32, w: f32, h: f32) {
        let shape = self.pts(poly::chamfer(x, y, w, h, 8.0));
        poly::grad_v(
            &shape,
            Color::new(0.0, 0.0, 0.0, 0.45),
            Color::new(0.1, 0.05, 0.16, 0.5),
        );
        poly::outline(&shape, 1.0 * self.s, alpha(WHITE, 0.3));
    }

    pub fn adapted_level(&self, x: f32, y: f32, steps: usize, on: usize) {
        for n in 0..steps {
            let cell = self.pts(poly::slanted(x + n as f32 * 22.0, y, 17.0, 20.0, 3.0));
            if n < on {
                poly::grad_v(&cell, hex("#ffe27a"), hex("#ff9f1a"));
            } else {
                poly::solid(&cell, alpha(WHITE, 0.12));
            }
        }
    }
}
