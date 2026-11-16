use crate::gfx::*;
use crate::poly;
use crate::ui::{hex, Align, Ui};

const H: f32 = 14.0;
const SPIN_R: f32 = 11.0;

impl Ui {
    pub fn progress_bar(&self, x: f32, y: f32, w: f32, p: f32, label: &str) {
        let p = p.clamp(0.0, 1.0);
        let pts = |x0: f32, ww: f32, y0: f32, hh: f32| -> Vec<Vec2> {
            poly::chamfer(x0, y0, ww, hh, hh * 0.45)
                .into_iter()
                .map(|q| vec2(self.x(q.x), self.y(q.y)))
                .collect()
        };
        poly::solid(
            &pts(x - 2.0, w + 4.0, y - 2.0, H + 4.0),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );
        poly::grad_v(
            &pts(x, w, y, H),
            Color::new(0.02, 0.0, 0.05, 0.85),
            Color::new(0.16, 0.08, 0.22, 0.75),
        );
        let fw = w * p;
        if fw > 1.0 {
            poly::grad_h(
                &pts(x, fw, y, H),
                hex("#ff4d6d"),
                crate::poly::lerp_c(hex("#ff4d6d"), hex("#ffcc33"), p),
            );
            poly::grad_v(
                &pts(x + 3.0, (fw - 6.0).max(0.0), y + 1.0, H * 0.45),
                Color::new(1.0, 1.0, 1.0, 0.45),
                Color::new(1.0, 1.0, 1.0, 0.0),
            );
            let phase = (self.t * 0.6) % 1.3 - 0.15;
            let cx = x + fw * phase;
            let (a, z) = ((cx - 30.0).max(x), (cx + 30.0).min(x + fw));
            if z > a {
                let m = cx.clamp(a, z);
                let quad = |l: f32, r: f32| {
                    vec![
                        vec2(self.x(l), self.y(y)),
                        vec2(self.x(r), self.y(y)),
                        vec2(self.x(r), self.y(y + H)),
                        vec2(self.x(l), self.y(y + H)),
                    ]
                };
                poly::grad_h(
                    &quad(a, m),
                    Color::new(1.0, 1.0, 1.0, 0.0),
                    Color::new(1.0, 1.0, 1.0, 0.35),
                );
                poly::grad_h(
                    &quad(m, z),
                    Color::new(1.0, 1.0, 1.0, 0.35),
                    Color::new(1.0, 1.0, 1.0, 0.0),
                );
            }
            let ex = x + fw;
            poly::circle(
                vec2(self.x(ex), self.y(y + H / 2.0)),
                5.0 * self.s,
                Color::new(1.0, 0.95, 0.8, 0.35),
            );
        }
        poly::outline(
            &pts(x, w, y, H),
            (1.2 * self.s).max(1.0),
            Color::new(1.0, 1.0, 1.0, 0.55),
        );
        self.spinner(x - 30.0, y + H / 2.0);
        self.text(label, x, y + 32.0, 15.0, hex("#d8d0f0"), Align::Left, false);
        self.text(
            &format!("{}%", (p * 100.0) as i32),
            x + w,
            y + 32.0,
            15.0,
            hex("#ffcc33"),
            Align::Right,
            true,
        );
    }

    fn spinner(&self, cx: f32, cy: f32) {
        let c = vec2(self.x(cx), self.y(cy));
        let r = SPIN_R * self.s;
        poly::ring(c, r, 2.4 * self.s, Color::new(1.0, 1.0, 1.0, 0.12));
        let start = self.t * 5.0;
        let steps = 18;
        for k in 0..steps {
            let t0 = start + k as f32 / steps as f32 * 4.2;
            let t1 = start + (k + 1) as f32 / steps as f32 * 4.2;
            let fade = k as f32 / steps as f32;
            let (u0, u1) = (vec2(t0.cos(), t0.sin()), vec2(t1.cos(), t1.sin()));
            let (ro, ri) = (r + 1.4 * self.s, r - 1.4 * self.s);
            poly::solid(
                &[c + u0 * ro, c + u1 * ro, c + u1 * ri, c + u0 * ri],
                Color::new(1.0, 0.8, 0.2, fade),
            );
        }
    }
}
