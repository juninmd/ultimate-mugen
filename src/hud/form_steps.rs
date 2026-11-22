use crate::gfx::*;
use crate::poly;
use crate::ui::Align;
use ume_engine::fighter::Fighter;

const SQUARE: f32 = 4.2;
const GAP: f32 = 1.6;
const LABEL_W: f32 = 6.5;
const TEXT_SIZE: f32 = 3.2;

fn bars(cost: f32) -> String {
    let value = cost / 1000.0;
    if (value - value.round()).abs() < 0.05 {
        format!("{}", value.round() as u32)
    } else {
        format!("{value:.1}")
    }
}

pub(super) struct Row<'a, P: Fn(f32, f32) -> Vec2, T: Fn(&str, f32, f32, f32, Color, Align, bool)> {
    pub p: &'a P,
    pub text: &'a T,
    pub s: f32,
    pub gt: f32,
    pub side: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub colors: [Color; 4],
}

pub(super) fn draw<P: Fn(f32, f32) -> Vec2, T: Fn(&str, f32, f32, f32, Color, Align, bool)>(
    row: &Row<P, T>,
    f: &Fighter,
) {
    let steps = f.form_count();
    if steps < 2 || !f.alive {
        return;
    }
    let step_w = SQUARE + LABEL_W + GAP;
    let total = steps as f32 * step_w;
    let origin = if row.side == 0 {
        row.x + row.w - total
    } else {
        row.x
    };
    let next = usize::from(f.form);
    let affordable = f.next_form_cost().is_some_and(|cost| f.power >= cost);
    let pulse = 0.55 + 0.45 * (row.gt * 6.0).sin().abs();
    let corners = |x: f32, y: f32, size: f32| -> Vec<Vec2> {
        poly::chamfer(x, y, size, size, 0.9)
            .into_iter()
            .map(|q| (row.p)(q.x, q.y))
            .collect()
    };
    for index in 0..steps {
        let x = origin + index as f32 * step_w;
        let color = row.colors[index.min(3)];
        let reached = index < next;
        let current = index == next;
        let cost = f.base_def.forms.get(index).map_or(1000.0, |form| form.cost);
        poly::solid(
            &corners(x - 0.5, row.y - 0.5, SQUARE + 1.0),
            Color::new(0.0, 0.0, 0.03, 0.82),
        );
        if reached {
            poly::solid(&corners(x, row.y, SQUARE), color);
        } else if current && affordable {
            poly::solid(
                &corners(x, row.y, SQUARE),
                poly::with_a(color, 0.35 + 0.5 * pulse),
            );
            poly::outline(
                &corners(x - 0.5, row.y - 0.5, SQUARE + 1.0),
                0.5 * row.s,
                poly::with_a(color, pulse),
            );
        } else if current {
            let fill = (f.power / cost).clamp(0.0, 1.0);
            let top = row.y + SQUARE - SQUARE * fill;
            let band = [
                (row.p)(x, top),
                (row.p)(x + SQUARE, top),
                (row.p)(x + SQUARE, row.y + SQUARE),
                (row.p)(x, row.y + SQUARE),
            ];
            poly::solid(&band, poly::with_a(color, 0.7));
            poly::outline(
                &corners(x - 0.5, row.y - 0.5, SQUARE + 1.0),
                0.4 * row.s,
                poly::with_a(color, 0.6),
            );
        } else {
            poly::outline(
                &corners(x - 0.5, row.y - 0.5, SQUARE + 1.0),
                0.4 * row.s,
                Color::new(1.0, 1.0, 1.0, 0.25),
            );
        }
        let label_color = if reached {
            poly::with_a(color, 0.9)
        } else {
            Color::new(1.0, 1.0, 1.0, if current { 0.9 } else { 0.5 })
        };
        let label = if reached {
            "OK".to_string()
        } else {
            format!("{}B", bars(cost))
        };
        (row.text)(
            &label,
            x + SQUARE + 0.9,
            row.y + SQUARE - 0.7,
            TEXT_SIZE,
            label_color,
            Align::Left,
            true,
        );
    }
}
