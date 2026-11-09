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

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    let mut checked = kind.to_vec();
    checked.extend_from_slice(body);
    out.extend_from_slice(&checked);
    out.extend_from_slice(&crc32(&checked).to_be_bytes());
}

pub fn encode_indexed_png(width: usize, height: usize, indices: &[u8], palette: &[[u8; 3]]) -> Vec<u8> {
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut header = Vec::new();
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 3, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    let plte: Vec<u8> = palette.iter().flatten().copied().collect();
    chunk(&mut out, b"PLTE", &plte);
    chunk(&mut out, b"tRNS", &[0]);
    let mut raw = Vec::with_capacity((width + 1) * height);
    for row in indices.chunks(width.max(1)) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    chunk(&mut out, b"IDAT", &compress_to_vec_zlib(&raw, 6));
    chunk(&mut out, b"IEND", &[]);
    out
}

pub fn write_indexed_png(path: &Path, width: usize, height: usize, indices: &[u8], palette: &[[u8; 3]]) -> Result<(), String> {
    std::fs::write(path, encode_indexed_png(width, height, indices, palette)).map_err(|e| format!("{}: {e}", path.display()))
}
