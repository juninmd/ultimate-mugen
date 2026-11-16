use super::*;
use crate::settings::GRID_PRESETS;

pub(crate) const GRID_W: f32 = 640.0;
pub(crate) const GRID_GAP: f32 = 6.0;
pub(crate) const GRID_Y: f32 = 98.0;
const TILE_MAX: f32 = 96.0;
const TILE_ASPECT: f32 = 1.17;
const GRID_BOTTOM: f32 = 628.0;
const GRID_BOTTOM_TEAM: f32 = 578.0;

impl App {
    fn grid_preset(&self) -> (&'static str, usize, usize, usize) {
        GRID_PRESETS[self.settings.grid.min(GRID_PRESETS.len() - 1)]
    }

    pub(crate) fn grid_tile(cols: usize) -> (f32, f32) {
        let cols = cols.max(1) as f32;
        let cw = ((GRID_W - (cols - 1.0) * GRID_GAP) / cols).min(TILE_MAX);
        (cw, cw * TILE_ASPECT)
    }

    pub(crate) fn grid_rows(&self) -> usize {
        let preset_rows = self.grid_preset().1;
        let (_, ch) = Self::grid_tile(self.grid_cols(super::select_view::last_view_len()));
        let bottom = if self.mode.is_team() {
            GRID_BOTTOM_TEAM
        } else {
            GRID_BOTTOM
        };
        let fit = ((bottom - GRID_Y + GRID_GAP) / (ch + GRID_GAP)).floor() as usize;
        preset_rows.min(fit.max(1))
    }

    pub(crate) fn grid_cols(&self, count: usize) -> usize {
        let (_, rows, min, max) = self.grid_preset();
        count.max(1).div_ceil(rows).clamp(min, max)
    }

    pub(crate) fn grid_label(&self) -> &'static str {
        self.grid_preset().0
    }

    pub(crate) fn cycle_grid(&mut self) {
        self.settings.grid = (self.settings.grid + 1) % GRID_PRESETS.len();
        self.settings.save();
        self.audio.sfx("select");
    }
}
