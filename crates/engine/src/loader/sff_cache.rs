use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use ume_formats::sff::{decode_sff, decode_sff_only, DecodedSff};

const SLOTS: usize = 4;
const SAMPLE: usize = 64 << 10;

struct Slot {
    fingerprint: (usize, u64, u64),
    decoded: Arc<DecodedSff>,
}

static CACHE: OnceLock<Mutex<Vec<Slot>>> = OnceLock::new();

fn fnv(seed: u64, bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut hash = seed;
    for byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

fn sample_hash(buf: &[u8]) -> u64 {
    if buf.len() <= SAMPLE * 3 {
        return fnv(0xcbf2_9ce4_8422_2325, buf.iter().copied());
    }
    let mid = buf.len() / 2 - SAMPLE / 2;
    let tail = buf.len() - SAMPLE;
    let h0 = fnv(0xcbf2_9ce4_8422_2325, buf[..SAMPLE].iter().copied());
    let h1 = fnv(h0.rotate_left(13), buf[mid..mid + SAMPLE].iter().copied());
    fnv(h1.rotate_left(29), buf[tail..].iter().copied())
}

fn fingerprint(buf: &[u8], wanted: &HashSet<(i32, i32)>) -> (usize, u64, u64) {
    let mut keys: Vec<(i32, i32)> = wanted.iter().copied().collect();
    keys.sort_unstable();
    let mask = fnv(
        0xcbf2_9ce4_8422_2325,
        keys.iter()
            .flat_map(|&(g, i)| g.to_le_bytes().into_iter().chain(i.to_le_bytes())),
    );
    (buf.len(), sample_hash(buf), mask)
}

pub fn decode_cached(buf: &[u8], wanted: &HashSet<(i32, i32)>) -> Result<DecodedSff, String> {
    let key = fingerprint(buf, wanted);
    let cache = CACHE.get_or_init(|| Mutex::new(Vec::with_capacity(SLOTS)));
    if let Ok(mut guard) = cache.lock() {
        if let Some(pos) = guard.iter().position(|s| s.fingerprint == key) {
            let hit = guard.remove(pos);
            let out = (*hit.decoded).clone();
            guard.push(hit);
            return Ok(out);
        }
    }
    let decoded = Arc::new(if wanted.is_empty() {
        decode_sff(buf)?
    } else {
        decode_sff_only(buf, |group, index| wanted.contains(&(group, index)))?
    });
    if let Ok(mut guard) = cache.lock() {
        guard.retain(|s| s.fingerprint != key);
        if guard.len() >= SLOTS {
            guard.remove(0);
        }
        guard.push(Slot {
            fingerprint: key,
            decoded: decoded.clone(),
        });
    }
    Ok((*decoded).clone())
}

