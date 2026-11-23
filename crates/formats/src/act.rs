pub fn decode_act(buf: &[u8]) -> Vec<u8> {
    let mut pal = vec![0u8; 1024];
    if buf.len() >= 768 {
        let src_chunks = buf[..768].as_chunks::<3>().0;
        let dst_chunks = pal.as_chunks_mut::<4>().0;
        for (i, src) in src_chunks.iter().enumerate() {
            let dst = &mut dst_chunks[255 - i];
            dst[0] = src[0];
            dst[1] = src[1];
            dst[2] = src[2];
            dst[3] = 255;
        }
        pal[3] = 0;
        return pal;
    }
    let count = (buf.len() / 3).min(256);
    for i in 0..count {
        let dst = (255 - i) * 4;
        pal[dst..dst + 3].copy_from_slice(&buf[i * 3..i * 3 + 3]);
        pal[dst + 3] = 255;
    }
    pal[3] = 0;
    pal
}

pub fn encode_act(pal: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 768];
    if pal.len() >= 1024 {
        let src_chunks = pal[..1024].as_chunks::<4>().0;
        let dst_chunks = out.as_chunks_mut::<3>().0;
        for (i, dst) in dst_chunks.iter_mut().enumerate() {
            let src = &src_chunks[255 - i];
            dst[0] = src[0];
            dst[1] = src[1];
            dst[2] = src[2];
        }
        return out;
    }
    for i in 0..256 {
        let src = (255 - i) * 4;
        for k in 0..3 {
            out[i * 3 + k] = pal.get(src + k).copied().unwrap_or(0);
        }
    }
    out
}

pub fn palette_average_luma(pal: &[u8]) -> f32 {
    let entries = (pal.len() / 4).min(256);
    if entries <= 1 {
        return 0.0;
    }
    let mut sum = 0.0_f64;
    let mut count = 0usize;
    for i in 1..entries {
        let off = i * 4;
        if pal[off + 3] == 0 {
            continue;
        }
        let r = pal[off] as f64;
        let g = pal[off + 1] as f64;
        let b = pal[off + 2] as f64;
        sum += 0.2126 * r + 0.7152 * g + 0.0722 * b;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f64 / 255.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_act_round_trips_and_marks_index_zero_transparent() {
        let mut raw = vec![0u8; 768];
        for i in 0..256 {
            raw[i * 3] = i as u8;
            raw[i * 3 + 1] = (255 - i) as u8;
            raw[i * 3 + 2] = ((i * 3) & 0xff) as u8;
        }
        let pal = decode_act(&raw);
        assert_eq!(pal.len(), 1024);
        assert_eq!(pal[3], 0, "index 0 must stay transparent");
        assert_eq!(pal[7], 255, "index 1 must be opaque");
        let reencoded = encode_act(&pal);
        assert_eq!(reencoded, raw);
    }

    #[test]
    fn partial_act_buffer_decodes_safely_without_panic() {
        let short = vec![10u8, 20, 30, 40, 50, 60, 70];
        let pal = decode_act(&short);
        assert_eq!(pal.len(), 1024);
        assert_eq!(&pal[255 * 4..255 * 4 + 4], &[10, 20, 30, 255]);
        assert_eq!(&pal[254 * 4..254 * 4 + 4], &[40, 50, 60, 255]);
        assert_eq!(pal[3], 0);
    }

    #[test]
    fn palette_average_luma_distinguishes_dark_and_bright_palettes() {
        let bright = decode_act(&vec![240u8; 768]);
        let dark = decode_act(&vec![16u8; 768]);
        assert!(palette_average_luma(&bright) > 0.9);
        assert!(palette_average_luma(&dark) < 0.1);
    }
}

