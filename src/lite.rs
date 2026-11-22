mod auto;
mod flags;
mod text;

pub use auto::Monitor;
pub use flags::{afterimage, apply, backdrop, crt, glass, scale, shadows};

use crate::settings::Settings;

pub const SCALES: [f32; 5] = [1.0, 0.75, 0.5, 0.35, 0.25];
pub const PRESET_COUNT: usize = 4;
pub const CUSTOM: usize = PRESET_COUNT;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Lite {
    pub scale: u8,
    pub particles: u8,
    pub afterimage: bool,
    pub glass: bool,
    pub shadows: bool,
    pub backdrop: bool,
    pub crt: bool,
    pub auto: bool,
}

impl Default for Lite {
    fn default() -> Self {
        Lite { scale: 0, particles: 2, afterimage: true, glass: true, shadows: true, backdrop: true, crt: true, auto: cfg!(target_arch = "wasm32") }
    }
}

const PRESETS: [(u8, u8, u8, bool, bool, bool, bool, bool); PRESET_COUNT] = [
    (0, 2, 2, true, true, true, true, true),
    (0, 1, 1, true, true, true, true, false),
    (1, 0, 1, false, false, true, false, false),
    (2, 0, 0, false, false, false, false, false),
];

impl Lite {
    pub fn render_scale(&self) -> f32 {
        SCALES[(self.scale as usize).min(SCALES.len() - 1)]
    }
}

impl Settings {
    pub fn preset(&self) -> usize {
        (0..PRESET_COUNT).find(|&p| self.matches_preset(p)).unwrap_or(CUSTOM)
    }

    fn matches_preset(&self, p: usize) -> bool {
        let (scale, fx, particles, afterimage, glass, shadows, backdrop, crt) = PRESETS[p];
        let l = &self.lite;
        (l.scale, self.fx, l.particles, l.afterimage, l.glass, l.shadows, l.backdrop, l.crt) == (scale, fx as usize, particles, afterimage, glass, shadows, backdrop, crt)
    }

    pub fn set_preset(&mut self, preset: usize) {
        let (scale, fx, particles, afterimage, glass, shadows, backdrop, crt) = PRESETS[preset.min(PRESET_COUNT - 1)];
        let auto = self.lite.auto;
        self.lite = Lite { scale, particles, afterimage, glass, shadows, backdrop, crt, auto };
        self.fx = fx as usize;
    }

    pub fn lighter_step(&mut self) -> bool {
        let next = match self.preset() {
            CUSTOM => 2,
            p if p + 1 < PRESET_COUNT => p + 1,
            _ => return false,
        };
        self.set_preset(next);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_the_high_preset() {
        assert_eq!(Settings::default().preset(), 0);
    }

    #[test]
    fn every_preset_round_trips_and_gets_lighter() {
        let mut s = Settings::default();
        for p in 0..PRESET_COUNT {
            s.set_preset(p);
            assert_eq!(s.preset(), p);
        }
        s.set_preset(0);
        while s.lighter_step() {}
        assert_eq!(s.preset(), PRESET_COUNT - 1);
        assert!(!s.lighter_step());
    }

    #[test]
    fn changing_one_option_makes_it_custom_and_persists() {
        let mut s = Settings::default();
        s.lite.shadows = false;
        assert_eq!(s.preset(), CUSTOM);
        let back = Settings::from_text(&s.to_text());
        assert_eq!(back.lite, s.lite);
    }
}
