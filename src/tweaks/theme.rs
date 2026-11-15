use super::model::Tweaks;
use crate::gfx::Color;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const ROLES: usize = 5;
const ACCENT_ALIASES: [u32; 2] = [0xff4d6d, 0xff4d6e];
const LIFE_TOP: [u8; 3] = [0xff, 0xf2, 0xa8];
const LIFE_LOW_TOP: [u8; 3] = [0xff, 0xb0, 0x66];
const TRAIL_TOP: [u8; 3] = [0xff, 0x8c, 0x8c];
const HIGHLIGHT: f32 = 0.6;
const TRAIL_ALPHA: f32 = 0.85;
const ACCENT_BRIGHT: [f32; 3] = [1.0, 0.3, 0.43];
const ACCENT_DEEP: [f32; 3] = [0.63, 0.12, 0.3];
const DEEPEN: f32 = 0.62;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static TARGETS: [AtomicU32; ROLES] = [const { AtomicU32::new(0) }; ROLES];

fn pack(color: [u8; 3]) -> u32 {
    (color[0] as u32) << 16 | (color[1] as u32) << 8 | color[2] as u32
}

fn roles(tweaks: &Tweaks) -> [[u8; 3]; ROLES] {
    [
        tweaks.accent,
        tweaks.title,
        tweaks.info,
        tweaks.muted,
        tweaks.text,
    ]
}

pub fn install(tweaks: &Tweaks) {
    let wanted = roles(tweaks);
    for (slot, color) in TARGETS.iter().zip(wanted) {
        slot.store(pack(color), Ordering::Relaxed);
    }
    ACTIVE.store(wanted != roles(&Tweaks::default()), Ordering::Relaxed);
}

pub fn remap(color: u32) -> u32 {
    if !ACTIVE.load(Ordering::Relaxed) {
        return color;
    }
    let sources = roles(&Tweaks::default());
    let role = if ACCENT_ALIASES.contains(&color) {
        Some(0)
    } else {
        sources.iter().position(|source| pack(*source) == color)
    };
    role.map_or(color, |index| TARGETS[index].load(Ordering::Relaxed))
}

fn shade(color: [u8; 3], alpha: f32) -> Color {
    Color::new(
        color[0] as f32 / 255.0,
        color[1] as f32 / 255.0,
        color[2] as f32 / 255.0,
        alpha,
    )
}

fn lifted(color: [u8; 3]) -> [u8; 3] {
    color.map(|channel| (channel as f32 + (255.0 - channel as f32) * HIGHLIGHT) as u8)
}

fn pair(chosen: [u8; 3], stock: [u8; 3], stock_top: [u8; 3], alpha: f32) -> (Color, Color) {
    let top = if chosen == stock {
        stock_top
    } else {
        lifted(chosen)
    };
    (shade(top, alpha), shade(chosen, alpha))
}

pub fn life_fill(low: bool) -> (Color, Color) {
    let (tweaks, stock) = (super::current(), Tweaks::default());
    if low {
        pair(tweaks.life_low, stock.life_low, LIFE_LOW_TOP, 1.0)
    } else {
        pair(tweaks.life, stock.life, LIFE_TOP, 1.0)
    }
}

pub fn life_trail() -> (Color, Color) {
    pair(
        super::current().damage,
        Tweaks::default().damage,
        TRAIL_TOP,
        TRAIL_ALPHA,
    )
}

fn custom_accent() -> Option<[u8; 3]> {
    let accent = super::current().accent;
    (accent != Tweaks::default().accent).then_some(accent)
}

pub fn menu_accent() -> Option<Color> {
    custom_accent().map(|accent| shade(accent, 1.0))
}

pub fn accent(alpha: f32) -> Color {
    custom_accent().map_or(
        Color::new(ACCENT_BRIGHT[0], ACCENT_BRIGHT[1], ACCENT_BRIGHT[2], alpha),
        |accent| shade(accent, alpha),
    )
}

pub fn accent_deep(alpha: f32) -> Color {
    let stock = Color::new(ACCENT_DEEP[0], ACCENT_DEEP[1], ACCENT_DEEP[2], alpha);
    custom_accent().map_or(stock, |accent| {
        shade(accent.map(|channel| (channel as f32 * DEEPEN) as u8), alpha)
    })
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
