use crate::lz4_sprite::{self, Pixels};
use crate::sff::DecodedSprite;
use std::sync::Arc;

mod read;
mod write;

pub use read::decode_umsff;
pub use write::encode_umsff;

pub const MAGIC: [u8; 4] = *b"UMSF";
const VERSION: u8 = 1;
const NONE: u32 = u32::MAX;
const MAX_DECODED_PIXELS: u64 = 1 << 29;

pub fn is_umsff(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && bytes[..4] == MAGIC
}

pub(crate) fn empty_sprite(source: &DecodedSprite) -> DecodedSprite {
    DecodedSprite {
        group: source.group,
        index: source.index,
        axis_x: source.axis_x,
        axis_y: source.axis_y,
        width: 0,
        height: 0,
        indexed: None,
        rgba: None,
        palette: None,
        palette_index: -1,
    }
}

pub(crate) fn pointer(data: &Arc<Vec<u8>>) -> usize {
    Arc::as_ptr(data) as usize
}

pub(crate) fn pixel_block(
    width: u32,
    height: u32,
    sprite: &DecodedSprite,
) -> Result<Option<(usize, Vec<u8>)>, String> {
    match (&sprite.indexed, &sprite.rgba) {
        (Some(indices), _) => Ok(Some((
            pointer(indices),
            lz4_sprite::encode_indexed(width, height, indices, &[])?,
        ))),
        (None, Some(rgba)) => Ok(Some((
            pointer(rgba),
            lz4_sprite::encode_rgba(width, height, rgba)?,
        ))),
        _ => Ok(None),
    }
}

pub(crate) fn block_pixels(bytes: &[u8]) -> Result<(bool, Arc<Vec<u8>>), String> {
    match lz4_sprite::decode(bytes)?.pixels {
        Pixels::Indexed { indices, .. } => Ok((true, Arc::new(indices))),
        Pixels::Rgba(rgba) => Ok((false, Arc::new(rgba))),
    }
}
