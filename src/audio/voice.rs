use super::{Audio, Source, SoundOpts};
use crate::sound_mix::Pcm;

impl Audio {
    pub fn play_clip(&mut self, bank: &str, index: i32, pcm: &Pcm, owner: u32, volume: f32) {
        if !self.enabled || volume <= 0.0 {
            return;
        }
        let def = format!("voice:{bank}");
        let key = (def.clone(), 0, index);
        if !matches!(self.sources.get(&key), Some(Some(_))) {
            self.sources.insert(key, Some(Source::Pcm(pcm.clone())));
        }
        self.sample(&def, 0, index, None, SoundOpts { owner, channel: 7, volume, ..Default::default() });
    }
}
