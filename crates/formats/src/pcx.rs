//! PCX decoder (8-bit, single plane), as embedded in SFF v1 files.
//! Port of `src/mugen/pcx.ts`.
//!
//! Layout: 128-byte header, RLE (or raw) scanlines of `bytes_per_line` bytes
//! each (padding bytes discarded), and an optional 256-color VGA palette at
//! the end introduced by a 0x0C marker byte.

pub const PCX_HEADER_SIZE: usize = 128;

#[derive(Clone, Debug)]
pub struct PcxHeader {
    pub encoding: u8,
    pub bits_per_pixel: u8,
    pub width: u32,
    pub height: u32,
    pub planes: u8,
    pub bytes_per_line: u32,
}

#[derive(Clone, Debug)]
pub struct PcxImage {
    pub width: u32,
    pub height: u32,
    pub bytes_per_line: u32,
    /// Indexed pixels, `width * height`.
    pub pixels: Vec<u8>,
    /// RGBA 256*4 palette (index 0 alpha 0, others 255), if found.
    pub palette: Option<Vec<u8>>,
    /// Offset right after the image data.
    pub data_end: usize,
}

pub fn read_pcx_header(buf: &[u8]) -> Result<PcxHeader, String> {
    if buf.len() < PCX_HEADER_SIZE {
        return Err("PCX: buffer too small for header".into());
    }
    let u16 = |o: usize| buf[o] as i32 | (buf[o + 1] as i32) << 8;
    let (xmin, ymin, xmax, ymax) = (u16(4), u16(6), u16(8), u16(10));
    let width = (xmax - xmin + 1).max(0) as u32;
    let height = (ymax - ymin + 1).max(0) as u32;
    if !crate::pixels_within_limit(width, height) {
        return Err(format!("PCX: {width}x{height} exceeds the pixel limit"));
    }
    let mut bytes_per_line = u16(66) as u32;
    if bytes_per_line < width {
        bytes_per_line = width; // broken headers: never drop real pixels
    }
    Ok(PcxHeader {
        encoding: buf[2],
        bits_per_pixel: buf[3],
        width,
        height,
        planes: if buf[65] == 0 { 1 } else { buf[65] },
        bytes_per_line,
    })
}

/// Decode PCX RLE scanlines from `data[start..end]`. A code byte >= 0xC0 is a
/// run of `code & 0x3F` copies of the next byte; anything else is a literal.
/// Runs are cut at scanline boundaries (MUGEN/Ikemen behavior); bytes in the
/// `[width, bytes_per_line)` padding area are dropped.
///
/// Returns the pixels and the offset just past the last consumed byte.
pub fn decode_pcx_rle(
    data: &[u8],
    width: u32,
    height: u32,
    bytes_per_line: u32,
    start: usize,
    end: usize,
) -> (Vec<u8>, usize) {
    let (w, h, bpl) = (width as usize, height as usize, bytes_per_line as usize);
    let end = end.min(data.len());
    let total = match w.checked_mul(h) {
        Some(t) => t,
        None => return (Vec::new(), start),
    };
    let mut out = Vec::new();
    if out.try_reserve_exact(total).is_err() {
        return (Vec::new(), start);
    }
    out.resize(total, 0);
    let mut i = start;
    for y in 0..h {
        let row = y * w;
        let mut x = 0usize;
        while x < bpl {
            if i >= end {
                return (out, i);
            }
            let mut d = data[i];
            i += 1;
            let mut n = 1usize;
            if d >= 0xc0 {
                n = (d & 0x3f) as usize;
                d = if i < end {
                    i += 1;
                    data[i - 1]
                } else {
                    0
                };
            }
            let stop = (x + n).min(bpl);
            let wstop = stop.min(w);
            if x < wstop {
                out[row + x..row + wstop].fill(d);
            }
            x = stop;
        }
    }
    (out, i)
}

/// Read a 768-byte RGB palette at `offset` into RGBA (index 0 transparent).
pub fn read_rgb_palette(buf: &[u8], offset: usize) -> Vec<u8> {
    let mut pal = vec![0u32; 256];
    if offset.saturating_add(768) <= buf.len() {
        let src = &buf[offset..offset + 768];
        for c in 1..256 {
            let o = c * 3;
            pal[c] = u32::from_ne_bytes([src[o], src[o + 1], src[o + 2], 255]);
        }
        pal[0] = u32::from_ne_bytes([src[0], src[1], src[2], 0]);
    } else {
        for c in 0..256 {
            let o = offset + c * 3;
            let r = buf.get(o).copied().unwrap_or(0);
            let g = buf.get(o + 1).copied().unwrap_or(0);
            let b = buf.get(o + 2).copied().unwrap_or(0);
            let a = if c == 0 { 0 } else { 255 };
            pal[c] = u32::from_ne_bytes([r, g, b, a]);
        }
    }
    pal.iter().flat_map(|v| v.to_ne_bytes()).collect()
}

/// Locate the 256-color palette of a PCX buffer. Preferred: the 0x0C marker
/// immediately after the image data. Fallbacks: a marker exactly 769 bytes
/// before the end, then (legacy SFF v1 quirk) the raw last 768 bytes.
/// Returns the offset of the first palette byte.
pub fn find_pcx_palette(buf: &[u8], data_end: usize) -> Option<usize> {
    let len = buf.len();
    if data_end + 769 <= len && buf[data_end] == 0x0c {
        return Some(data_end + 1);
    }
    if len >= PCX_HEADER_SIZE + 769 && buf[len - 769] == 0x0c {
        return Some(len - 768);
    }
    if len >= 768 && len - 768 >= data_end {
        return Some(len - 768);
    }
    None
}

/// Decode an 8-bit single-plane PCX image.
pub fn decode_pcx(buf: &[u8], read_palette: bool) -> Result<PcxImage, String> {
    let h = read_pcx_header(buf)?;
    if h.bits_per_pixel != 8 || h.planes != 1 {
        return Err(format!(
            "PCX: unsupported format ({} bpp, {} planes); only 8-bit 1-plane",
            h.bits_per_pixel, h.planes
        ));
    }
    let (pixels, data_end) = if h.encoding == 1 {
        decode_pcx_rle(buf, h.width, h.height, h.bytes_per_line, PCX_HEADER_SIZE, buf.len())
    } else {
        let (w, bpl) = (h.width as usize, h.bytes_per_line as usize);
        let px = raw_rows(buf, w, h.height as usize, bpl);
        (px, buf.len().min(PCX_HEADER_SIZE + h.height as usize * bpl))
    };
    let palette = if read_palette { find_pcx_palette(buf, data_end).map(|po| read_rgb_palette(buf, po)) } else { None };
    Ok(PcxImage { width: h.width, height: h.height, bytes_per_line: h.bytes_per_line, pixels, palette, data_end })
}

/// Uncompressed scanlines starting right after the header.
pub(crate) fn raw_rows(buf: &[u8], w: usize, h: usize, bpl: usize) -> Vec<u8> {
    let total = match w.checked_mul(h) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let mut px = Vec::new();
    if px.try_reserve_exact(total).is_err() {
        return Vec::new();
    }
    px.resize(total, 0);
    for y in 0..h {
        let src = PCX_HEADER_SIZE + y * bpl;
        if src >= buf.len() {
            break;
        }
        let avail = w.min(buf.len() - src);
        px[y * w..y * w + avail].copy_from_slice(&buf[src..src + avail]);
    }
    px
}

impl PcxHeader {
    pub fn is_8bit_indexed(&self) -> bool {
        self.bits_per_pixel == 8 && self.planes == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_truncated_and_oversized_pcx_headers() {
        assert!(read_pcx_header(&[0u8; 64]).is_err());
        let mut huge = vec![0u8; PCX_HEADER_SIZE];
        huge[8] = 0xff;
        huge[9] = 0x7f;
        huge[10] = 0xff;
        huge[11] = 0x7f;
        assert!(read_pcx_header(&huge).is_err());
    }

    #[test]
    fn decodes_rle_pcx_with_vga_palette_and_fixes_short_bpl() {
        let mut buf = vec![0u8; PCX_HEADER_SIZE];
        buf[2] = 1;
        buf[3] = 8;
        buf[8] = 3;
        buf[10] = 1;
        buf[65] = 1;
        buf[66] = 2;
        buf.extend_from_slice(&[0xc4, 7, 0xc4, 9]);
        buf.push(0x0c);
        buf.extend_from_slice(&vec![128u8; 768]);
        let hdr = read_pcx_header(&buf).expect("valid header");
        assert!(hdr.is_8bit_indexed());
        assert_eq!((hdr.width, hdr.height, hdr.bytes_per_line), (4, 2, 4));
        let img = decode_pcx(&buf, true).expect("valid pcx");
        assert_eq!(img.pixels, vec![7, 7, 7, 7, 9, 9, 9, 9]);
        let pal = img.palette.expect("vga palette");
        assert_eq!(pal[3], 0);
        assert_eq!(pal[7], 255);
    }
}

