//! SFF (Elecbyte sprite file) decoder: versions 1.01 (PCX subfiles) and
//! 2.00 / 2.01 (raw, RLE8, RLE5, LZ5, PNG8/24/32). Port of `src/mugen/sff.ts`
//! and `src/mugen/sprite.ts`; decompression follows Ikemen GO (image.go).

use crate::pcx::{
    decode_pcx_rle, find_pcx_palette, raw_rows, read_pcx_header, read_rgb_palette, PCX_HEADER_SIZE,
};
pub use crate::png::{decode_png, png_indexed_to_rgba, DecodedPng};
use crate::Reader;
use std::collections::HashMap;
use std::sync::Arc;

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
    let mut out = vec![0u8; n * 4];
    let pal: Option<&[u8]> = palette.or(s.palette.as_deref().map(|v| v.as_slice()));
    let (Some(idx), Some(pal)) = (&s.indexed, pal) else {
        return out;
    };
    if pal.len() >= 1024 {
        for (o, &c) in out.as_chunks_mut::<4>().0.iter_mut().zip(idx.iter()) {
            if c != 0 {
                let p = c as usize * 4;
                o[0] = pal[p];
                o[1] = pal[p + 1];
                o[2] = pal[p + 2];
                o[3] = 255;
            }
        }
    } else {
        for (o, &c) in out.as_chunks_mut::<4>().0.iter_mut().zip(idx.iter()) {
            if c != 0 {
                let p = c as usize * 4;
                for (k, v) in o[..3].iter_mut().enumerate() {
                    *v = pal.get(p + k).copied().unwrap_or(0);
                }
                o[3] = 255;
            }
        }
    }
    out
}

pub const MAX_SPRITE_PIXELS: usize = 1 << 24;

pub fn decode_sff(buf: &[u8]) -> Result<DecodedSff, String> {
    decode_sff_only(buf, |_, _| true)
}

pub fn decode_sff_only(
    buf: &[u8],
    wanted: impl Fn(i32, i32) -> bool,
) -> Result<DecodedSff, String> {
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
// SFF v1
// ---------------------------------------------------------------------------

/// SFF v1 layout:
///   header: [0]sig(12) [12]ver(4) [16]numGroups u32 [20]numImages u32
///           [24]firstSubfileOffset u32 [28]subheaderSize u32 [32]paletteType u8
///   subfile: [0]next u32 [4]length u32 [8]axisX i16 [10]axisY i16 [12]group u16
///            [14]index u16 [16]linkIndex u16 [18]samePalette u8 [19]comment(13)
///            followed by PCX data (length bytes; 0 => linked sprite).
fn decode_sff_v1(buf: &[u8], wanted: impl Fn(i32, i32) -> bool) -> DecodedSff {
    let r = Reader(buf);
    // each subfile needs a distinct 32-byte header: bounds runaway counts / cycles
    let count = (r.u32(20) as usize).min(buf.len() / 32 + 1);
    let mut ofs = r.u32(24) as usize;
    let mut sprites: Vec<DecodedSprite> = Vec::with_capacity(count);
    let mut palettes: Vec<Palette> = Vec::new();
    let mut pal_keys: HashMap<Arc<Vec<u8>>, usize> = HashMap::new();
    // palette / palette index of the immediately preceding subfile (the one `samePalette` refers to)
    let mut prev: Option<(Option<Arc<Vec<u8>>>, i32)> = None;
    let mut prev_adjacent = false;

    for _ in 0..count {
        if ofs == 0 || ofs.saturating_add(32) > buf.len() {
            break;
        }
        let next = r.u32(ofs) as usize;
        let length = r.u32(ofs + 4) as usize;
        let mut spr = empty_sprite(
            r.u16(ofs + 12) as i32,
            r.u16(ofs + 14) as i32,
            r.i16(ofs + 8) as i32,
            r.i16(ofs + 10) as i32,
        );
        let link = r.u16(ofs + 16) as usize;
        let same_pal = r.u8(ofs + 18) != 0;
        let data_start = ofs + 32;
        let keep = wanted(spr.group, spr.index);

        if length == 0 {
            if keep {
                if let Some(src) = sprites.get(link) {
                    spr.width = src.width;
                    spr.height = src.height;
                    spr.indexed = src.indexed.clone();
                    spr.rgba = src.rgba.clone();
                    spr.palette = src.palette.clone();
                    spr.palette_index = src.palette_index;
                }
            }
        } else if keep {
            let block_end = if next > data_start { next } else { data_start.saturating_add(length) }.min(buf.len());
            let pcx = &buf[data_start..block_end];
            if let Some(h) = read_pcx_header(pcx).ok().filter(|h| {
                (h.width as usize).saturating_mul(h.height as usize) <= MAX_SPRITE_PIXELS
            }) {
                spr.width = h.width;
                spr.height = h.height;

                let data_end = if h.encoding == 1 {
                    let (px, end) = decode_pcx_rle(
                        pcx,
                        h.width,
                        h.height,
                        h.bytes_per_line,
                        PCX_HEADER_SIZE,
                        pcx.len(),
                    );
                    spr.indexed = Some(Arc::new(px));
                    end
                } else {
                    spr.indexed = Some(Arc::new(raw_rows(
                        pcx,
                        h.width as usize,
                        h.height as usize,
                        h.bytes_per_line as usize,
                    )));
                    PCX_HEADER_SIZE + h.height as usize * h.bytes_per_line as usize
                };
                let mut pal: Option<Arc<Vec<u8>>> = None;
                let inherited = if same_pal && prev_adjacent { prev.clone() } else { None };
                match inherited {
                    Some((ppal, pidx)) => {
                        pal = ppal.clone();
                        spr.palette_index = pidx;
                    }
                    None => {
                        if let Some(po) = find_pcx_palette(pcx, data_end) {
                            pal = Some(Arc::new(read_rgb_palette(pcx, po)));
                        } else if prev_adjacent {
                            if let Some((ppal, pidx)) = &prev {
                                pal = ppal.clone();
                                spr.palette_index = *pidx;
                            }
                        }
                        if let Some(p) = &pal {
                            if spr.palette_index < 0 {
                                if let Some(&found) = pal_keys.get(p) {
                                    pal = Some(palettes[found].data.clone());
                                    spr.palette_index = found as i32;
                                } else {
                                    let idx = palettes.len();
                                    palettes.push(Palette {
                                        group: spr.group,
                                        index: spr.index,
                                        data: p.clone(),
                                    });
                                    pal_keys.insert(p.clone(), idx);
                                    spr.palette_index = idx as i32;
                                }
                            }
                        }
                    }
                }
                spr.palette = pal;
            }
        }
        if spr.indexed.is_some() {
            prev = Some((spr.palette.clone(), spr.palette_index));
        }
        prev_adjacent = spr.indexed.is_some();
        sprites.push(spr);
        ofs = next;
    }
    DecodedSff {
        version: 1,
        sprites,
        palettes,
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
    let mut palettes: Vec<Palette> = Vec::new();
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
    let mut jobs: Vec<(usize, u8, u8, &[u8])> = Vec::new();
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
            links.push(Some(link).filter(|&l| l < i));
        } else {
            let start = (if flags & 1 == 0 { ldata } else { tdata }).saturating_add(d_ofs);
            let s = start.min(buf.len());
            let e = start.saturating_add(d_len).min(buf.len());
            jobs.push((i, fmt, depth, &buf[s..e.max(s)]));
            links.push(None);
        }
        sprites.push(spr);
    }
    let mut needed: Vec<bool> = sprites.iter().map(|s| wanted(s.group, s.index)).collect();
    for i in (0..sprites.len()).rev() {
        if let (true, Some(l)) = (needed[i], links[i]) {
            needed[l] = true;
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
    for i in 0..sprites.len() {
        if let Some(l) = links[i] {
            let (w, h, idx, rgba) = (
                sprites[l].width,
                sprites[l].height,
                sprites[l].indexed.clone(),
                sprites[l].rgba.clone(),
            );
            let spr = &mut sprites[i];
            spr.width = w;
            spr.height = h;
            spr.indexed = idx;
            spr.rgba = rgba;
        }
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
            if depth == 24 || depth == 32 {
                let bpp = (depth / 8) as usize;
                let mut out = vec![0u8; n * 4];
                for (p, o) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                    let g = |k: usize| data.get(p * bpp + k).copied().unwrap_or(0);
                    o[0] = g(0);
                    o[1] = g(1);
                    o[2] = g(2);
                    o[3] = if bpp == 4 { g(3) } else { 255 };
                }
                spr.rgba = Some(Arc::new(out));
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

// ---------------------------------------------------------------------------
// SFF v2 decompressors (ported from Ikemen GO image.go)
// ---------------------------------------------------------------------------

/// RLE8: a byte with top bits 01 (`b & 0xC0 == 0x40`) is a run of `b & 0x3F`
/// copies of the next byte; any other byte is a literal pixel.
pub fn decode_rle8(src: &[u8], size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let (mut i, mut j) = (0usize, 0usize);
    let len = src.len();
    while j < size && i < len {
        let d = src[i];
        i += 1;
        if d & 0xc0 == 0x40 {
            let n = (d & 0x3f) as usize;
            let val = if i < len {
                let v = src[i];
                i += 1;
                v
            } else {
                0
            };
            let e = (j + n).min(size);
            if val != 0 {
                out[j..e].fill(val);
            }
            j = e;
        } else {
            if d != 0 {
                out[j] = d;
            }
            j += 1;
        }
    }
    out
}

/// RLE5 packet: `[runLen u8] [dataLen: bit7 = color follows, bits0-6 = count]
/// [color u8 if bit7, else 0]`; the color is written runLen+1 times. Then
/// `count` bytes follow, each `rrrccccc`: color c (5 bits) written r+1 times.
pub fn decode_rle5(src: &[u8], size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let (mut i, mut j) = (0usize, 0usize);
    let len = src.len();
    while j < size && i < len {
        let mut rl = src[i] as i32;
        i += 1;
        if i >= len {
            break;
        }
        let dlb = src[i];
        i += 1;
        let mut dl = (dlb & 0x7f) as i32;
        let mut c = 0u8;
        if dlb & 0x80 != 0 {
            if i >= len {
                break;
            }
            c = src[i];
            i += 1;
        }
        loop {
            let count = (rl + 1) as usize;
            let end = (j + count).min(size);
            if c != 0 {
                out[j..end].fill(c);
            }
            j = end;
            dl -= 1;
            if dl < 0 {
                break;
            }
            if i >= len {
                return out;
            }
            let b = src[i];
            i += 1;
            c = b & 0x1f;
            rl = (b >> 5) as i32;
        }
    }
    out
}

/// LZ5: a control byte supplies 8 flags (LSB first); 0 = RLE packet, 1 = LZ packet.
///  RLE: `ccc ddddd` -> color d repeated c times; if c == 0, the next byte + 8 is the count.
///  LZ short (`b & 0x3F != 0`): copy `(b & 0x3F) + 1` bytes; offset = next byte + 1,
///    except every 4th short packet, whose offset is built from the top two bits
///    of the previous 3 short packets plus its own (recycled bits), + 1.
///  LZ long (`b & 0x3F == 0`): offset = `((b & 0xC0) << 2 | next) + 1`, length = next2 + 3.
pub fn decode_lz5(src: &[u8], size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let len = src.len();
    if len == 0 {
        return out;
    }
    let mut i = 0usize;
    let mut j = 0usize;
    let mut ct = src[i];
    i += 1;
    let mut cts = 0u32;
    let (mut rb, mut rbc) = (0usize, 0u32);
    macro_rules! rd {
        () => {
            if i < len {
                i += 1;
                src[i - 1] as usize
            } else {
                0
            }
        };
    }
    while j < size {
        if i >= len {
            break;
        }
        let d = src[i] as usize;
        i += 1;
        if ct & (1 << cts) != 0 {
            let (off, n);
            if d & 0x3f == 0 {
                off = ((d << 2) | rd!()) + 1;
                n = rd!() + 2;
            } else {
                rb |= (d & 0xc0) >> rbc;
                rbc += 2;
                n = d & 0x3f;
                if rbc < 8 {
                    off = rd!() + 1;
                } else {
                    off = rb + 1;
                    rb = 0;
                    rbc = 0;
                }
            }
            // copies n + 1 bytes (may overlap)
            let e = (j + n + 1).min(size);
            if off > j {
                // reads before the start of the buffer yield 0
                while j < e {
                    out[j] = if j >= off { out[j - off] } else { 0 };
                    j += 1;
                }
            } else if off >= e - j {
                out.copy_within(j - off..e - off, j);
                j = e;
            } else {
                while j < e {
                    out[j] = out[j - off];
                    j += 1;
                }
            }
        } else {
            let (n, c);
            if d & 0xe0 == 0 {
                n = rd!() + 8;
                c = d as u8;
            } else {
                n = d >> 5;
                c = (d & 0x1f) as u8;
            }
            let e = (j + n).min(size);
            out[j..e].fill(c);
            j = e;
        }
        cts += 1;
        if cts >= 8 {
            if i >= len {
                break;
            }
            ct = src[i];
            i += 1;
            cts = 0;
        }
    }
    out
}
