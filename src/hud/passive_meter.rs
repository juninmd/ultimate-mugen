use crate::gfx::*;
use crate::poly;
use ume_engine::fighter::Fighter;

const STRIP_HEIGHT: f32 = 1.5;
const STRIP_LIFT: f32 = 0.6;
const PIP_WIDTH: f32 = 3.4;
const PIP_GAP: f32 = 0.9;
const PIP_SPAN_SHARE: f32 = 0.55;
const RAGE_LOW: [f32; 3] = [1.0, 0.2, 0.12];
const RAGE_HIGH: [f32; 3] = [1.0, 0.82, 0.3];

fn rect<P: Fn(f32, f32) -> Vec2>(p: &P, s: f32, x: f32, y: f32, w: f32, h: f32) -> [Vec2; 4] {
    let a = p(x, y);
    [
        a,
        a + vec2(w * s, 0.0),
        a + vec2(w * s, h * s),
        a + vec2(0.0, h * s),
    ]
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> Color {
    Color::new(
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        1.0,
    )
}

pub(super) struct Bar {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub side: usize,
}

pub(super) fn draw<P: Fn(f32, f32) -> Vec2>(p: &P, s: f32, gt: f32, f: &Fighter, bar: &Bar) {
    let Some(view) = f.passive_view() else { return };
    if !f.alive {
        return;
    }
    let y = bar.y + bar.h - STRIP_HEIGHT - STRIP_LIFT;
    let inset = bar.h * 0.45;
    let (span_x, span_w) = (bar.x + inset, bar.w - inset * 2.0);
    if let Some(progress) = view.rage {
        poly::solid(
            &rect(p, s, span_x, y, span_w, STRIP_HEIGHT),
            Color::new(0.0, 0.0, 0.0, 0.35),
        );
        let filled = span_w * progress;
        if filled > 0.2 {
            let x = if bar.side == 0 {
                span_x
            } else {
                span_x + span_w - filled
            };
            let pulse = 0.8 + 0.2 * (gt * (4.0 + progress * 8.0)).sin();
            let hot = mix(RAGE_LOW, RAGE_HIGH, progress);
            poly::solid(
                &rect(p, s, x, y, filled, STRIP_HEIGHT),
                poly::with_a(hot, pulse),
            );
        }
    }
    if view.max_stacks > 0 {
        let pitch = (PIP_WIDTH + PIP_GAP).min(span_w * PIP_SPAN_SHARE / view.max_stacks as f32);
        let width = pitch - PIP_GAP.min(pitch * 0.3);
        let lit = Color::from_rgba(
            f.def.color[0].max(140),
            f.def.color[1].max(140),
            f.def.color[2].max(140),
            255,
        );
        let backing_x = if bar.side == 0 {
            span_x - 0.6
        } else {
            span_x + span_w - pitch * view.max_stacks as f32 - 0.6 + PIP_GAP
        };
        poly::solid(
            &rect(
                p,
                s,
                backing_x,
                y - 0.6,
                pitch * view.max_stacks as f32 + 0.6,
                STRIP_HEIGHT * 1.15 + 1.2,
            ),
            Color::new(0.0, 0.0, 0.05, 0.45),
        );
        for index in 0..view.max_stacks {
            let x = if bar.side == 0 {
                span_x + pitch * index as f32
            } else {
                span_x + span_w - pitch * (index as f32 + 1.0) + (pitch - width)
            };
            let on = index < view.stacks;
            let flash = if on && index + 1 == view.stacks {
                0.75 + 0.25 * (gt * 9.0).sin()
            } else {
                1.0
            };
            let color = if on {
                poly::with_a(lit, flash)
            } else {
                Color::new(0.0, 0.0, 0.0, 0.4)
            };
            poly::solid(&rect(p, s, x, y, width, STRIP_HEIGHT * 1.15), color);
        }
    }
}
