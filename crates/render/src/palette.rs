//! Rows of the palette atlas.
//!
//! Every indexed sprite is coloured by looking up a 256-colour row in one texture (`PALETTE_ROWS`
//! rows of 4 bytes each). The atlas is small on purpose: it is uploaded once and sampled by index
//! from the sprite shader. The rows are a scarce resource, so identical palettes must share one:
//! a mirror match, a `RemapPal` that resolves to a palette another fighter already uses, or the
//! same fighter re-warmed after a cache flush would otherwise each burn a row and force a full
//! palette upload for pixels that are already resident.

use std::collections::HashMap;

pub type PaletteColors = [[u8; 4]; 256];

#[derive(Default)]
pub struct PaletteRows {
    rows: HashMap<PaletteColors, u32>,
    used: u32,
}

impl PaletteRows {
    pub fn new() -> Self {
        PaletteRows::default()
    }

    /// Row holding `colors`, allocating and reporting `true` when it has to be uploaded first.
    /// `None` means the atlas is full and the caller must reset it.
    pub fn row(&mut self, colors: &PaletteColors) -> Option<(u32, bool)> {
        if let Some(&row) = self.rows.get(colors) {
            return Some((row, false));
        }
        if self.used >= crate::PALETTE_ROWS {
            return None;
        }
        let row = self.used;
        self.used += 1;
        self.rows.insert(*colors, row);
        Some((row, true))
    }

    /// Rows handed out since the last `clear`.
    pub fn used(&self) -> u32 {
        self.used
    }

    pub fn clear(&mut self) {
        if self.used > 0 || !self.rows.is_empty() {
            self.rows.clear();
            self.used = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colors(seed: u32) -> PaletteColors {
        let mut c = [[0u8; 4]; 256];
        for (i, px) in c.iter_mut().enumerate() {
            *px = [(seed & 0xff) as u8, (seed >> 8) as u8, i as u8, 255];
        }
        c
    }

    #[test]
    fn identical_palettes_share_one_row() {
        let mut rows = PaletteRows::new();
        assert_eq!(
            rows.row(&colors(1)),
            Some((0, true)),
            "first palette is uploaded"
        );
        for _ in 0..50 {
            assert_eq!(
                rows.row(&colors(1)),
                Some((0, false)),
                "same palette, already resident"
            );
        }
        assert_eq!(rows.used(), 1);
    }

    #[test]
    fn distinct_palettes_get_distinct_rows() {
        let mut rows = PaletteRows::new();
        for seed in 0..8 {
            assert_eq!(rows.row(&colors(seed)), Some((seed, true)));
        }
        assert_eq!(rows.used(), 8);
        assert_eq!(rows.row(&colors(3)), Some((3, false)));
        assert_eq!(rows.used(), 8);
    }

    #[test]
    fn a_full_atlas_reports_failure_instead_of_overwriting() {
        let mut rows = PaletteRows::new();
        for seed in 0..crate::PALETTE_ROWS {
            assert!(rows.row(&colors(seed)).is_some(), "row {seed} fits");
        }
        assert_eq!(rows.used(), crate::PALETTE_ROWS);
        assert!(
            rows.row(&colors(crate::PALETTE_ROWS)).is_none(),
            "no room left"
        );
        assert_eq!(
            rows.row(&colors(5)),
            Some((5, false)),
            "resident palettes still work"
        );
    }

    #[test]
    fn clear_frees_every_row() {
        let mut rows = PaletteRows::new();
        for seed in 0..4 {
            rows.row(&colors(seed));
        }
        assert_eq!(rows.used(), 4);
        rows.clear();
        assert_eq!(rows.used(), 0);
        assert_eq!(rows.row(&colors(0)), Some((0, true)));
    }
}
