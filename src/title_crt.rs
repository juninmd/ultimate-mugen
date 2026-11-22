use crate::gfx::*;
use crate::ui::Ui;

const SCANLINE_PITCH: f32 = 3.0;
const SPECKS: u32 = 260;

fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn unit(seed: u32) -> f32 {
    (hash(seed) & 0xffff) as f32 / 65535.0
}

pub fn flicker(frame: u32) -> f32 {
    1.0 + (unit(frame / 2) - 0.5) * 0.08
}

pub fn sweep_position(time: f32) -> f32 {
    (time * 0.11).fract()
}

pub fn draw(ui: &Ui, time: f32, animated: bool) {
    if !crate::lite::crt() {
        return;
    }
    let (w, h) = (screen_width(), screen_height());
    let pitch = SCANLINE_PITCH * ui.s.max(0.5);
    let frame = if animated { (time * 30.0) as u32 } else { 0 };
    let level = if animated { flicker(frame) } else { 1.0 };
    let mut y = 0.0;
    while y < h {
        draw_rectangle(
            0.0,
            y,
            w,
            pitch * 0.45,
            Color::new(0.0, 0.0, 0.02, 0.3 * level),
        );
        y += pitch;
    }
    if !animated {
        return;
    }
    let band_y = sweep_position(time) * (h + 120.0 * ui.s) - 60.0 * ui.s;
    draw_rectangle(
        0.0,
        band_y,
        w,
        60.0 * ui.s,
        Color::new(0.75, 0.9, 1.0, 0.06),
    );
    let tear = unit(frame.wrapping_mul(7919));
    if tear > 0.93 {
        let ty = unit(frame ^ 0x55) * h;
        draw_rectangle(0.0, ty, w, 2.0 * ui.s, Color::new(0.85, 0.95, 1.0, 0.1));
    }
    for i in 0..SPECKS {
        let seed = frame.wrapping_mul(2654435761).wrapping_add(i * 977);
        let (x, y) = (unit(seed) * w, unit(seed ^ 0xa5a5) * h);
        let size = (1.0 + unit(seed ^ 0x3c3c) * 2.0) * ui.s;
        let tone = 0.55 + unit(seed ^ 0x9e37) * 0.45;
        draw_rectangle(x, y, size * 3.0, size, Color::new(tone, tone, tone, 0.2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flicker_stays_within_a_small_band() {
        assert!((0..2000).all(|f| (flicker(f) - 1.0).abs() <= 0.0401));
    }

    #[test]
    fn sweep_wraps_inside_the_unit_range() {
        assert!((0..500).all(|t| (0.0..1.0).contains(&sweep_position(t as f32 * 0.37))));
    }

    #[test]
    fn noise_changes_between_frames() {
        assert_ne!(unit(1), unit(2));
    }
}
