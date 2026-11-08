use super::SR;
pub(crate) fn wav(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let total_bytes = 44 + n as usize * 2;
    let mut v = Vec::with_capacity(total_bytes);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + n * 2).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&SR.to_le_bytes());
    v.extend_from_slice(&(SR * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&(n * 2).to_le_bytes());
    v.resize(total_bytes, 0);
    let dst = &mut v[44..];
    for (i, s) in samples.iter().enumerate() {
        let val = (s.clamp(-1.0, 1.0) * 32000.0) as i16;
        let b = val.to_le_bytes();
        dst[i * 2] = b[0];
        dst[i * 2 + 1] = b[1];
    }
    v
}
