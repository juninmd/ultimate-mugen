//! SND (ElecbyteSnd) decoder. Port of `src/mugen/snd.ts`.
//!
//! Header: [0]"ElecbyteSnd\0" [12]verLo u16 [14]verHi u16 [16]numSounds u32
//!         [20]firstSubfileOffset u32
//! Subfile: [0]nextSubfileOffset u32 [4]length u32 [8]group i32 [12]index i32
//!          [16] `length` bytes of sound data (normally a RIFF/WAVE file).

use crate::Reader;
use std::collections::{hash_map::Entry, HashMap, HashSet};

const SIGNATURE: &[u8; 12] = b"ElecbyteSnd\0";

/// Decode an SND file into `(group, index)` -> sound file bytes (WAV).
/// Duplicate keys keep the first occurrence (MUGEN/Ikemen behavior).
/// An invalid file yields an empty map (see [`decode_snd_checked`]).
pub fn decode_snd(buf: &[u8]) -> HashMap<(i32, i32), Vec<u8>> {
    decode_snd_checked(buf).unwrap_or_default()
}

/// Like [`decode_snd`] but reports an invalid signature as an error.
pub fn decode_snd_checked(buf: &[u8]) -> Result<HashMap<(i32, i32), Vec<u8>>, String> {
    if buf.len() < SIGNATURE.len() || &buf[..12] != SIGNATURE {
        return Err("SND: invalid signature (expected ElecbyteSnd)".into());
    }
    let r = Reader(buf);
    let count = r.u32(16) as usize;
    let mut ofs = r.u32(20) as usize;
    let mut out = HashMap::with_capacity(count.min(8192));
    let mut seen: Option<HashSet<usize>> = None;
    for _ in 0..count {
        if ofs < 24 || ofs.saturating_add(16) > buf.len() {
            break;
        }
        let next = r.u32(ofs) as usize;
        let length = r.u32(ofs + 4) as usize;
        let key = (r.i32(ofs + 8), r.i32(ofs + 12));
        let start = ofs + 16;
        let end = buf.len().min(start.saturating_add(length));
        if end > start {
            if let Entry::Vacant(e) = out.entry(key) { e.insert(buf[start..end].to_vec()); }
        }
        if next <= ofs {
            let set = seen.get_or_insert_with(HashSet::new);
            if !set.insert(next) { break; }
        }
        ofs = next;
    }
    Ok(out)
}

const SND_HEADER_LEN: usize = 512;

pub fn encode_snd(sounds: &[((i32, i32), &[u8])]) -> Vec<u8> {
    let payload: usize = sounds.iter().map(|(_, data)| 16 + data.len()).sum();
    let mut out = Vec::with_capacity(SND_HEADER_LEN + payload);
    out.extend_from_slice(SIGNATURE);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(sounds.len() as u32).to_le_bytes());
    let first_offset = if sounds.is_empty() { 0u32 } else { SND_HEADER_LEN as u32 };
    out.extend_from_slice(&first_offset.to_le_bytes());
    out.resize(SND_HEADER_LEN, 0);

    for (idx, ((group, index), data)) in sounds.iter().enumerate() {
        let current = out.len();
        let next = if idx + 1 < sounds.len() {
            (current + 16 + data.len()) as u32
        } else {
            0u32
        };
        out.extend_from_slice(&next.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&group.to_le_bytes());
        out.extend_from_slice(&index.to_le_bytes());
        out.extend_from_slice(data);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_snd_round_trips_multiple_entries() {
        let wav_a = b"RIFF\x24\x00\x00\x00WAVEfmt ";
        let wav_b = b"RIFF\x10\x00\x00\x00WAVEdata";
        let encoded = encode_snd(&[((0, 0), wav_a.as_slice()), ((5, 2), wav_b.as_slice())]);
        assert!(encoded.len() >= SND_HEADER_LEN);
        let decoded = decode_snd_checked(&encoded).expect("valid encoded SND");
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.get(&(0, 0)).map(Vec::as_slice), Some(wav_a.as_slice()));
        assert_eq!(decoded.get(&(5, 2)).map(Vec::as_slice), Some(wav_b.as_slice()));
    }

    #[test]
    fn encode_snd_handles_empty_bank() {
        let encoded = encode_snd(&[]);
        assert_eq!(encoded.len(), SND_HEADER_LEN);
        let decoded = decode_snd_checked(&encoded).expect("valid empty SND");
        assert!(decoded.is_empty());
    }
}

