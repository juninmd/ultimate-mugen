use super::{design, height_for, Shape};
use crate::gfx::*;
use crate::ui::Ui;
use std::f32::consts::{PI, TAU};

const ARC_POINTS: usize = 15;
const STAR_INNER_RATIO: f32 = 0.42;

struct Frame {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    alpha: f32,
}

impl Frame {
    fn at(&self, x: f32, y: f32) -> Vec2 {
        vec2(self.x + x * self.w, self.y + y * self.h)
    }

    fn ink(&self, color: u32) -> Color {
        Color::new(
            ((color >> 16) & 0xff) as f32 / 255.0,
            ((color >> 8) & 0xff) as f32 / 255.0,
            (color & 0xff) as f32 / 255.0,
            self.alpha,
        )
    }

    fn star(&self, center: Vec2, radius: f32, points: u8, color: Color) {
        let steps = points as usize * 2;
        let corner = |n: usize| {
            let reach = if n % 2 == 0 {
                radius
            } else {
                radius * STAR_INNER_RATIO
            };
            let angle = n as f32 / steps as f32 * TAU - PI / 2.0;
            center + vec2(angle.cos(), angle.sin()) * reach
        };
        for n in 0..steps {
            draw_triangle(center, corner(n), corner(n + 1), color);
        }
    }

    fn half(&self, center: Vec2, radius: f32, top: bool, color: Color) {
        let mut arc = [Vec2::ZERO; ARC_POINTS];
        let turn = if top { PI } else { 0.0 };
        for (n, slot) in arc.iter_mut().enumerate() {
            let angle = turn + n as f32 / (ARC_POINTS - 1) as f32 * PI;
            *slot = center + vec2(angle.cos(), angle.sin()) * radius;
        }
        draw_convex(&arc, color);
    }

    fn shape(&self, shape: &Shape) {
        match *shape {
            Shape::Rect { x, y, w, h, color } => draw_rectangle(
                self.x + x * self.w,
                self.y + y * self.h,
                w * self.w,
                h * self.h,
                self.ink(color),
            ),
            Shape::Disc { x, y, r, color } => {
                let center = self.at(x, y);
                draw_circle(center.x, center.y, r * self.h, self.ink(color))
            }
            Shape::Half {
                x,
                y,
                r,
                top,
                color,
            } => self.half(self.at(x, y), r * self.h, top, self.ink(color)),
            Shape::Star {
                x,
                y,
                r,
                points,
                color,
            } => self.star(self.at(x, y), r * self.h, points, self.ink(color)),
            Shape::Poly { points, color } => {
                let mut corners = [Vec2::ZERO; 8];
                let count = points.len().min(corners.len());
                for (slot, p) in corners.iter_mut().zip(points) {
                    *slot = self.at(p[0], p[1]);
                }
                draw_convex(&corners[..count], self.ink(color))
            }
        }
    }
}

impl Ui {
    pub(crate) fn flag(&self, code: &str, x: f32, y: f32, width: f32, alpha: f32) -> bool {
        let Some(shapes) = design(code) else {
            return false;
        };
        let height = height_for(width);
        self.rect(
            x - 1.0,
            y - 1.0,
            width + 2.0,
            height + 2.0,
            Color::new(0.0, 0.0, 0.0, 0.55 * alpha),
        );
        let frame = Frame {
            x: self.x(x),
            y: self.y(y),
            w: width * self.s,
            h: height * self.s,
            alpha,
        };
        for shape in shapes {
            frame.shape(shape);
        }
        self.rect_lines(
            x,
            y,
            width,
            height,
            1.0,
            Color::new(1.0, 1.0, 1.0, 0.28 * alpha),
        );
        true
    }
}
