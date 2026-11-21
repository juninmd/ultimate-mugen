//! Sound preload helpers shared by the desktop and the browser paths.
//!
//! The browser has no worker thread, so there the match decodes one frame's worth of samples per call instead of
//! every sample of every fighter in a single frame (which froze the loading bar for half a second).
use super::{CharSounds, SampleKey, Source};

/// Samples decoded per browser preload call.
#[cfg(target_arch = "wasm32")]
pub(super) const PER_FRAME: usize = 24;

/// Decodes up to `PER_FRAME` queued samples and reports whether the queue is now empty.
#[cfg(target_arch = "wasm32")]
pub(super) fn drain(sources: &mut std::collections::HashMap<SampleKey, Option<Source>>, queue: &mut Vec<(SampleKey, Vec<u8>)>) -> bool {
    let take = queue.len().min(PER_FRAME);
    let chunk: Vec<(SampleKey, Vec<u8>)> = queue.drain(..take).collect();
    for (key, r) in ume_formats::par::map(&chunk, |(k, b)| (k.clone(), super::decode(b).ok_or_else(|| b.clone()))) {
        sources.insert(key, Some(r.map(Source::Pcm).unwrap_or_else(Source::Raw)));
    }
    queue.is_empty()
}

/// Every sample of the match that is not in `sources` yet.
pub(super) fn missing(chars: &[CharSounds], sources: &std::collections::HashMap<SampleKey, Option<Source>>) -> Vec<(SampleKey, Vec<u8>)> {
    chars
        .iter()
        .flat_map(|(id, sounds)| sounds.iter().map(move |(k, b)| ((id.to_string(), k.0, k.1), b)))
        .filter(|(k, _)| !sources.contains_key(k))
        .map(|(k, b)| (k, b.clone()))
        .collect()
}
