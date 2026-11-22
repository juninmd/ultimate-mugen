//! SFF (Elecbyte sprite file) decoder: versions 1.01 (PCX subfiles) and
//! 2.00 / 2.01 (raw, RLE8, RLE5, LZ5, PNG8/24/32). Port of `src/mugen/sff.ts`
//! and `src/mugen/sprite.ts`; decompression follows Ikemen GO (image.go).

pub use crate::png::{decode_png, png_indexed_to_rgba, DecodedPng};
use crate::Reader;
use std::sync::Arc;

mod codecs;
mod v1;
use v1::decode_sff_v1;
pub use codecs::{decode_lz5, decode_rle5, decode_rle8};

#[derive(Clone, Debug)]
pub struct DecodedSprite {
    pub group: i32,
    pub index: i32,
    /// Axis (pivot): the sprite is drawn at `(x - axis_x, y - axis_y)`.
    pub axis_x: i32,
    pub axis_y: i32,
    pub width: u32,
    pub height: u32,
    /// Indexed pixels (8-bit) when paletted; `None` for true-color.
    pub indexed: Option<Arc<Vec<u8>>>,
    /// RGBA pixels for true-color (PNG 24/32, raw 24/32) sprites.
    pub rgba: Option<Arc<Vec<u8>>>,
    /// RGBA 256*4 palette used by this sprite (own or shared), when paletted.
    pub palette: Option<Arc<Vec<u8>>>,
    /// Index into [`DecodedSff::palettes`], or -1.
    pub palette_index: i32,
}

#[derive(Clone, Debug)]
pub struct Palette {
    pub group: i32,
    pub index: i32,
    /// RGBA 256*4.
    pub data: Arc<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct DecodedSff {
    /// 1 or 2.
    pub version: u8,
    pub sprites: Vec<DecodedSprite>,
    /// SFF v2 palette table, or SFF v1 per-sprite palettes (deduplicated).
    pub palettes: Vec<Palette>,
}

/// SFF v2 sprite formats.
pub mod format {
    pub const RAW: u8 = 0;
    pub const INVALID: u8 = 1;
    pub const RLE8: u8 = 2;
    pub const RLE5: u8 = 3;
    pub const LZ5: u8 = 4;
    pub const PNG8: u8 = 10;
    pub const PNG24: u8 = 11;
    pub const PNG32: u8 = 12;
}

const SIGNATURE: &[u8; 12] = b"ElecbyteSpr\0";

/// Convert a sprite into RGBA using `palette` (or its own palette); index 0
/// is transparent. True-color sprites return a copy of their RGBA data.
pub fn sprite_to_rgba(s: &DecodedSprite, palette: Option<&[u8]>) -> Vec<u8> {
    if let Some(rgba) = &s.rgba {
        return rgba.to_vec();
    }
    let n = s.width as usize * s.height as usize;
    if n == 0 {
        return Vec::new();
    }
    let pal: Option<&[u8]> = palette.or(s.palette.as_deref().map(|v| v.as_slice()));
    let (Some(idx), Some(pal)) = (&s.indexed, pal) else {
        return vec![0u8; n * 4];
    };
    let mut table = [0u32; 256];
    if pal.len() >= 1024 {
        for c in 1..256usize {
            let p = c * 4;
            table[c] = u32::from_ne_bytes([pal[p], pal[p + 1], pal[p + 2], pal[p + 3]]);
        }
    } else {
        for c in 1..256usize {
            let p = c * 4;
            let r = pal.get(p).copied().unwrap_or(0);
            let g = pal.get(p + 1).copied().unwrap_or(0);
            let b = pal.get(p + 2).copied().unwrap_or(0);
            table[c] = u32::from_ne_bytes([r, g, b, 255]);
        }
    }
    let mut out_u32 = vec![0u32; n];
    let limit = idx.len().min(n);
    for i in 0..limit {
        out_u32[i] = table[idx[i] as usize];
    }
    out_u32.iter().flat_map(|v| v.to_ne_bytes()).collect()
}

pub const MAX_SPRITE_PIXELS: usize = 1 << 24;

pub fn decode_sff(buf: &[u8]) -> Result<DecodedSff, String> {
    decode_sff_only(buf, |_, _| true)
}

pub fn decode_sff_only(
    buf: &[u8],
    wanted: impl Fn(i32, i32) -> bool,
) -> Result<DecodedSff, String> {
    if crate::umsff::is_umsff(buf) {
        return crate::umsff::decode_umsff(buf, wanted);
    }
    if buf.len() < SIGNATURE.len() || &buf[..12] != SIGNATURE {
        return Err("SFF: invalid signature (expected ElecbyteSpr)".into());
    }
    match buf.get(15).copied().unwrap_or(0) {
        1 => Ok(decode_sff_v1(buf, wanted)),
        2 => Ok(decode_sff_v2(buf, wanted)),
        v => Err(format!("SFF: unsupported version {v}")),
    }
}

fn empty_sprite(group: i32, index: i32, axis_x: i32, axis_y: i32) -> DecodedSprite {
    DecodedSprite {
        group,
        index,
        axis_x,
        axis_y,
        width: 0,
        height: 0,
        indexed: None,
        rgba: None,
        palette: None,
        palette_index: -1,
    }
}

// ---------------------------------------------------------------------------
// SFF v2
// ---------------------------------------------------------------------------

/// SFF v2 layout:
///   header: [0]sig(12) [12]verlo3 [13]verlo2 [14]verlo1 [15]verhi
///           [36]spriteListOffset [40]spriteCount [44]paletteListOffset
///           [48]paletteCount [52]ldataOffset [56]ldataLength
///           [60]tdataOffset [64]tdataLength
///   sprite node (28 bytes): group u16, index u16, w u16, h u16, axisX i16,
///     axisY i16, linkIndex u16, format u8, colorDepth u8, dataOffset u32,
///     dataLength u32, paletteIndex u16, flags u16 (bit0: 0=ldata, 1=tdata)
///   palette node (16 bytes): group u16, index u16, numColors u16,
///     linkIndex u16, dataOffset u32 (in ldata), dataLength u32 (0 => linked)
fn decode_sff_v2(buf: &[u8], wanted: impl Fn(i32, i32) -> bool) -> DecodedSff {
    let r = Reader(buf);
    let verlo2 = r.u8(13);
    let spr_ofs = r.u32(36) as usize;
    let spr_count = r.u32(40) as usize;
    let pal_ofs = r.u32(44) as usize;
    let pal_count = r.u32(48) as usize;
    let ldata = r.u32(52) as usize;
    let tdata = r.u32(60) as usize;

    // --- palettes
    let mut palettes: Vec<Palette> = Vec::with_capacity(pal_count.min(buf.len() / 16 + 1));
    for i in 0..pal_count {
        let o = pal_ofs.saturating_add(i.saturating_mul(16));
        if o.saturating_add(16) > buf.len() {
            break;
        }
        let (group, index, link) = (r.u16(o) as i32, r.u16(o + 2) as i32, r.u16(o + 6) as usize);
        let (d_ofs, d_len) = (r.u32(o + 8) as usize, r.u32(o + 12) as usize);
        let data = if d_len == 0 && link < palettes.len() {
            palettes[link].data.clone()
        } else {
            let mut data = vec![0u8; 1024];
            let start = ldata.saturating_add(d_ofs);
            let n = (d_len / 4).min(256);
            if start.saturating_add(n * 4) <= buf.len() {
                if verlo2 == 0 {
                    let src_chunks = buf[start..start + n * 4].as_chunks::<4>().0;
                    let dst_chunks = data[..n * 4].as_chunks_mut::<4>().0;
                    for (src, dst) in src_chunks.iter().zip(dst_chunks.iter_mut()) {
                        dst[0] = src[0];
                        dst[1] = src[1];
                        dst[2] = src[2];
                        dst[3] = 255;
                    }
                } else {
                    data[..n * 4].copy_from_slice(&buf[start..start + n * 4]);
                }
            } else {
                for c in 0..n {
                    let s = start.saturating_add(c * 4);
                    if s.saturating_add(3) >= buf.len() {
                        break;
                    }
                    data[c * 4..c * 4 + 3].copy_from_slice(&buf[s..s + 3]);
                    data[c * 4 + 3] = if verlo2 == 0 { 255 } else { buf[s + 3] };
                }
            }
            data[3] = 0;
            Arc::new(data)
        };
        palettes.push(Palette { group, index, data });
    }

    // --- sprites
    let mut sprites: Vec<DecodedSprite> = Vec::with_capacity(spr_count.min(buf.len() / 28 + 1));
    let mut links: Vec<Option<usize>> = Vec::with_capacity(sprites.capacity());
    let mut jobs: Vec<(usize, u8, u8, &[u8])> = Vec::with_capacity(spr_count.min(buf.len() / 28 + 1));
    for i in 0..spr_count {
        let o = spr_ofs.saturating_add(i.saturating_mul(28));
        if o.saturating_add(28) > buf.len() {
            break;
        }
        let link = r.u16(o + 12) as usize;
        let fmt = r.u8(o + 14);
        let depth = r.u8(o + 15);
        let (d_ofs, d_len) = (r.u32(o + 16) as usize, r.u32(o + 20) as usize);
        let pal_idx = r.u16(o + 24) as usize;
        let flags = r.u16(o + 26);
        let mut spr = empty_sprite(
            r.u16(o) as i32,
            r.u16(o + 2) as i32,
            r.i16(o + 8) as i32,
            r.i16(o + 10) as i32,
        );
        spr.width = r.u16(o + 4) as u32;
        spr.height = r.u16(o + 6) as u32;
        if !crate::pixels_within_limit(spr.width, spr.height) {
            spr.width = 0;
            spr.height = 0;
        }
        spr.palette_index = pal_idx as i32;
        if d_len == 0 {
            links.push(Some(link));
        } else {
            let start = (if flags & 1 == 0 { ldata } else { tdata }).saturating_add(d_ofs);
            let s = start.min(buf.len());
            let e = start.saturating_add(d_len).min(buf.len());
            jobs.push((i, fmt, depth, &buf[s..e.max(s)]));
            links.push(None);
        }
        sprites.push(spr);
    }
    for (i, link) in links.iter_mut().enumerate() {
        *link = link.filter(|&l| l != i && l < sprites.len());
    }
    let mut needed: Vec<bool> = sprites.iter().map(|s| wanted(s.group, s.index)).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..sprites.len() {
            if !needed[i] {
                continue;
            }
            if let Some(l) = links[i].filter(|&l| !needed[l]) {
                needed[l] = true;
                changed = true;
            }
        }
    }
    jobs.retain(|(i, ..)| needed[*i]);
    let decoded = crate::par::map(&jobs, |&(i, fmt, depth, data)| {
        let mut spr = empty_sprite(0, 0, 0, 0);
        spr.width = sprites[i].width;
        spr.height = sprites[i].height;
        spr.palette_index = sprites[i].palette_index;
        let has_palette = palettes.get(spr.palette_index as usize).is_some();
        decode_v2_image(&mut spr, fmt, depth, data, has_palette);
        spr
    });
    for ((i, ..), d) in jobs.iter().zip(decoded) {
        let spr = &mut sprites[*i];
        spr.width = d.width;
        spr.height = d.height;
        spr.indexed = d.indexed;
        spr.rgba = d.rgba;
        spr.palette = d.palette;
    }
    let mut waiting: Vec<usize> = (0..sprites.len()).filter(|&i| links[i].is_some()).collect();
    while !waiting.is_empty() {
        let mut progressed = false;
        waiting.retain(|&i| {
            let Some(l) = links[i] else { return false };
            let target = &sprites[l];
            if target.rgba.is_none() && target.indexed.is_none() {
                return true;
            }
            let (w, h, idx, rgba) = (target.width, target.height, target.indexed.clone(), target.rgba.clone());
            let spr = &mut sprites[i];
            spr.width = w;
            spr.height = h;
            spr.indexed = idx;
            spr.rgba = rgba;
            progressed = true;
            false
        });
        if !progressed {
            break;
        }
    }
    for i in 0..sprites.len() {
        let spr = &mut sprites[i];
        if spr.rgba.is_some() {
            spr.palette_index = -1;
        } else if let Some(p) = usize::try_from(spr.palette_index)
            .ok()
            .and_then(|i| palettes.get(i))
        {
            spr.palette = Some(p.data.clone());
        }
    }
    DecodedSff {
        version: 2,
        sprites,
        palettes,
    }
}

fn decode_v2_image(
    spr: &mut DecodedSprite,
    fmt: u8,
    depth: u8,
    data: &[u8],
    has_sff_palette: bool,
) {
    if (spr.width as usize) * (spr.height as usize) > MAX_SPRITE_PIXELS {
        spr.width = 0;
        spr.height = 0;
    }
    let n = spr.width as usize * spr.height as usize;
    // compressed formats are prefixed by a u32 uncompressed length
    let body = &data[data.len().min(4)..];
    match fmt {
        format::RAW => {
            if depth == 32 {
                let mut out = vec![0u8; n * 4];
                let k = (n * 4).min(data.len());
                out[..k].copy_from_slice(&data[..k]);
                spr.rgba = Some(Arc::new(out));
            } else if depth == 24 {
                let mut out = vec![0u32; n];
                let count = n.min(data.len() / 3);
                for (p, chunk) in data[..count * 3].chunks_exact(3).enumerate() {
                    out[p] = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], 255]);
                }
                let bytes: Vec<u8> = out.iter().flat_map(|v| v.to_ne_bytes()).collect();
                spr.rgba = Some(Arc::new(bytes));
            } else {
                let mut out = vec![0u8; n];
                let k = n.min(data.len());
                out[..k].copy_from_slice(&data[..k]);
                spr.indexed = Some(Arc::new(out));
            }
        }
        format::RLE8 => spr.indexed = Some(Arc::new(decode_rle8(body, n))),
        format::RLE5 => spr.indexed = Some(Arc::new(decode_rle5(body, n))),
        format::LZ5 => spr.indexed = Some(Arc::new(decode_lz5(body, n))),
        format::PNG8 | format::PNG24 | format::PNG32 => match decode_png(body) {
            Ok(png) => {
                spr.width = png.width;
                spr.height = png.height;
                if fmt == format::PNG8 && png.indexed.is_some() {
                    if !has_sff_palette {
                        spr.palette = png.palette.clone().map(Arc::new);
                    }
                    spr.indexed = png.indexed.map(Arc::new);
                } else if png.rgba.is_some() {
                    spr.rgba = png.rgba.map(Arc::new);
                } else {
                    spr.rgba = Some(Arc::new(png_indexed_to_rgba(&png)));
                }
            }
            // broken PNG: keep an empty (transparent) sprite instead of failing the file
            Err(_) => spr.indexed = Some(Arc::new(vec![0u8; n])),
        },
        // INVALID (1) or unknown: empty sprite
        _ => spr.indexed = Some(Arc::new(vec![0u8; n])),
    }
}

