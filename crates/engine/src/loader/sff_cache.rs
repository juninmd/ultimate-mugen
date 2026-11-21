//! One-slot cache for decoded `.sff` files.
//!
//! A mirror match, a rematch and a revisit of the same fighter in the select screen all re-decode the same bytes:
//! the registry caches per palette, but the palette changes while the sprite file does not. Decoding an 8 MB `.sff`
//! costs 0.2–0.9 s on the browser, and the decoded sprites keep their pixels in `Arc`s, so a cache hit is a cheap
//! clone that shares every pixel buffer.
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use ume_formats::sff::{decode_sff, decode_sff_only, DecodedSff};

struct Slot {
    fingerprint: (usize, u64, u64),
    decoded: Arc<DecodedSff>,
}

static SLOT: OnceLock<Mutex<Option<Slot>>> = OnceLock::new();

fn fnv(seed: u64, bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut hash = seed;
    for byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

fn fingerprint(buf: &[u8], wanted: &HashSet<(i32, i32)>) -> (usize, u64, u64) {
    let mut keys: Vec<(i32, i32)> = wanted.iter().copied().collect();
    keys.sort_unstable();
    let mask = fnv(0xcbf2_9ce4_8422_2325, keys.iter().flat_map(|&(g, i)| g.to_le_bytes().into_iter().chain(i.to_le_bytes())));
    (buf.len(), fnv(0xcbf2_9ce4_8422_2325, buf.iter().copied()), mask)
}

/// Decodes `buf`, reusing the previous result when the bytes and the wanted sprite keys are identical. An empty
/// `wanted` set means every sprite.
pub fn decode_cached(buf: &[u8], wanted: &HashSet<(i32, i32)>) -> Result<DecodedSff, String> {
    let key = fingerprint(buf, wanted);
    let slot = SLOT.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = slot.lock() {
        if let Some(hit) = guard.as_ref().filter(|s| s.fingerprint == key) {
            return Ok((*hit.decoded).clone());
        }
    }
    let decoded = Arc::new(if wanted.is_empty() {
        decode_sff(buf)?
    } else {
        decode_sff_only(buf, |group, index| wanted.contains(&(group, index)))?
    });
    if let Ok(mut guard) = slot.lock() {
        *guard = Some(Slot { fingerprint: key, decoded: decoded.clone() });
    }
    Ok((*decoded).clone())
}
