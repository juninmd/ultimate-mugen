//! Video / quality settings: fullscreen, window size presets (up to 4K), art quality, FPS counter. Persisted next to the executable (native).
use crate::gfx::*;

pub const RESOLUTIONS: [(u32, u32, &str); 5] = [(1280, 720, "1280×720 (HD)"), (1920, 1080, "1920×1080 (Full HD)"), (2560, 1440, "2560×1440 (QHD)"), (3840, 2160, "3840×2160 (4K UHD)"), (5120, 2880, "5120×2880 (5K)")];
/// Art quality: 0 = auto (by display height), then fixed pixels-per-unit for procedural art.
/// Post-processing level (0 off · 1 grade/vignette/chroma · 2 + bloom) and impact-effects intensity (accessibility: reduced shake/flash).
pub const FX_LEVELS: [&str; 3] = ["Desligado", "Médio", "Alto (bloom)"];
pub const IMPACT: [(f32, &str); 4] = [(0.0, "Mínimo (fotossensibilidade)"), (0.4, "Reduzido"), (1.0, "Normal"), (1.3, "Intenso")];
pub const QUALITY: [(f32, &str); 4] = [(0.0, "Automática"), (3.0, "Padrão (3×)"), (4.5, "Alta (4,5×)"), (6.0, "Ultra 4K (6×)")];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings { pub fullscreen: bool, pub res: usize, pub quality: usize, pub fps: bool, pub fx: usize, pub impact: usize, pub keep_aspect: bool, pub linear: bool, pub window: (u32, u32) }

impl Default for Settings { fn default() -> Self { Settings { fullscreen: false, res: 1, quality: 0, fps: false, fx: 2, impact: 2, keep_aspect: true, linear: false, window: (0, 0) } } }

/// Art resolution (pixels per game unit) for a display of physical height `h`.
pub fn auto_quality(h: f32) -> f32 { if h <= 900.0 { 3.0 } else if h <= 1300.0 { 4.0 } else if h <= 1700.0 { 4.5 } else { 6.0 } }

impl Settings {
    pub fn art_res(&self, physical_h: f32) -> f32 { let q = QUALITY[self.quality.min(QUALITY.len() - 1)].0; if q <= 0.0 { auto_quality(physical_h) } else { q } }
    #[cfg(not(target_arch = "wasm32"))]
    fn path() -> std::path::PathBuf {
        #[cfg(target_os = "android")]
        if let Some(d) = ume_shell::ANDROID_APP.get().and_then(|a| a.internal_data_path()) { return d.join("ultimate-mugen.cfg"); }
        std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("ultimate-mugen.cfg"))).unwrap_or_else(|| "ultimate-mugen.cfg".into())
    }
    pub fn load() -> Self {
        let mut s = Settings::default();
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(t) = std::fs::read_to_string(Self::path()) {
            for l in t.lines() {
                let Some((k, v)) = l.split_once('=') else { continue };
                let (k, v) = (k.trim(), v.trim());
                match k {
                    "fullscreen" => s.fullscreen = v == "1",
                    "res" => s.res = v.parse().unwrap_or(1).min(RESOLUTIONS.len() - 1),
                    "quality" => s.quality = v.parse().unwrap_or(0).min(QUALITY.len() - 1),
                    "fps" => s.fps = v == "1",
                    "fx" => s.fx = v.parse().unwrap_or(2).min(2),
                    "impact_lv" => s.impact = v.parse().unwrap_or(2).min(IMPACT.len() - 1),
                    "keep_aspect" => s.keep_aspect = v != "0",
                    "filter" => s.linear = v == "linear",
                    "window" => s.window = parse_window(v),
                    _ => {}
                }
            }
        }
        s
    }
    pub fn save(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        let _ = std::fs::write(Self::path(), format!("fullscreen={}\nres={}\nquality={}\nfps={}\nfx={}\nimpact_lv={}\nkeep_aspect={}\nfilter={}\nwindow={}x{}\n", self.fullscreen as i32, self.res, self.quality, self.fps as i32, self.fx, self.impact, self.keep_aspect as i32, if self.linear { "linear" } else { "nearest" }, self.window.0, self.window.1));
    }
    /// Physical (pixel) height of the drawable area.
    pub fn physical_h() -> f32 { screen_height() * dpi_scale() }
    pub fn window_size(&self) -> (u32, u32) {
        if self.window.0 >= MIN_WINDOW.0 && self.window.1 >= MIN_WINDOW.1 { return self.window; }
        let (w, h, _) = RESOLUTIONS[self.res.min(RESOLUTIONS.len() - 1)];
        (w, h)
    }
    pub fn apply_window(&self) {
        set_fullscreen(self.fullscreen);
        #[cfg(not(target_arch = "wasm32"))]
        if !self.fullscreen { let (w, h) = self.window_size(); { let d = dpi_scale().max(0.5); request_new_screen_size(w as f32 / d, h as f32 / d) }; }
    }
}

pub const MIN_WINDOW: (u32, u32) = (320, 180);

fn parse_window(v: &str) -> (u32, u32) {
    v.split_once('x').and_then(|(w, h)| Some((w.trim().parse().ok()?, h.trim().parse().ok()?))).unwrap_or((0, 0))
}

#[derive(Default)]
pub struct WindowWatch { seen: (u32, u32), stable: f32 }

impl WindowWatch {
    const SETTLE_SECONDS: f32 = 0.8;

    pub fn observe(&mut self, size: (u32, u32), dt: f32, fullscreen: bool, saved: (u32, u32)) -> Option<(u32, u32)> {
        if fullscreen || size.0 < MIN_WINDOW.0 || size.1 < MIN_WINDOW.1 { self.seen = (0, 0); return None; }
        if size != self.seen { self.seen = size; self.stable = 0.0; return None; }
        self.stable += dt;
        (self.stable >= Self::SETTLE_SECONDS && size != saved).then_some(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_size_prefers_saved_window_over_preset() {
        let mut s = Settings::default();
        assert_eq!(s.window_size(), (1920, 1080));
        s.window = (1000, 700);
        assert_eq!(s.window_size(), (1000, 700));
        s.window = (10, 10);
        assert_eq!(s.window_size(), (1920, 1080));
    }

    #[test]
    fn window_string_round_trips_and_rejects_garbage() {
        assert_eq!(parse_window("1366x768"), (1366, 768));
        assert_eq!(parse_window("junk"), (0, 0));
        assert_eq!(parse_window("12x"), (0, 0));
    }

    #[test]
    fn watch_reports_only_after_the_resize_settles() {
        let mut w = WindowWatch::default();
        assert_eq!(w.observe((900, 600), 0.1, false, (0, 0)), None);
        assert_eq!(w.observe((900, 600), 0.5, false, (0, 0)), None);
        assert_eq!(w.observe((900, 600), 0.5, false, (0, 0)), Some((900, 600)));
        assert_eq!(w.observe((900, 600), 0.5, false, (900, 600)), None);
        assert_eq!(w.observe((910, 600), 2.0, false, (900, 600)), None);
    }

    #[test]
    fn watch_ignores_fullscreen_and_minimised_windows() {
        let mut w = WindowWatch::default();
        for _ in 0..10 { assert_eq!(w.observe((1920, 1080), 0.5, true, (0, 0)), None); }
        for _ in 0..10 { assert_eq!(w.observe((0, 0), 0.5, false, (0, 0)), None); }
    }
}
