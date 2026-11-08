use crate::audio::{Audio, SoundOpts};
use ume_engine::matchs::Match;

pub(super) fn play_awakening(m: &Match, audio: &mut Audio, fighter: u32) {
    if let Some(f) = m
        .fighters
        .iter()
        .find(|f| f.id == fighter && f.def.id == "umf:kaito")
    {
        audio.sample(
            &f.def.id,
            1000,
            3,
            f.def.sounds.get(&(1000, 3)),
            SoundOpts {
                owner: fighter,
                channel: 4,
                ..Default::default()
            },
        );
    }
}
