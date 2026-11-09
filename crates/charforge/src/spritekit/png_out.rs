use miniz_oxide::deflate::compress_to_vec_zlib;
use std::path::Path;

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (ia, ib, ic) = (a as i32, b as i32, c as i32);
    let p = ia + ib - ic;
    let (pa, pb, pc) = ((p - ia).abs(), (p - ib).abs(), (p - ic).abs());
    if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c }
}

fn filtered(kind: u8, row: &[u8], above: &[u8], bpp: usize) -> Vec<u8> {
    (0..row.len()).map(|i| {
        let left = if i >= bpp { row[i - bpp] } else { 0 };
        let up = above[i];
        let up_left = if i >= bpp { above[i - bpp] } else { 0 };
        row[i].wrapping_sub(match kind {
            0 => 0,
            1 => left,
            2 => up,
            3 => ((left as u16 + up as u16) / 2) as u8,
            _ => paeth(left, up, up_left),
        })
    }).collect()
}

fn adaptive_scanlines(data: &[u8], row_bytes: usize, bpp: usize) -> Vec<u8> {
    let zero = vec![0u8; row_bytes];
    let mut out = Vec::with_capacity(data.len() + data.len() / row_bytes.max(1) + 1);
    let mut above: &[u8] = &zero;
    for row in data.chunks(row_bytes.max(1)) {
        let best = (0u8..if bpp == 1 { 1 } else { 5 })
            .map(|kind| (kind, filtered(kind, row, above, bpp)))
            .min_by_key(|(_, bytes)| bytes.iter().map(|&b| (b as i8).unsigned_abs() as u64).sum::<u64>())
            .unwrap_or((0, row.to_vec()));
        out.push(best.0);
        out.extend_from_slice(&best.1);
        above = row;
    }
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    let mut checked = kind.to_vec();
    checked.extend_from_slice(body);
    out.extend_from_slice(&checked);
    out.extend_from_slice(&crc32(&checked).to_be_bytes());
}

pub fn encode_indexed_png(width: usize, height: usize, indices: &[u8], palette: &[[u8; 3]]) -> Vec<u8> {
    let rgba: Vec<[u8; 4]> = palette.iter().enumerate().map(|(i, c)| [c[0], c[1], c[2], if i == 0 { 0 } else { 255 }]).collect();
    encode_palette_png(width, height, indices, &rgba, 6)
}

pub fn encode_palette_png(width: usize, height: usize, indices: &[u8], palette: &[[u8; 4]], level: u8) -> Vec<u8> {
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut header = Vec::new();
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 3, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    let plte: Vec<u8> = palette.iter().flat_map(|c| [c[0], c[1], c[2]]).collect();
    chunk(&mut out, b"PLTE", &plte);
    let alphas: Vec<u8> = palette.iter().map(|c| c[3]).collect();
    if let Some(last) = alphas.iter().rposition(|&a| a != 255) {
        chunk(&mut out, b"tRNS", &alphas[..=last]);
    }
    chunk(&mut out, b"IDAT", &compress_to_vec_zlib(&adaptive_scanlines(indices, width, 1), level));
    chunk(&mut out, b"IEND", &[]);
    out
}

pub fn encode_rgb_png(width: usize, height: usize, rgb: &[u8], level: u8) -> Vec<u8> {
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut header = Vec::new();
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &compress_to_vec_zlib(&adaptive_scanlines(rgb, width * 3, 3), level));
    chunk(&mut out, b"IEND", &[]);
    out
}

pub fn write_indexed_png(path: &Path, width: usize, height: usize, indices: &[u8], palette: &[[u8; 3]]) -> Result<(), String> {
    std::fs::write(path, encode_indexed_png(width, height, indices, palette)).map_err(|e| format!("{}: {e}", path.display()))
}
