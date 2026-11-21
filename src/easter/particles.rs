use super::{ActiveEgg, EggSprite};
use crate::gfx::*;
use crate::ui::Ui;

pub(crate) fn confetti_burst(ui: &Ui, cx: f32, cy: f32, t: f32, alpha: f32, base: [f32; 3]) {
    let palette = [
        [1.0, 0.86, 0.24],
        [1.0, 0.36, 0.52],
        [0.32, 0.92, 1.0],
        base,
    ];
    for i in 0..14 {
        let seed = i as f32 * 2.39996;
        let angle = seed + (t * 0.4);
        let speed = 55.0 + (i % 4) as f32 * 26.0;
        let life = (t * 1.15).min(1.6);
        let px = cx + angle.cos() * speed * life;
        let py = cy + angle.sin() * speed * life * 0.65 + life * life * 48.0;
        let fade = (1.0 - (t / 1.8).clamp(0.0, 1.0)) * alpha;
        if fade <= 0.02 {
            continue;
        }
        let c = palette[i % palette.len()];
        let sz = if i % 3 == 0 { 6.0 } else { 4.5 };
        ui.rect(px - sz * 0.5, py - sz * 0.5, sz, sz, Color::new(c[0], c[1], c[2], fade));
    }
}

pub(crate) fn flyby_trail(ui: &Ui, vx: f32, vy: f32, egg: &ActiveEgg, alpha: f32) {
    let dir = if egg.flip { 1.0 } else { -1.0 };
    for i in 1..=7 {
        let lag = i as f32;
        let tx = vx + dir * lag * 16.0;
        let ty = vy + ((egg.t * 9.0 + lag * 0.9).sin()) * (5.0 + lag * 1.4);
        let fade = (1.0 - lag / 8.5) * 0.78 * alpha;
        let sz = (8.5 - lag * 0.9).max(2.5);
        let col = match egg.sprite {
            EggSprite::UfoNeon => {
                if i % 2 == 0 { [0.28, 0.96, 1.0] } else { [1.0, 0.26, 0.84] }
            }
            EggSprite::ArcadeCoin => [1.0, 0.88, 0.34],
            EggSprite::KitsuneMask => [1.0, 0.42, 0.56],
            _ => egg.color,
        };
        ui.rect(tx - sz * 0.5, ty - sz * 0.5, sz, sz, Color::new(col[0], col[1], col[2], fade));
    }
}

pub(crate) fn floor_trail(ui: &Ui, vx: f32, vy: f32, egg: &ActiveEgg, alpha: f32) {
    let dir = if egg.flip { 1.0 } else { -1.0 };
    let is_dust = egg.sprite == EggSprite::Tumbleweed;
    for i in 1..=6 {
        let lag = i as f32;
        let tx = vx + dir * (18.0 + lag * 14.0);
        let lift = ((egg.t * 11.0 + lag).sin().abs()) * 10.0 + lag * 2.2;
        let ty = vy + 24.0 - lift;
        let fade = (1.0 - lag / 7.0) * 0.65 * alpha;
        let sz = if is_dust { 5.0 + lag * 0.9 } else { 4.5 };
        let col = if is_dust {
            [0.86, 0.74, 0.54]
        } else if i % 2 == 0 {
            [0.44, 0.9, 0.48]
        } else {
            [1.0, 0.78, 0.28]
        };
        ui.rect(tx - sz * 0.5, ty - sz * 0.5, sz, sz, Color::new(col[0], col[1], col[2], fade));
    }
}

pub(crate) fn ribbon_shimmer(ui: &Ui, x: f32, y: f32, w: f32, h: f32, t: f32, alpha: f32) {
    let sweep = ((t * 0.85) % 1.4) - 0.2;
    if !(0.0..=1.0).contains(&sweep) {
        return;
    }
    let sx = x + sweep * (w - 36.0);
    ui.rect(sx, y + 2.0, 28.0, h - 4.0, Color::new(1.0, 1.0, 1.0, 0.14 * alpha));
}

pub(crate) fn speech_tail(ui: &Ui, x: f32, y: f32, w: f32, h: f32, side: usize, accent: Color, alpha: f32) {
    let tx = if side == 0 { x + 28.0 } else { x + w - 42.0 };
    let ty = y + h;
    ui.rect(tx, ty, 14.0, 6.0, Color::new(0.07, 0.06, 0.14, 0.9 * alpha));
    ui.rect(tx + 3.0, ty + 6.0, 8.0, 5.0, Color::new(0.07, 0.06, 0.14, 0.9 * alpha));
    ui.rect(tx + 4.0, ty + 10.0, 5.0, 3.0, accent);
}
