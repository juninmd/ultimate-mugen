//! Player settings (video, audio, gameplay rules, language), persisted through `storage` (file on native, localStorage on web).
use crate::gfx::*;

mod persist;
mod window;
#[cfg(test)]
mod tests;

pub use window::WindowWatch;

pub const RESOLUTIONS: [(u32, u32, &str); 5] = [
    (1280, 720, "1280×720 (HD)"),
    (1920, 1080, "1920×1080 (Full HD)"),
    (2560, 1440, "2560×1440 (QHD)"),
    (3840, 2160, "3840×2160 (4K UHD)"),
    (5120, 2880, "5120×2880 (5K)"),
];
pub const FX_LEVELS: [&str; 3] = ["Desligado", "Médio", "Alto (bloom)"];
pub const IMPACT: [(f32, &str); 4] = [
    (0.0, "Mínimo (fotossensibilidade)"),
    (0.4, "Reduzido"),
    (1.0, "Normal"),
    (1.3, "Intenso"),
];
pub const QUALITY: [(f32, &str); 4] = [
    (0.0, "Automática"),
    (3.0, "Padrão (3×)"),
    (4.5, "Alta (4,5×)"),
    (6.0, "Ultra 4K (6×)"),
];

pub const FPS_CAPS: [u32; 5] = [0, 30, 60, 120, 144];
pub const GRID_PRESETS: [(&str, usize, usize, usize); 3] = [("COMPACTO", 9, 8, 12), ("NORMAL", 7, 6, 10), ("GRANDE", 4, 4, 6)];
pub const GAME_SPEEDS: [f32; 5] = [0.75, 0.9, 1.0, 1.15, 1.3];
pub const ROUND_SECONDS: [u32; 4] = [99, 60, 30, 0];
pub const ROUNDS_TO_WIN: [i32; 3] = [1, 2, 3];
pub const LIFE_PERCENT: [u32; 5] = [50, 75, 100, 150, 200];
/// Life recovered between Survival fights, in percent of the bar.
pub const SURVIVAL_RECOVERY: [u32; 5] = [0, 20, 35, 50, 100];
pub const VOLUME_STEPS: u8 = 10;
/// Palettes a fighter can use (M.U.G.E.N `pal1..pal12`).
pub const PALETTE_COUNT: usize = 12;
pub const FORMAT_VERSION: u32 = 2;
const FILE: &str = "ultimate-mugen.cfg";

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings {
    pub fullscreen: bool,
    pub res: usize,
    pub quality: usize,
    pub fps: bool,
    pub fx: usize,
    pub impact: usize,
    pub vsync: bool,
    pub fps_cap: usize,
    pub keep_aspect: bool,
    pub linear: bool,
    pub window: (u32, u32),
    pub master: u8,
    pub music: u8,
    pub sfx: u8,
    pub mute: bool,
    pub difficulty: i32,
    pub round_time: usize,
    pub rounds: usize,
    pub life: usize,
    pub single_round: bool,
    pub classic_pushbox: bool,
    pub team_life_share: bool,
    pub game_speed: usize,
    /// Survival: life recovered between fights (index into `SURVIVAL_RECOVERY`) and the local best score
    pub survival_recovery: usize,
    pub survival_best: u64,
    /// Last palette chosen for P1 and P2 (1..=PALETTE_COUNT)
    pub pal1: usize,
    pub pal2: usize,
    pub grid: usize,
    pub lang: crate::i18n::Lang,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            fullscreen: false,
            res: 1,
            quality: 0,
            fps: false,
            fx: 2,
            impact: 2,
            vsync: true,
            fps_cap: 0,
            keep_aspect: true,
            linear: false,
            window: (0, 0),
            master: 8,
            music: 10,
            sfx: 10,
            mute: false,
            difficulty: 4,
            round_time: 0,
            rounds: 1,
            life: 2,
            single_round: false,
            classic_pushbox: true,
            team_life_share: true,
            game_speed: 2,
            survival_recovery: 2,
            survival_best: 0,
            pal1: 1,
            pal2: 1,
            grid: 1,
            lang: crate::i18n::Lang::En,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rules {
    pub rounds_to_win: i32,
    pub round_ticks: i32,
    pub life_percent: u32,
    pub single_round: bool,
    pub classic_pushbox: bool,
    pub team_life_share: bool,
    pub game_speed: f32,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            rounds_to_win: 2,
            round_ticks: 99 * 60,
            life_percent: 100,
            single_round: false,
            classic_pushbox: true,
            team_life_share: true,
            game_speed: 1.0,
        }
    }
}

pub fn auto_quality(h: f32) -> f32 {
    if h <= 900.0 {
        3.0
    } else if h <= 1300.0 {
        4.0
    } else if h <= 1700.0 {
        4.5
    } else {
        6.0
    }
}


impl Settings {
    pub fn art_res(&self, physical_h: f32) -> f32 {
        let q = QUALITY[self.quality.min(QUALITY.len() - 1)].0;
        if q <= 0.0 {
            auto_quality(physical_h).min(if cfg!(target_arch = "wasm32") { 3.0 } else { f32::MAX })
        } else {
            q
        }
    }
    pub fn rules(&self) -> Rules {
        crate::tweaks::apply_rules(Rules {
            rounds_to_win: ROUNDS_TO_WIN[self.rounds.min(ROUNDS_TO_WIN.len() - 1)],
            round_ticks: ROUND_SECONDS[self.round_time.min(ROUND_SECONDS.len() - 1)] as i32 * 60,
            life_percent: LIFE_PERCENT[self.life.min(LIFE_PERCENT.len() - 1)],
            single_round: self.single_round,
            classic_pushbox: self.classic_pushbox,
            team_life_share: self.team_life_share,
            game_speed: GAME_SPEEDS[self.game_speed.min(GAME_SPEEDS.len() - 1)],
        })
    }
    pub fn fps_cap_value(&self) -> u32 {
        FPS_CAPS[self.fps_cap.min(FPS_CAPS.len() - 1)]
    }
    pub fn master_gain(&self) -> f32 {
        if self.mute {
            0.0
        } else {
            self.master as f32 / VOLUME_STEPS as f32
        }
    }
    pub fn music_gain(&self) -> f32 {
        self.music as f32 / VOLUME_STEPS as f32
    }
    pub fn sfx_gain(&self) -> f32 {
        self.sfx as f32 / VOLUME_STEPS as f32
    }

    pub fn apply_runtime(&self) {
        crate::i18n::set(self.lang);
        set_vsync(self.vsync);
        set_frame_cap(self.fps_cap_value());
    }
    /// Physical (pixel) height of the drawable area.
    pub fn physical_h() -> f32 {
        screen_height() * dpi_scale()
    }
}
