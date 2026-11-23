use miniz_oxide::deflate::compress_to_vec_zlib;

fn crc32_update(mut crc: u32, bytes: &[u8]) -> u32 {
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    crc
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (ia, ib, ic) = (a as i32, b as i32, c as i32);
    let p = ia + ib - ic;
    let (pa, pb, pc) = ((p - ia).abs(), (p - ib).abs(), (p - ic).abs());
    if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c }
}

fn filter_into(kind: u8, row: &[u8], above: &[u8], bpp: usize, dst: &mut Vec<u8>) -> u64 {
    dst.clear();
    let mut score = 0u64;
    for i in 0..row.len() {
        let left = if i >= bpp { row[i - bpp] } else { 0 };
        let up = above[i];
        let up_left = if i >= bpp { above[i - bpp] } else { 0 };
        let v = row[i].wrapping_sub(match kind {
            0 => 0,
            1 => left,
            2 => up,
            3 => ((left as u16 + up as u16) / 2) as u8,
            _ => paeth(left, up, up_left),
        });
        score += (v as i8).unsigned_abs() as u64;
        dst.push(v);
    }
    score
}

fn adaptive_scanlines(data: &[u8], row_bytes: usize, bpp: usize) -> Vec<u8> {
    let stride = row_bytes.max(1);
    let zero = vec![0u8; stride];
    let mut out = Vec::with_capacity(data.len() + data.len() / stride + 1);
    let mut trial = Vec::with_capacity(stride);
    let mut best = Vec::with_capacity(stride);
    let mut above: &[u8] = &zero;
    let max_kind = if bpp == 1 { 1u8 } else { 5u8 };
    for row in data.chunks(stride) {
        let mut best_kind = 0u8;
        let mut best_score = u64::MAX;
        for kind in 0u8..max_kind {
            let score = filter_into(kind, row, above, bpp, &mut trial);
            if score < best_score {
                best_score = score;
                best_kind = kind;
                std::mem::swap(&mut best, &mut trial);
            }
        }
        out.push(best_kind);
        out.extend_from_slice(&best);
        above = row;
    }
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(body);
    let crc = !crc32_update(crc32_update(0xFFFF_FFFFu32, kind), body);
    out.extend_from_slice(&crc.to_be_bytes());
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

pub fn encode_rgba_png(width: usize, height: usize, rgba: &[u8], level: u8) -> Vec<u8> {
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut header = Vec::new();
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &compress_to_vec_zlib(&adaptive_scanlines(rgba, width * 4, 4), level));
    chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::png::decode_png;

    #[test]
    fn palette_png_round_trips_indices_and_every_alpha() {
        let palette: Vec<[u8; 4]> = (0..256u32).map(|i| [i as u8, (255 - i) as u8, (i * 7) as u8, if i == 0 { 0 } else { (i % 3 * 100) as u8 }]).collect();
        let indices: Vec<u8> = (0..12u32).map(|i| (i * 37) as u8).collect();
        let png = decode_png(&encode_palette_png(4, 3, &indices, &palette, 9)).unwrap();
        assert_eq!(png.indexed.unwrap(), indices);
        assert_eq!(png.palette.unwrap(), palette.concat());
    }

    #[test]
    fn rgba_png_round_trips_pixels() {
        let rgba: Vec<u8> = (0..2 * 3 * 4).map(|i| (i * 11) as u8).collect();
        let png = decode_png(&encode_rgba_png(2, 3, &rgba, 9)).unwrap();
        assert_eq!(png.rgba.unwrap(), rgba);
    }

    #[test]
    fn rgb_png_round_trips_as_opaque_rgba() {
        let rgb: Vec<u8> = (0..3 * 2 * 3).map(|i| (i * 17) as u8).collect();
        let png = decode_png(&encode_rgb_png(3, 2, &rgb, 6)).unwrap();
        let rgba = png.rgba.unwrap();
        assert_eq!(rgba.len(), 3 * 2 * 4);
        for (px, chunk) in rgba.chunks_exact(4).enumerate() {
            assert_eq!(&chunk[..3], &rgb[px * 3..px * 3 + 3]);
            assert_eq!(chunk[3], 255);
        }
    }
}
