//! Budgeted sprite uploads into the render atlases.
//!
//! A fighter can carry tens of megabytes of sprite sheets (a Hyper-DBZ character is ~34 MB of index
//! art across 2510 sprites), so warming the atlas has to be paid in chunks across frames instead of
//! inside the single frame that starts a fight. The browser M.U.G.E.N port this engine is measured
//! against sidesteps the problem by giving every sprite its own texture and streaming it on first
//! draw; we keep the atlas and copy it a frame at a time.
use super::*;

impl Gfx {
    /// Uploads at most `budget` bytes of `keys` starting at `at`, stopping early when the atlas is at
    /// its occupancy cap. Returns the resume index (`keys.len()` once the art is fully resident).
    pub fn prefetch_step(
        &mut self,
        def: &CharDef,
        keys: &[(i32, i32)],
        at: usize,
        budget: u64,
    ) -> usize {
        const ATLAS_CAP: f32 = 0.5;
        let mut used = 0;
        for (i, key) in keys.iter().enumerate().skip(at) {
            let Some(spr) = self.sprite_tex(def, *key, &[]) else {
                continue;
            };
            if spr.resident() {
                continue;
            }
            let bytes = spr.atlas_bytes();
            if used > 0 && used + bytes > budget {
                return i;
            }
            if !spr.prefetch(ATLAS_CAP) {
                return keys.len();
            }
            used += bytes;
        }
        keys.len()
    }
}

impl SpriteTex {
    /// True once the sprite already lives in an atlas texture for the current atlas generation.
    pub fn resident(&self) -> bool {
        match self {
            SpriteTex::Rgba(t) => t.is_resident(),
            SpriteTex::Indexed(t, _) => t.is_resident(),
        }
    }

    /// Bytes this sprite still has to copy into its atlas (RGBA art is 4x an indexed sheet).
    pub fn atlas_bytes(&self) -> u64 {
        match self {
            SpriteTex::Rgba(t) => (t.width() * t.height() * 4.0) as u64,
            SpriteTex::Indexed(t, _) => t.pixels_len(),
        }
    }
}
