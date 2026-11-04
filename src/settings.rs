//! Video / quality settings: fullscreen, window size presets (up to 4K), art quality, FPS counter. Persisted next to the executable (native).
use crate::gfx::*;

pub const RESOLUTIONS: [(u32, u32, &str); 5] = [(1280, 720, "1280×720 (HD)"), (1920, 1080, "1920×1080 (Full HD)"), (2560, 1440, "2560×1440 (QHD)"), (3840, 2160, "3840×2160 (4K UHD)"), (5120, 2880, "5120×2880 (5K)")];
/// Art quality: 0 = auto (by display height), then fixed pixels-per-unit for procedural art.
/// Post-processing level (0 off · 1 grade/vignette/chroma · 2 + bloom) and impact-effects intensity (accessibility: reduced shake/flash).
pub const FX_LEVELS: [&str; 3] = ["Desligado", "Médio", "Alto (bloom)"];
pub const IMPACT: [(f32, &str); 4] = [(0.0, "Mínimo (fotossensibilidade)"), (0.4, "Reduzido"), (1.0, "Normal"), (1.3, "Intenso")];
pub const QUALITY: [(f32, &str); 4] = [(0.0, "Automática"), (3.0, "Padrão (3×)"), (4.5, "Alta (4,5×)"), (6.0, "Ultra 4K (6×)")];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings { pub fullscreen: bool, pub res: usize, pub quality: usize, pub fps: bool, pub fx: usize, pub impact: usize }

impl Default for Settings { fn default() -> Self { Settings { fullscreen: false, res: 1, quality: 0, fps: false, fx: 2, impact: 2 } } }

/// Art resolution (pixels per game unit) for a display of physical height `h`.
pub fn auto_quality(h: f32) -> f32 { if h <= 900.0 { 3.0 } else if h <= 1300.0 { 4.0 } else if h <= 1700.0 { 4.5 } else { 6.0 } }

impl Settings {
    pub fn art_res(&self, physical_h: f32) -> f32 { let q = QUALITY[self.quality.min(QUALITY.len() - 1)].0; if q <= 0.0 { auto_quality(physical_h) } else { q } }
    #[cfg(not(target_arch = "wasm32"))]
    fn path() -> std::path::PathBuf { std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("ultimate-mugen.cfg"))).unwrap_or_else(|| "ultimate-mugen.cfg".into()) }
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
                    _ => {}
                }
            }
        }
        s
    }
    pub fn save(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        let _ = std::fs::write(Self::path(), format!("fullscreen={}\nres={}\nquality={}\nfps={}\nfx={}\nimpact_lv={}\n", self.fullscreen as i32, self.res, self.quality, self.fps as i32, self.fx, self.impact));
    }
    /// Physical (pixel) height of the drawable area.
    pub fn physical_h() -> f32 { screen_height() * dpi_scale() }
    pub fn apply_window(&self) {
        set_fullscreen(self.fullscreen);
        #[cfg(not(target_arch = "wasm32"))]
        if !self.fullscreen { let (w, h, _) = RESOLUTIONS[self.res.min(RESOLUTIONS.len() - 1)]; { let d = dpi_scale().max(0.5); request_new_screen_size(w as f32 / d, h as f32 / d) }; }
    }
}
