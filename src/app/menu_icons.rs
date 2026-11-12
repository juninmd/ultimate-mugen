use super::*;

const SIZE: f32 = 13.0;

pub(super) const ICON_OF: [usize; 10] = [0, 1, 2, 4, 5, 10, 11, 13, 14, 15];

struct Pen<'a> {
    ui: &'a Ui,
    cx: f32,
    cy: f32,
    color: Color,
}

impl Pen<'_> {
    fn line(&self, a: (f32, f32), b: (f32, f32)) {
        draw_line(
            self.ui.x(self.cx + a.0 * SIZE),
            self.ui.y(self.cy + a.1 * SIZE),
            self.ui.x(self.cx + b.0 * SIZE),
            self.ui.y(self.cy + b.1 * SIZE),
            2.2 * self.ui.s,
            self.color,
        );
    }

    fn ring(&self, at: (f32, f32), r: f32) {
        draw_circle_lines(self.ui.x(self.cx + at.0 * SIZE), self.ui.y(self.cy + at.1 * SIZE), r * SIZE * self.ui.s, 2.2 * self.ui.s, self.color);
    }

    fn dot(&self, at: (f32, f32), r: f32) {
        draw_circle(self.ui.x(self.cx + at.0 * SIZE), self.ui.y(self.cy + at.1 * SIZE), r * SIZE * self.ui.s, self.color);
    }

    fn poly(&self, pts: &[(f32, f32)]) {
        for i in 0..pts.len() {
            self.line(pts[i], pts[(i + 1) % pts.len()]);
        }
    }
}

impl App {
    pub(super) fn menu_icon(&self, index: usize, cx: f32, cy: f32, color: Color) {
        let p = Pen { ui: &self.ui, cx, cy, color };
        match index {
            0 => {
                p.poly(&[(-0.8, -0.9), (0.8, -0.9), (0.8, 0.9), (-0.8, 0.9)]);
                p.line((-0.4, -0.3), (0.4, -0.3));
                p.line((-0.4, 0.2), (0.4, 0.2));
            }
            1 => {
                p.poly(&[(-0.8, -0.8), (0.8, -0.8), (0.6, 0.2), (0.0, 0.7), (-0.6, 0.2)]);
                p.line((0.0, 0.7), (0.0, 1.0));
                p.line((-0.5, 1.0), (0.5, 1.0));
            }
            2 => {
                p.line((0.0, 0.9), (0.0, -0.9));
                p.line((-0.7, 0.2), (0.0, -0.9));
                p.line((0.7, 0.2), (0.0, -0.9));
                p.line((-0.6, 0.9), (0.6, 0.9));
            }
            3 => {
                p.poly(&[(-0.9, -0.5), (0.9, -0.5), (0.9, 0.7), (-0.9, 0.7)]);
                p.dot((-0.35, 0.1), 0.18);
                p.dot((0.35, 0.1), 0.18);
                p.line((0.0, -0.5), (0.0, -0.95));
            }
            4 => {
                p.line((-0.9, -0.9), (0.9, 0.9));
                p.line((0.9, -0.9), (-0.9, 0.9));
                p.line((-0.9, -0.9), (-0.5, -0.9));
                p.line((0.9, -0.9), (0.5, -0.9));
            }
            5 => {
                p.ring((0.0, 0.0), 0.9);
                p.ring((0.0, 0.0), 0.4);
                p.dot((0.0, 0.0), 0.12);
            }
            6 => {
                p.poly(&[(-0.95, 0.0), (-0.4, -0.55), (0.4, -0.55), (0.95, 0.0), (0.4, 0.55), (-0.4, 0.55)]);
                p.dot((0.0, 0.0), 0.28);
            }
            7 => {
                for i in 0..3 {
                    p.ring((-0.65 + i as f32 * 0.65, 0.0), 0.3);
                }
                p.line((-0.35, 0.0), (-0.3, 0.0));
            }
            8 => {
                p.ring((-0.4, 0.0), 0.5);
                p.ring((0.4, 0.0), 0.5);
            }
            9 => {
                p.line((-0.9, -0.4), (0.7, -0.4));
                p.line((0.4, -0.8), (0.8, -0.4));
                p.line((0.9, 0.4), (-0.7, 0.4));
                p.line((-0.4, 0.8), (-0.8, 0.4));
            }
            10 => {
                p.poly(&[(-0.9, -0.7), (0.9, -0.7), (0.9, 0.7), (-0.9, 0.7)]);
                p.line((-0.9, -0.2), (0.9, -0.2));
                p.dot((-0.45, 0.35), 0.14);
            }
            11 => {
                p.poly(&[(-0.9, -0.3), (0.9, -0.3), (0.7, 0.9), (-0.7, 0.9)]);
                p.line((-0.4, -0.3), (-0.2, -0.9));
                p.line((0.4, -0.3), (0.2, -0.9));
            }
            12 => {
                p.poly(&[(-0.95, -0.4), (0.95, -0.4), (0.95, 0.6), (-0.95, 0.6)]);
                p.line((-0.5, 0.1), (-0.1, 0.1));
                p.line((-0.3, -0.1), (-0.3, 0.3));
                p.dot((0.45, 0.0), 0.14);
            }
            13 => {
                p.ring((0.0, 0.0), 0.5);
                for i in 0..8 {
                    let a = i as f32 * std::f32::consts::FRAC_PI_4;
                    p.line((a.cos() * 0.7, a.sin() * 0.7), (a.cos() * 1.0, a.sin() * 1.0));
                }
            }
            14 => {
                p.ring((0.0, 0.0), 0.9);
                p.poly(&[(-0.2, -0.4), (0.5, 0.0), (-0.2, 0.4)]);
            }
            _ => {
                p.line((0.0, -0.95), (0.0, 0.0));
                p.line((-0.7, -0.6), (-0.7, 0.4));
                p.line((0.7, -0.6), (0.7, 0.4));
                p.line((-0.7, 0.4), (0.0, 0.95));
                p.line((0.7, 0.4), (0.0, 0.95));
            }
        }
    }
}
