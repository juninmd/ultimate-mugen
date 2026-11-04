//! Minimal PNG decoder (port of `decodePng` in `src/mugen/sff.ts`).
//!
//! Non-interlaced only. Color types 0 (gray), 2 (RGB), 3 (indexed, bit depth
//! 1/2/4/8), 4 (gray+alpha) and 6 (RGBA); 16-bit samples are reduced to their
//! high byte. All five filter types.

use crate::Reader;

#[derive(Clone, Debug)]
pub struct DecodedPng {
    pub width: u32,
    pub height: u32,
    pub color_type: u8,
    pub bit_depth: u8,
    /// Color type 3: one palette index per pixel.
    pub indexed: Option<Vec<u8>>,
    /// Color type 3: RGBA 256*4 from PLTE (+tRNS).
    pub palette: Option<Vec<u8>>,
    /// Other color types: straight (non-premultiplied) RGBA.
    pub rgba: Option<Vec<u8>>,
}

const PNG_SIG: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

/// zlib inflate into a buffer of exactly `expected` bytes (the Adler-32
/// checksum is not verified; trailing extra output is ignored). Errors when
/// the stream yields fewer than `expected` bytes.
pub(crate) fn inflate_zlib_exact(z: &[u8], expected: usize) -> Result<Vec<u8>, String> {
    use miniz_oxide::inflate::core::{decompress, inflate_flags, DecompressorOxide};
    let mut out = vec![0u8; expected];
    let mut st = DecompressorOxide::new();
    let flags = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
    let (status, _, written) = decompress(&mut st, z, &mut out, 0, flags);
    if written < expected {
        return Err(format!("truncated image data (zlib: {status:?})"));
    }
    Ok(out)
}

pub fn decode_png(buf: &[u8]) -> Result<DecodedPng, String> {
    if buf.len() < 8 || buf[..8] != PNG_SIG {
        return Err("PNG: bad signature".into());
    }
    let r = Reader(buf);
    let mut p = 8usize;
    let (mut width, mut height, mut bit_depth, mut color_type, mut interlace) = (0u32, 0u32, 0u8, 0u8, 0u8);
    let mut plte: Option<&[u8]> = None;
    let mut trns: Option<&[u8]> = None;
    let mut z: Vec<u8> = Vec::new();
    while p + 8 <= buf.len() {
        let len = r.u32be(p) as usize;
        let ty = &buf[p + 4..p + 8];
        let data = &buf[p + 8..(p + 8).saturating_add(len).min(buf.len())];
        match ty {
            b"IHDR" => {
                width = r.u32be(p + 8);
                height = r.u32be(p + 12);
                bit_depth = r.u8(p + 16);
                color_type = r.u8(p + 17);
                interlace = r.u8(p + 20);
            }
            b"PLTE" => plte = Some(data),
            b"tRNS" => trns = Some(data),
            b"IDAT" => z.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
        p = p.saturating_add(12).saturating_add(len);
    }
    if width == 0 || height == 0 {
        return Err("PNG: missing IHDR".into());
    }
    if !crate::pixels_within_limit(width, height) {
        return Err(format!("PNG: {width}x{height} exceeds the pixel limit"));
    }
    if interlace != 0 {
        return Err("PNG: interlaced images are not supported".into());
    }
    let channels: usize = match color_type {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        6 => 4,
        _ => return Err(format!("PNG: unsupported color type {color_type}")),
    };
    let depth_ok = if color_type == 3 { matches!(bit_depth, 1 | 2 | 4 | 8) } else { matches!(bit_depth, 8 | 16) };
    if !depth_ok {
        return Err(format!("PNG: unsupported bit depth {bit_depth} for color type {color_type}"));
    }

    if (width as u64) * (height as u64) > crate::sff::MAX_SPRITE_PIXELS as u64 {
        return Err("PNG: image too large".into());
    }
    let (w, h) = (width as usize, height as usize);
    let bits_pp = channels * bit_depth as usize;
    let bpp = (bits_pp >> 3).max(1); // filter byte distance
    let stride = (width as u64 * bits_pp as u64).div_ceil(8);
    let expected = (stride + 1).saturating_mul(height as u64);
    // deflate cannot expand more than ~1032:1: reject before allocating
    if expected > (z.len() as u64).saturating_mul(1100).saturating_add(4096) || expected > usize::MAX as u64 / 2 {
        return Err("PNG: truncated image data".into());
    }
    let (stride, expected) = (stride as usize, expected as usize);
    let mut img = inflate_zlib_exact(&z, expected).map_err(|e| format!("PNG: {e}"))?;
    // unfilter each row in place, then compact it to `y * stride`
    let zero = vec![0u8; stride];
    for y in 0..h {
        let ro = y * (stride + 1);
        let ft = img[ro];
        let (head, tail) = img.split_at_mut(ro + 1);
        let prev: &[u8] = if y == 0 { &zero } else { &head[(y - 1) * stride..y * stride] };
        unfilter(ft, &mut tail[..stride], prev, bpp)?;
        img.copy_within(ro + 1..ro + 1 + stride, y * stride);
    }
    img.truncate(stride * h);

    let n = w * h;
    if color_type == 3 {
        let indexed = if bit_depth == 8 {
            img.truncate(n);
            img
        } else {
            let bd = bit_depth as usize;
            let per_byte = 8 / bd;
            let mask = ((1u16 << bd) - 1) as u8;
            let mut out = vec![0u8; n];
            for y in 0..h {
                let row = &img[y * stride..(y + 1) * stride];
                let dst = &mut out[y * w..(y + 1) * w];
                for (x, d) in dst.iter_mut().enumerate() {
                    let b = row[x / per_byte];
                    let shift = 8 - bd * ((x % per_byte) + 1);
                    *d = (b >> shift) & mask;
                }
            }
            out
        };
        let palette = plte.map(|plte| {
            let mut pal = vec![0u8; 1024];
            let cnt = (plte.len() / 3).min(256);
            for c in 0..cnt {
                pal[c * 4..c * 4 + 3].copy_from_slice(&plte[c * 3..c * 3 + 3]);
                pal[c * 4 + 3] = trns.and_then(|t| t.get(c).copied()).unwrap_or(255);
            }
            pal
        });
        return Ok(DecodedPng { width, height, color_type, bit_depth, indexed: Some(indexed), palette, rgba: None });
    }

    if bit_depth == 8 && color_type == 6 {
        img.truncate(n * 4);
        return Ok(DecodedPng { width, height, color_type, bit_depth, indexed: None, palette: None, rgba: Some(img) });
    }
    let mut rgba = vec![0u8; n * 4];
    let bs = if bit_depth == 16 { 2 } else { 1 }; // bytes per sample; take high byte
    // tRNS for gray/RGB: single transparent color
    let tr = trns.filter(|_| color_type == 0 || color_type == 2);
    for y in 0..h {
        let row = y * stride;
        for x in 0..w {
            let s = row + x * channels * bs;
            let d = (y * w + x) * 4;
            let (rr, gg, bb, mut aa);
            match color_type {
                0 => {
                    rr = img[s];
                    gg = rr;
                    bb = rr;
                    aa = 255;
                    if let Some(t) = tr {
                        if t.len() >= 2 && (if bs == 1 { img[s] == t[1] } else { img[s] == t[0] && img[s + 1] == t[1] }) {
                            aa = 0;
                        }
                    }
                }
                4 => {
                    rr = img[s];
                    gg = rr;
                    bb = rr;
                    aa = img[s + bs];
                }
                2 => {
                    rr = img[s];
                    gg = img[s + bs];
                    bb = img[s + 2 * bs];
                    aa = 255;
                    // tRNS holds 16-bit BE samples: low byte at 8-bit depth, high byte at 16-bit
                    if let Some(t) = tr {
                        let hi = if bs == 2 { 0 } else { 1 };
                        if t.len() >= 6 && rr == t[hi] && gg == t[2 + hi] && bb == t[4 + hi] {
                            aa = 0;
                        }
                    }
                }
                _ => {
                    rr = img[s];
                    gg = img[s + bs];
                    bb = img[s + 2 * bs];
                    aa = img[s + 3 * bs];
                }
            }
            rgba[d] = rr;
            rgba[d + 1] = gg;
            rgba[d + 2] = bb;
            rgba[d + 3] = aa;
        }
    }
    Ok(DecodedPng { width, height, color_type, bit_depth, indexed: None, palette: None, rgba: Some(rgba) })
}

fn unfilter(ft: u8, line: &mut [u8], prev: &[u8], bpp: usize) -> Result<(), String> {
    let len = line.len();
    let prev = &prev[..len];
    match ft {
        0 => {}
        1 => {
            for x in bpp..len {
                line[x] = line[x].wrapping_add(line[x - bpp]);
            }
        }
        2 => {
            for (l, &p) in line.iter_mut().zip(prev) {
                *l = l.wrapping_add(p);
            }
        }
        3 => {
            for x in 0..len {
                let a = if x >= bpp { line[x - bpp] as u16 } else { 0 };
                line[x] = line[x].wrapping_add(((a + prev[x] as u16) >> 1) as u8);
            }
        }
        4 => {
            for x in 0..len {
                let a = if x >= bpp { line[x - bpp] as i16 } else { 0 };
                let b = prev[x] as i16;
                let c = if x >= bpp { prev[x - bpp] as i16 } else { 0 };
                let pp = a + b - c;
                let (pa, pb, pc) = ((pp - a).abs(), (pp - b).abs(), (pp - c).abs());
                let pred = if pa <= pb && pa <= pc {
                    a
                } else if pb <= pc {
                    b
                } else {
                    c
                };
                line[x] = line[x].wrapping_add(pred as u8);
            }
        }
        _ => return Err(format!("PNG: bad filter type {ft}")),
    }
    Ok(())
}

/// Expand an indexed PNG through its own palette (alpha from tRNS).
pub fn png_indexed_to_rgba(png: &DecodedPng) -> Vec<u8> {
    let n = png.width as usize * png.height as usize;
    let mut out = vec![0u8; n * 4];
    if let (Some(idx), Some(pal)) = (&png.indexed, &png.palette) {
        for (o, &c) in out.chunks_exact_mut(4).zip(idx.iter()) {
            o.copy_from_slice(&pal[c as usize * 4..c as usize * 4 + 4]);
        }
    }
    out
}
