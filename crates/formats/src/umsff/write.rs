use super::{pixel_block, MAGIC, NONE, VERSION};
use crate::sff::DecodedSff;
use std::collections::HashMap;
use std::sync::Arc;

fn put(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    put(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
}

#[derive(Default)]
struct PaletteTable {
    ids: HashMap<Vec<u8>, u32>,
    list: Vec<Vec<u8>>,
}

impl PaletteTable {
    fn intern(&mut self, data: &Arc<Vec<u8>>) -> u32 {
        if let Some(&id) = self.ids.get(data.as_slice()) {
            return id;
        }
        let id = self.list.len() as u32;
        self.ids.insert(data.to_vec(), id);
        self.list.push(data.to_vec());
        id
    }
}

pub fn encode_umsff(sff: &DecodedSff) -> Result<Vec<u8>, String> {
    let mut table = PaletteTable::default();
    let mut palette_rows = Vec::with_capacity(sff.palettes.len() * 12);
    for palette in &sff.palettes {
        palette_rows.extend_from_slice(&palette.group.to_le_bytes());
        palette_rows.extend_from_slice(&palette.index.to_le_bytes());
        put(&mut palette_rows, table.intern(&palette.data));
    }
    let mut block_ids: HashMap<usize, u32> = HashMap::new();
    let mut blocks: Vec<Vec<u8>> = Vec::new();
    let mut sprite_rows = Vec::with_capacity(sff.sprites.len() * 36);
    for sprite in &sff.sprites {
        let block = match pixel_block(sprite.width, sprite.height, sprite)? {
            None => NONE,
            Some((key, bytes)) => *block_ids.entry(key).or_insert_with(|| {
                blocks.push(bytes);
                (blocks.len() - 1) as u32
            }),
        };
        let palette = sprite
            .palette
            .as_ref()
            .map_or(NONE, |data| table.intern(data));
        for value in [sprite.group, sprite.index, sprite.axis_x, sprite.axis_y] {
            sprite_rows.extend_from_slice(&value.to_le_bytes());
        }
        for value in [sprite.width, sprite.height] {
            put(&mut sprite_rows, value);
        }
        sprite_rows.extend_from_slice(&sprite.palette_index.to_le_bytes());
        put(&mut sprite_rows, palette);
        put(&mut sprite_rows, block);
    }
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&[VERSION, sff.version, 0, 0]);
    put(&mut out, table.list.len() as u32);
    for palette in &table.list {
        put_blob(&mut out, palette);
    }
    put(&mut out, sff.palettes.len() as u32);
    out.extend_from_slice(&palette_rows);
    put(&mut out, blocks.len() as u32);
    for block in &blocks {
        put_blob(&mut out, block);
    }
    put(&mut out, sff.sprites.len() as u32);
    out.extend_from_slice(&sprite_rows);
    Ok(out)
}
