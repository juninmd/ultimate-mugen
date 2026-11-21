use super::particles;
use super::sprite::draw_sprite;
use super::{ActiveEgg, EggSprite, EggStyle};
use crate::gfx::*;
use crate::ui::{Align, Ui, VH, VW};

pub(crate) fn render_all(ui: &Ui, eggs: &[ActiveEgg]) {
    let mut ribbon_slot = 0;
    for egg in eggs {
        match egg.style {
            EggStyle::CornerPop => draw_corner_pop(ui, egg),
            EggStyle::FloorRoll => draw_floor_roll(ui, egg),
            EggStyle::Flyby => draw_flyby(ui, egg),
            EggStyle::TopRibbon => {
                draw_top_ribbon(ui, egg, ribbon_slot);
                ribbon_slot += 1;
            }
            EggStyle::SpeechBubble => draw_speech_bubble(ui, egg),
        }
    }
}

fn envelope(t: f32, dur: f32) -> f32 {
    let fade_in = (t / 0.18).clamp(0.0, 1.0);
    let fade_out = ((dur - t) / 0.28).clamp(0.0, 1.0);
    fade_in * fade_out
}

fn ease_out_back(u: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let x = u.clamp(0.0, 1.0) - 1.0;
    1.0 + c3 * x * x * x + c1 * x * x
}

fn draw_corner_pop(ui: &Ui, egg: &ActiveEgg) {
    let enter = ease_out_back((egg.t / 0.26).clamp(0.0, 1.0));
    let exit = ((egg.dur - egg.t) / 0.22).clamp(0.0, 1.0);
    let slide = (1.0 - enter) * 180.0 + (1.0 - exit) * 200.0;
    let bx = VW - 118.0 + slide;
    let by = VH - 92.0 + (egg.t * 14.0).sin().abs() * -6.0;
    let alpha = exit;

    particles::confetti_burst(ui, bx - 40.0, by - 40.0, egg.t, alpha, egg.color);

    let col = Color::new(egg.color[0], egg.color[1], egg.color[2], alpha);
    let tw = ui.measure(&egg.title, 26.0, true).max(ui.measure(&egg.sub, 14.0, false));
    let bw = (tw + 38.0).max(200.0);
    let px = (bx - bw - 56.0).max(40.0);
    let py = by - 58.0;

    ui.rect(px, py, bw, 62.0, Color::new(0.08, 0.05, 0.14, 0.92 * alpha));
    particles::ribbon_shimmer(ui, px, py, bw, 62.0, egg.t, alpha);
    ui.rect_lines(px, py, bw, 62.0, 3.0, col);
    ui.text_bold(
        &egg.title,
        px + bw * 0.5,
        py + 31.0,
        26.0,
        Color::new(1.0, 0.96, 0.72, alpha),
        Color::new(0.06, 0.03, 0.1, alpha),
        Align::Center,
    );
    if !egg.sub.is_empty() {
        ui.text(&egg.sub, px + bw * 0.5, py + 52.0, 14.0, Color::new(0.95, 0.95, 1.0, 0.92 * alpha), Align::Center, false);
    }

    draw_sprite(egg.sprite, ui.x(bx), ui.y(by), 144.0 * ui.s, egg.t, alpha, egg.flip, 0.0);
}

fn draw_floor_roll(ui: &Ui, egg: &ActiveEgg) {
    let p = (egg.t / egg.dur).clamp(0.0, 1.0);
    let alpha = envelope(egg.t, egg.dur);
    let vx = if egg.flip { VW + 60.0 - p * (VW + 120.0) } else { -60.0 + p * (VW + 120.0) };
    let is_roller = egg.sprite == EggSprite::Tumbleweed;
    let bounce = (p * std::f32::consts::PI * if is_roller { 9.0 } else { 12.0 }).sin().abs();
    let vy = VH - 88.0 - bounce * if is_roller { 26.0 } else { 8.0 };
    let rot = if is_roller { p * 14.0 * if egg.flip { -1.0 } else { 1.0 } } else { 0.0 };

    particles::floor_trail(ui, vx, vy, egg, alpha);
    draw_sprite(egg.sprite, ui.x(vx), ui.y(vy), 84.0 * ui.s, egg.t, alpha, egg.flip, rot);
    draw_top_ribbon(ui, egg, 0);
}

fn draw_flyby(ui: &Ui, egg: &ActiveEgg) {
    let p = (egg.t / egg.dur).clamp(0.0, 1.0);
    let alpha = envelope(egg.t, egg.dur);
    let vx = if egg.flip { VW + 80.0 - p * (VW + 160.0) } else { -80.0 + p * (VW + 160.0) };
    let vy = 210.0 + (p * std::f32::consts::PI * 4.0).sin() * 24.0;

    particles::flyby_trail(ui, vx, vy, egg, alpha);
    draw_sprite(egg.sprite, ui.x(vx), ui.y(vy), 82.0 * ui.s, egg.t, alpha, egg.flip, 0.0);
    draw_top_ribbon(ui, egg, 0);
}

fn draw_top_ribbon(ui: &Ui, egg: &ActiveEgg, slot: usize) {
    let alpha = envelope(egg.t, egg.dur);
    let slide = (1.0 - ease_out_back((egg.t / 0.24).clamp(0.0, 1.0))) * -48.0;
    let tw = ui.measure(&egg.title, 20.0, true).max(ui.measure(&egg.sub, 13.0, false));
    let has_icon = egg.sprite != EggSprite::None;
    let w = (tw + if has_icon { 92.0 } else { 44.0 }).max(240.0);
    let h = 50.0;
    let x = VW * 0.5 - w * 0.5;
    let y = 116.0 + slot as f32 * 58.0 + slide;
    let accent = Color::new(egg.color[0], egg.color[1], egg.color[2], alpha);

    if egg.sprite == EggSprite::CrownPerfect {
        particles::confetti_burst(ui, x + w * 0.5, y + h * 0.5, egg.t, alpha * 0.85, egg.color);
    }

    ui.rect(x, y, w, h, Color::new(0.06, 0.05, 0.12, 0.86 * alpha));
    particles::ribbon_shimmer(ui, x, y, w, h, egg.t, alpha);
    ui.rect(x, y, 5.0, h, accent);
    ui.rect_lines(x, y, w, h, 2.0, Color::new(egg.color[0], egg.color[1], egg.color[2], 0.85 * alpha));

    let text_cx = x + w * 0.5 + if has_icon { 16.0 } else { 0.0 };
    if has_icon {
        draw_sprite(egg.sprite, ui.x(x + 32.0), ui.y(y + h * 0.5), 40.0 * ui.s, egg.t, alpha, false, 0.0);
    }
    ui.text_bold(
        &egg.title,
        text_cx,
        y + 24.0,
        20.0,
        Color::new(1.0, 0.96, 0.84, alpha),
        Color::new(0.05, 0.03, 0.1, alpha),
        Align::Center,
    );
    if !egg.sub.is_empty() {
        ui.text(&egg.sub, text_cx, y + 42.0, 13.0, Color::new(0.86, 0.9, 0.98, 0.9 * alpha), Align::Center, false);
    }
}

fn draw_speech_bubble(ui: &Ui, egg: &ActiveEgg) {
    let alpha = envelope(egg.t, egg.dur);
    let pop = ease_out_back((egg.t / 0.22).clamp(0.0, 1.0));
    let tw = ui.measure(&egg.title, 18.0, false).max(ui.measure(&egg.speaker, 14.0, true));
    let w = (tw + 36.0).max(220.0).min(480.0);
    let h = 58.0;
    let x = if egg.side == 0 { 46.0 } else { VW - w - 46.0 };
    let y = 148.0 + (1.0 - pop) * -28.0;
    let accent = Color::new(egg.color[0], egg.color[1], egg.color[2], alpha);

    ui.rect(x, y, w, h, Color::new(0.07, 0.06, 0.14, 0.9 * alpha));
    ui.rect(x, y, w, 4.0, accent);
    ui.rect_lines(x, y, w, h, 2.0, Color::new(1.0, 1.0, 1.0, 0.32 * alpha));
    particles::speech_tail(ui, x, y, w, h, egg.side, accent, alpha);

    let align = if egg.side == 0 { Align::Left } else { Align::Right };
    let tx = if egg.side == 0 { x + 16.0 } else { x + w - 16.0 };
    if !egg.speaker.is_empty() {
        ui.text_bold(&egg.speaker, tx, y + 22.0, 14.0, accent, Color::new(0.05, 0.03, 0.1, alpha), align);
    }
    ui.text(&egg.title, tx, y + 45.0, 18.0, Color::new(0.98, 0.98, 1.0, alpha), align, false);
}
