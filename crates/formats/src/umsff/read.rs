use super::{block_pixels, empty_sprite, is_umsff, MAX_DECODED_PIXELS, NONE, VERSION};
use crate::sff::{DecodedSff, DecodedSprite, Palette};
use std::collections::HashMap;
use std::sync::Arc;

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .at
            .checked_add(len)
            .filter(|&end| end <= self.bytes.len())
            .ok_or("UMSF: truncated")?;
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn word(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn signed(&mut self) -> Result<i32, String> {
        self.word().map(|value| value as i32)
    }

    fn count(&mut self, row: usize) -> Result<usize, String> {
        let count = self.word()? as usize;
        if count > (self.bytes.len() - self.at) / row.max(1) {
            return Err("UMSF: count does not fit the file".into());
        }
        Ok(count)
    }
}

struct Row {
    group: i32,
    index: i32,
    axis_x: i32,
    axis_y: i32,
    width: u32,
    height: u32,
    palette_index: i32,
    palette: u32,
    block: u32,
}

fn row(cursor: &mut Cursor) -> Result<Row, String> {
    Ok(Row {
        group: cursor.signed()?,
        index: cursor.signed()?,
        axis_x: cursor.signed()?,
        axis_y: cursor.signed()?,
        width: cursor.word()?,
        height: cursor.word()?,
        palette_index: cursor.signed()?,
        palette: cursor.word()?,
        block: cursor.word()?,
    })
}

fn blobs<'a>(cursor: &mut Cursor<'a>) -> Result<Vec<&'a [u8]>, String> {
    let count = cursor.count(4)?;
    (0..count)
        .map(|_| {
            let len = cursor.word()? as usize;
            cursor.take(len)
        })
        .collect()
}

pub fn decode_umsff(bytes: &[u8], wanted: impl Fn(i32, i32) -> bool) -> Result<DecodedSff, String> {
    if !is_umsff(bytes) || bytes[4] != VERSION {
        return Err("UMSF: bad header or version".into());
    }
    let mut cursor = Cursor { bytes, at: 8 };
    let palettes: Vec<Arc<Vec<u8>>> = blobs(&mut cursor)?
        .into_iter()
        .map(|data| Arc::new(data.to_vec()))
        .collect();
    let palette_count = cursor.count(12)?;
    let mut entries = Vec::with_capacity(palette_count);
    for _ in 0..palette_count {
        let (group, index, id) = (cursor.signed()?, cursor.signed()?, cursor.word()? as usize);
        let data = palettes
            .get(id)
            .ok_or("UMSF: palette reference out of range")?
            .clone();
        entries.push(Palette { group, index, data });
    }
    let blocks = blobs(&mut cursor)?;
    let sprite_count = cursor.count(36)?;
    let rows: Vec<Row> = (0..sprite_count)
        .map(|_| row(&mut cursor))
        .collect::<Result<_, _>>()?;
    let mut needed: Vec<u32> = rows
        .iter()
        .filter(|r| r.block != NONE && wanted(r.group, r.index))
        .map(|r| r.block)
        .collect();
    needed.sort_unstable();
    needed.dedup();
    let declared: u64 = rows
        .iter()
        .filter(|r| r.block != NONE && wanted(r.group, r.index))
        .map(|r| r.width as u64 * r.height as u64)
        .sum();
    if declared > MAX_DECODED_PIXELS {
        return Err("UMSF: declared sprite sizes are over the budget".into());
    }
    let results = crate::par::map(&needed, |&id| {
        blocks
            .get(id as usize)
            .ok_or_else(|| "UMSF: block reference out of range".to_string())
            .and_then(|b| block_pixels(b))
    });
    let mut decoded: HashMap<u32, (bool, Arc<Vec<u8>>)> = HashMap::with_capacity(needed.len());
    for (id, result) in needed.iter().zip(results) {
        decoded.insert(*id, result?);
    }
    let sprites = rows
        .iter()
        .map(|r| sprite(r, &palettes, &decoded, &wanted))
        .collect::<Result<_, _>>()?;
    Ok(DecodedSff {
        version: bytes[5],
        sprites,
        palettes: entries,
    })
}

fn sprite(
    r: &Row,
    palettes: &[Arc<Vec<u8>>],
    decoded: &HashMap<u32, (bool, Arc<Vec<u8>>)>,
    wanted: &impl Fn(i32, i32) -> bool,
) -> Result<DecodedSprite, String> {
    let mut sprite = DecodedSprite {
        group: r.group,
        index: r.index,
        axis_x: r.axis_x,
        axis_y: r.axis_y,
        width: 0,
        height: 0,
        indexed: None,
        rgba: None,
        palette: None,
        palette_index: -1,
    };
    if !wanted(r.group, r.index) {
        return Ok(empty_sprite(&sprite));
    }
    sprite.width = r.width;
    sprite.height = r.height;
    sprite.palette_index = r.palette_index;
    if r.palette != NONE {
        sprite.palette = Some(
            palettes
                .get(r.palette as usize)
                .ok_or("UMSF: palette reference out of range")?
                .clone(),
        );
    }
    if r.block != NONE {
        let (indexed, pixels) = decoded.get(&r.block).ok_or("UMSF: missing block")?;
        let pixel_bytes = if *indexed { 1 } else { 4 };
        if pixels.len() != r.width as usize * r.height as usize * pixel_bytes {
            return Err("UMSF: block size does not match the sprite".into());
        }
        if *indexed {
            sprite.indexed = Some(pixels.clone())
        } else {
            sprite.rgba = Some(pixels.clone())
        }
    }
    Ok(sprite)
}
