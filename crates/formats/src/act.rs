//! ACT palette decoder (Photoshop color table as used by MUGEN characters).
//! Port of `src/mugen/act.ts`.
//!
//! 256 RGB triplets (768 bytes; trailing extra bytes are ignored). MUGEN
//! stores colors in REVERSE order: file entry i is palette index 255 - i
//! (same as Ikemen GO's readActPalette). Index 0 is transparent.

/// Decode to RGBA 256*4 (index 0 alpha 0).
pub fn decode_act(buf: &[u8]) -> Vec<u8> {
    let mut pal = vec![0u8; 1024];
    let count = (buf.len() / 3).min(256);
    for i in 0..count {
        let dst = (255 - i) * 4;
        pal[dst..dst + 3].copy_from_slice(&buf[i * 3..i * 3 + 3]);
        pal[dst + 3] = 255;
    }
    pal[3] = 0;
    pal
}

/// Inverse of [`decode_act`]: RGBA 256*4 palette -> 768-byte MUGEN .act.
pub fn encode_act(pal: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 768];
    for i in 0..256 {
        let src = (255 - i) * 4;
        for k in 0..3 {
            out[i * 3 + k] = pal.get(src + k).copied().unwrap_or(0);
        }
    }
    out
}
