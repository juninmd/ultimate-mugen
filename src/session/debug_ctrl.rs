use crate::audio::Audio;
use crate::controls::Controls;
use crate::gfx::{is_key_pressed, KeyCode};
use crate::session::Session;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

static FROZEN: AtomicBool = AtomicBool::new(false);
static SPEED_IDX: AtomicU8 = AtomicU8::new(0);
static TAB_IDX: AtomicU8 = AtomicU8::new(0);
static LAYER_IDX: AtomicU8 = AtomicU8::new(0);

const SPEEDS: [f32; 4] = [1.0, 0.5, 0.25, 0.1];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DebugTab {
    Overview = 0,
    VarsMaps = 1,
    Clipboard = 2,
    Console = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DebugLayers {
    All = 0,
    BoxesOnly = 1,
    HudOnly = 2,
}

pub fn reset() {
    FROZEN.store(false, Ordering::Relaxed);
    SPEED_IDX.store(0, Ordering::Relaxed);
    LAYER_IDX.store(0, Ordering::Relaxed);
}

pub fn frozen() -> bool {
    FROZEN.load(Ordering::Relaxed)
}

pub fn set_frozen(on: bool) {
    FROZEN.store(on, Ordering::Relaxed);
}

pub fn speed_mul() -> f32 {
    SPEEDS[(SPEED_IDX.load(Ordering::Relaxed) as usize) % SPEEDS.len()]
}

pub fn tab() -> DebugTab {
    match TAB_IDX.load(Ordering::Relaxed) % 4 {
        1 => DebugTab::VarsMaps,
        2 => DebugTab::Clipboard,
        3 => DebugTab::Console,
        _ => DebugTab::Overview,
    }
}

pub fn layers() -> DebugLayers {
    match LAYER_IDX.load(Ordering::Relaxed) % 3 {
        1 => DebugLayers::BoxesOnly,
        2 => DebugLayers::HudOnly,
        _ => DebugLayers::All,
    }
}

pub fn cycle_tab() -> DebugTab {
    let next = (TAB_IDX.load(Ordering::Relaxed) + 1) % 4;
    TAB_IDX.store(next, Ordering::Relaxed);
    tab()
}

pub fn cycle_speed() -> f32 {
    let next = (SPEED_IDX.load(Ordering::Relaxed) + 1) % SPEEDS.len() as u8;
    SPEED_IDX.store(next, Ordering::Relaxed);
    speed_mul()
}

pub fn cycle_layers() -> DebugLayers {
    let next = (LAYER_IDX.load(Ordering::Relaxed) + 1) % 3;
    LAYER_IDX.store(next, Ordering::Relaxed);
    layers()
}

pub fn handle_fight_keys(s: &mut Session, replaying: bool, c: &mut Controls, audio: &mut Audio) {
    if !s.r.debug {
        if frozen() || SPEED_IDX.load(Ordering::Relaxed) != 0 {
            reset();
        }
        return;
    }
    if is_key_pressed(KeyCode::F4) {
        cycle_tab();
    }
    if is_key_pressed(KeyCode::F5) {
        set_frozen(!frozen());
    }
    if is_key_pressed(KeyCode::F7) {
        cycle_speed();
    }
    if is_key_pressed(KeyCode::F8) {
        cycle_layers();
    }
    let step_key = is_key_pressed(KeyCode::F6)
        || (!replaying
            && frozen()
            && (is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::Right)));
    if step_key {
        set_frozen(true);
        s.step_frame(c, audio);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_tabs_speeds_and_layers_cleanly() {
        reset();
        TAB_IDX.store(0, Ordering::Relaxed);
        assert_eq!(tab(), DebugTab::Overview);
        assert_eq!(cycle_tab(), DebugTab::VarsMaps);
        assert_eq!(cycle_tab(), DebugTab::Clipboard);
        assert_eq!(cycle_tab(), DebugTab::Console);
        assert_eq!(cycle_tab(), DebugTab::Overview);

        assert_eq!(speed_mul(), 1.0);
        assert_eq!(cycle_speed(), 0.5);
        assert_eq!(cycle_speed(), 0.25);
        assert_eq!(cycle_speed(), 0.1);
        assert_eq!(cycle_speed(), 1.0);

        assert_eq!(layers(), DebugLayers::All);
        assert_eq!(cycle_layers(), DebugLayers::BoxesOnly);
        assert_eq!(cycle_layers(), DebugLayers::HudOnly);
        assert_eq!(cycle_layers(), DebugLayers::All);
    }
}
