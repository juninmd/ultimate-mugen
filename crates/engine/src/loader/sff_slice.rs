//! Browser SFF decoding: only the sprites the animations actually draw.
//!
//! A M.U.G.E.N. pack sold as a single 25 MB `.sff` can hold thousands of sprites while the character draws a few
//! hundred. Decoding every PCX/PNG subfile in one frame cost 0.5–1.2 s of frozen loading screen on the browser
//! (the desktop decodes in parallel threads and does not care). The animations are plain text and cheap to parse,
//! so they go first and tell the decoder which sprite keys are worth decoding.
use std::collections::{HashMap, HashSet};
use ume_formats::sff::DecodedSff;
use crate::chardef::Animation;

/// Sprite keys the character draws, plus the 9000,0 icon every pack carries.
pub fn wanted_keys(anims: &HashMap<i32, Animation>) -> HashSet<(i32, i32)> {
    let mut needed: HashSet<(i32, i32)> = HashSet::new();
    for anim in anims.values() {
        for frame in &anim.frames {
            needed.insert((frame.group, frame.index));
        }
    }
    if !needed.is_empty() {
        needed.insert((9000, 0));
    }
    needed
}

/// Decodes `buf`, keeping only the sprites referenced by `anims` (plus the icon and every sprite when the
/// animations could not be parsed). Paletted sprites resolve their palette exactly like the full decode: the walk
/// still visits every subfile, it only skips the pixel payload of the ones nobody draws.
pub fn decode_for_anims(buf: &[u8], anims: &HashMap<i32, Animation>) -> Result<DecodedSff, String> {
    let needed = wanted_keys(anims);
    let mut sff = match super::sff_cache::decode_cached(buf, &needed) {
        Ok(sff) => sff,
        Err(error) => return Err(error),
    };
    sff.sprites.retain(|s| s.width > 0 || s.height > 0 || s.indexed.is_some() || s.rgba.is_some());
    Ok(sff)
}
