use super::canvas::Drawn;
use crate::gfx::*;
use crate::render::Gfx;
use ume_engine::fighter::Fighter;

const GUARD_STATES: std::ops::RangeInclusive<i32> = 120..=155;
const GUARD_HIT_STATES: std::ops::RangeInclusive<i32> = 150..=155;
const GUARD_FLASH_TICKS: i32 = 10;
const SHIELD_SEGMENTS: usize = 18;

fn reiatsu(awakened: bool) -> [f32; 3] {
    if awakened {
        [1.0, 0.72, 0.08]
    } else {
        [1.0, 0.12, 0.22]
    }
}

pub(super) fn trail(g: &Gfx, f: &Fighter, drawn: &Drawn, awakened: bool) {
    let Some(ai) = f.after_image.as_ref().filter(|_| crate::lite::afterimage()) else {
        return;
    };
    let shown = ai.params.shown(ai.trail.len(), ai.length);
    let [r, gr, b] = reiatsu(awakened);
    set_blend(GfxBlend::Add);
    for (k, &i) in shown.iter().enumerate().rev() {
        let t = &ai.trail[i];
        if t.facing != f.facing {
            continue;
        }
        let fade = 0.55 * (1.0 - k as f32 / shown.len().max(1) as f32);
        let offset = [(t.x - f.pos.x) * g.view_scale, (t.y - f.pos.y) * g.view_scale];
        drawn.draw(offset, Color::new(r, gr, b, fade));
    }
    set_blend(GfxBlend::Alpha);
}

pub(super) fn guard(f: &Fighter, feet: [f32; 2], height: f32, awakened: bool) {
    if !GUARD_STATES.contains(&f.state_no) {
        return;
    }
    let flash = if GUARD_HIT_STATES.contains(&f.state_no) {
        (1.0 - f.state_time as f32 / GUARD_FLASH_TICKS as f32).max(0.0)
    } else {
        0.0
    };
    let [r, g, b] = reiatsu(awakened);
    let center = [feet[0] + f.facing * height * 0.3, feet[1] - height * 0.55];
    let (rx, ry) = (height * (0.1 + 0.03 * flash), height * (0.46 + 0.05 * flash));
    let pulse = 0.5 + 0.5 * ((f.state_time as f32) * 0.35).sin();
    set_blend(GfxBlend::Add);
    draw_ellipse(center[0], center[1], rx, ry, 0.0, Color::new(r, g, b, 0.10 + 0.25 * flash));
    for ring in 0..3 {
        let k = 1.0 - ring as f32 * 0.22;
        let alpha = (0.35 + 0.15 * pulse) * k + 0.5 * flash;
        outline(center, rx * k, ry * k, 1.5 + flash * 2.0, Color::new(r, g, b, alpha));
    }
    if flash > 0.0 {
        for spoke in 0..8 {
            let a = spoke as f32 * std::f32::consts::TAU / 8.0;
            let reach = height * (0.12 + 0.25 * (1.0 - flash));
            let from = [center[0] + a.cos() * rx * 0.5, center[1] + a.sin() * ry * 0.3];
            let to = [from[0] + a.cos() * reach * 0.6 * f.facing, from[1] + a.sin() * reach];
            draw_line(from[0], from[1], to[0], to[1], 2.0, Color::new(1.0, 1.0, 1.0, 0.8 * flash));
        }
    }
    set_blend(GfxBlend::Alpha);
}

fn outline(center: [f32; 2], rx: f32, ry: f32, thickness: f32, color: Color) {
    let point = |i: usize| {
        let a = i as f32 * std::f32::consts::TAU / SHIELD_SEGMENTS as f32;
        [center[0] + a.cos() * rx, center[1] + a.sin() * ry]
    };
    for i in 0..SHIELD_SEGMENTS {
        let (p, q) = (point(i), point(i + 1));
        draw_line(p[0], p[1], q[0], q[1], thickness, color);
    }
}
