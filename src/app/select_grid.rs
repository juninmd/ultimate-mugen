use super::*;
use crate::settings::GRID_PRESETS;

impl App {
    fn grid_preset(&self) -> (&'static str, usize, usize, usize) {
        GRID_PRESETS[self.settings.grid.min(GRID_PRESETS.len() - 1)]
    }

    pub(crate) fn grid_rows(&self) -> usize {
        self.grid_preset().1
    }

    pub(crate) fn grid_cols(&self, count: usize) -> usize {
        let (_, rows, min, max) = self.grid_preset();
        count.div_ceil(rows).clamp(min, max)
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
