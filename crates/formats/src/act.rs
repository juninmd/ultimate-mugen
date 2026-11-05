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
