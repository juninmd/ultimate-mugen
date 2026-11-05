//! SND (ElecbyteSnd) decoder. Port of `src/mugen/snd.ts`.
//!
//! Header: [0]"ElecbyteSnd\0" [12]verLo u16 [14]verHi u16 [16]numSounds u32
//!         [20]firstSubfileOffset u32
//! Subfile: [0]nextSubfileOffset u32 [4]length u32 [8]group i32 [12]index i32
//!          [16] `length` bytes of sound data (normally a RIFF/WAVE file).

use crate::Reader;
use std::collections::{HashMap, HashSet};

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
    let cap = count.min(buf.len() / 16 + 1);
    let mut out = HashMap::with_capacity(cap);
    let mut seen = HashSet::with_capacity(cap);
    for _ in 0..count {
        if ofs < 24 || ofs.saturating_add(16) > buf.len() || !seen.insert(ofs) {
            break;
        }
        let next = r.u32(ofs) as usize;
        let length = r.u32(ofs + 4) as usize;
        let key = (r.i32(ofs + 8), r.i32(ofs + 12));
        let start = ofs + 16;
        let end = buf.len().min(start.saturating_add(length));
        if end > start {
            out.entry(key).or_insert_with(|| buf[start..end].to_vec());
        }
        ofs = next;
    }
    Ok(out)
}
