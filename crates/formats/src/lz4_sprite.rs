pub const MAGIC: [u8; 4] = *b"UML1";
pub const MAX_SIDE: usize = 8192;
pub const MAX_PIXELS: usize = 16 << 20;
const HEADER: usize = 19;
const KIND_RGBA: u8 = 0;
const KIND_INDEXED: u8 = 1;

pub enum Pixels {
    Rgba(Vec<u8>),
    Indexed { indices: Vec<u8>, palette: Vec<u8> },
}

pub struct LzSprite {
    pub width: u32,
    pub height: u32,
    pub pixels: Pixels,
}

struct Bounds {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

pub fn is_lz4_sprite(bytes: &[u8]) -> bool {
    bytes.len() >= HEADER && bytes[..4] == MAGIC
}

fn check_size(width: usize, height: usize) -> Result<(), String> {
    if width > MAX_SIDE || height > MAX_SIDE || width * height > MAX_PIXELS {
        return Err(format!("lz4 sprite: {width}x{height} is over the limit"));
    }
    Ok(())
}

fn bounds(data: &[u8], width: usize, height: usize, bpp: usize) -> Bounds {
    let blank = |row: usize, col: usize| {
        data[(row * width + col) * bpp..][..bpp]
            .iter()
            .all(|&byte| byte == 0)
    };
    let rows: Vec<usize> = (0..height)
        .filter(|&row| (0..width).any(|col| !blank(row, col)))
        .collect();
    let (Some(&top), Some(&bottom)) = (rows.first(), rows.last()) else {
        return Bounds {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
    };
    let first = (top..=bottom)
        .filter_map(|row| (0..width).find(|&col| !blank(row, col)))
        .min()
        .unwrap_or(0);
    let last = (top..=bottom)
        .filter_map(|row| (0..width).rev().find(|&col| !blank(row, col)))
        .max()
        .unwrap_or(0);
    Bounds {
        x: first,
        y: top,
        w: last - first + 1,
        h: bottom - top + 1,
    }
}

fn encode(
    kind: u8,
    width: usize,
    height: usize,
    data: &[u8],
    palette: &[u8],
    bpp: usize,
) -> Result<Vec<u8>, String> {
    check_size(width, height)?;
    if data.len() != width * height * bpp {
        return Err(format!(
            "lz4 sprite: {} bytes for {width}x{height}",
            data.len()
        ));
    }
    if !palette.len().is_multiple_of(4) || palette.len() > 1024 {
        return Err("lz4 sprite: palette must be 0 to 256 RGBA colors".into());
    }
    let area = bounds(data, width, height, bpp);
    let mut out = Vec::with_capacity(HEADER + palette.len() + data.len() / 2);
    out.extend_from_slice(&MAGIC);
    out.push(kind);
    out.extend_from_slice(&((palette.len() / 4) as u16).to_le_bytes());
    for value in [width, height, area.x, area.y, area.w, area.h] {
        out.extend_from_slice(&(value as u16).to_le_bytes());
    }
    out.extend_from_slice(palette);
    if area.w == 0 {
        return Ok(out);
    }
    let mut cropped = Vec::with_capacity(area.w * area.h * bpp);
    for row in area.y..area.y + area.h {
        cropped.extend_from_slice(&data[(row * width + area.x) * bpp..][..area.w * bpp]);
    }
    out.extend_from_slice(&lz4_flex::block::compress(&cropped));
    Ok(out)
}

pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    encode(KIND_RGBA, width as usize, height as usize, rgba, &[], 4)
}

pub fn encode_indexed(
    width: u32,
    height: u32,
    indices: &[u8],
    palette: &[u8],
) -> Result<Vec<u8>, String> {
    encode(
        KIND_INDEXED,
        width as usize,
        height as usize,
        indices,
        palette,
        1,
    )
}

fn field(bytes: &[u8], at: usize) -> usize {
    u16::from_le_bytes([bytes[at], bytes[at + 1]]) as usize
}

pub fn decode(bytes: &[u8]) -> Result<LzSprite, String> {
    if !is_lz4_sprite(bytes) {
        return Err("lz4 sprite: bad header".into());
    }
    let kind = bytes[4];
    let colors = field(bytes, 5);
    let (width, height) = (field(bytes, 7), field(bytes, 9));
    let area = Bounds {
        x: field(bytes, 11),
        y: field(bytes, 13),
        w: field(bytes, 15),
        h: field(bytes, 17),
    };
    check_size(width, height)?;
    let bpp = match kind {
        KIND_RGBA if colors == 0 => 4,
        KIND_INDEXED if colors <= 256 => 1,
        _ => return Err("lz4 sprite: unknown kind".into()),
    };
    if area.x + area.w > width || area.y + area.h > height {
        return Err("lz4 sprite: crop outside the canvas".into());
    }
    let palette_end = HEADER + colors * 4;
    let palette = bytes
        .get(HEADER..palette_end)
        .ok_or("lz4 sprite: truncated palette")?
        .to_vec();
    let mut canvas = vec![0u8; width * height * bpp];
    if area.w > 0 {
        let packed = &bytes[palette_end..];
        if area.w == width && area.h == height {
            expect_len(
                lz4_flex::block::decompress_into(packed, &mut canvas),
                canvas.len(),
            )?;
        } else {
            let mut cropped = vec![0u8; area.w * area.h * bpp];
            expect_len(
                lz4_flex::block::decompress_into(packed, &mut cropped),
                cropped.len(),
            )?;
            for (row, line) in cropped.chunks_exact(area.w * bpp).enumerate() {
                canvas[((area.y + row) * width + area.x) * bpp..][..line.len()]
                    .copy_from_slice(line);
            }
        }
    }
    let pixels = if kind == KIND_RGBA {
        Pixels::Rgba(canvas)
    } else {
        Pixels::Indexed {
            indices: canvas,
            palette,
        }
    };
    Ok(LzSprite {
        width: width as u32,
        height: height as u32,
        pixels,
    })
}

fn expect_len<E: std::fmt::Display>(
    result: Result<usize, E>,
    expected: usize,
) -> Result<(), String> {
    match result {
        Ok(len) if len == expected => Ok(()),
        Ok(len) => Err(format!(
            "lz4 sprite: block holds {len} bytes, expected {expected}"
        )),
        Err(error) => Err(format!("lz4 sprite: {error}")),
    }
}

pub fn expand_to_rgba(sprite: LzSprite) -> (u32, u32, Vec<u8>) {
    let LzSprite {
        width,
        height,
        pixels,
    } = sprite;
    match pixels {
        Pixels::Rgba(rgba) => (width, height, rgba),
        Pixels::Indexed { indices, palette } => {
            let mut table = [0u32; 256];
            for (slot, color) in table.iter_mut().zip(palette.chunks_exact(4)) {
                *slot = u32::from_le_bytes([color[0], color[1], color[2], color[3]]);
            }
            let mut rgba = Vec::with_capacity(indices.len() * 4);
            for &index in &indices {
                rgba.extend_from_slice(&table[index as usize].to_le_bytes());
            }
            (width, height, rgba)
        }
    }
}

pub fn decode_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    decode(bytes).map(expand_to_rgba)
}
