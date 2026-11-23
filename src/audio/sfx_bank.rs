use super::{easter_sfx, form_sfx, synth, wav, Audio, SR};
use crate::gfx::audio::load_sound_now;
use crate::sound_mix::Pcm;

const KINDS: &[(&str, u32)] = &[
    ("hitlight", 3), ("crowd", 1), ("hit", 3), ("hitheavy", 3), ("counter", 1), ("guard", 3), ("guardbreak", 2),
    ("swing", 3), ("swingh", 2), ("land", 2), ("dash", 1), ("jump", 1), ("super", 1), ("transform", 1),
    ("defeat", 1), ("ko", 1), ("round", 1), ("fight", 1), ("win", 1), ("perfect", 1), ("meter", 1),
    ("select", 1), ("back", 1), ("confirm", 1), ("combo1", 1), ("combo2", 1), ("combo3", 1), ("combo4", 1),
    ("combo5", 1), ("combo6", 1), ("error", 1), ("toggle", 1), ("tick", 1), ("load", 1), ("pause", 1),
    ("results", 1), ("purchase", 1), ("unlock", 1),
];

const FIGHT_KINDS: &[&str] = &[
    "hitlight", "hit", "hitheavy", "counter", "guard", "guardbreak", "swing", "swingh", "land", "dash", "jump",
    "super", "transform", "defeat", "ko", "round", "fight", "win", "perfect", "meter", "crowd",
];

fn kinds() -> impl Iterator<Item = (&'static str, u32)> {
    KINDS.iter().copied().chain(easter_sfx::KINDS.iter().copied()).chain(form_sfx::KINDS.iter().copied())
}

impl Audio {
    /// Synthesizes a built-in effect the first time it is wanted: building all of them at boot cost the web main thread over 100 ms.
    pub(super) fn ensure_builtin(&mut self, kind: &str) {
        let Some((name, variants)) = kinds().find(|(name, _)| *name == kind) else { return };
        if self.builtin_pcm.contains_key(&(name, 0)) {
            return;
        }
        for v in 0..variants {
            let mono = synth(name, v);
            if let Ok(sound) = load_sound_now(&wav(&mono)) {
                self.builtin.entry(name).or_default().push(sound);
            }
            self.builtin_pcm.insert((name, v), Pcm { rate: SR, frames: mono.iter().map(|&x| [x, x]).collect() });
        }
    }

    /// Builds the effects every fight uses while the loading screen is up; the rest wait for their first play.
    pub fn warm_fight_sfx(&mut self) {
        if !self.enabled {
            return;
        }
        for name in FIGHT_KINDS {
            self.ensure_builtin(name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmed_fight_effects_are_known_kinds_and_every_kind_synthesizes() {
        let known: Vec<&str> = kinds().map(|(name, _)| name).collect();
        assert!(FIGHT_KINDS.iter().all(|name| known.contains(name)), "a warmed kind must exist");
        let mut seen = std::collections::HashSet::new();
        assert!(known.iter().all(|name| seen.insert(*name)), "kinds are unique so lookups are unambiguous");
        for (name, variants) in kinds() {
            for v in 0..variants {
                assert!(!synth(name, v).is_empty(), "{name} variant {v} has samples");
            }
        }
    }
}
