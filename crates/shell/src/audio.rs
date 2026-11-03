use std::rc::Rc;

pub use quad_snd::PlaySoundParams;

thread_local! {
    static CTX: quad_snd::AudioContext = quad_snd::AudioContext::new();
}

struct Guard(quad_snd::Sound);

impl Drop for Guard {
    fn drop(&mut self) { let _ = CTX.try_with(|c| self.0.delete(c)); }
}

#[derive(Clone)]
pub struct Sound(Rc<Guard>);

pub async fn load_sound_from_bytes(data: &[u8]) -> Result<Sound, String> {
    if data.is_empty() { return Err("empty sound".into()); }
    Ok(Sound(Rc::new(Guard(CTX.with(|c| quad_snd::Sound::load(c, data))))))
}

pub fn play_sound(s: &Sound, params: PlaySoundParams) { CTX.with(|c| { s.0 .0.play(c, params); }) }
pub fn play_sound_once(s: &Sound) { play_sound(s, PlaySoundParams::default()) }
pub fn stop_sound(s: &Sound) { CTX.with(|c| s.0 .0.stop(c)) }
pub fn set_sound_volume(s: &Sound, volume: f32) { CTX.with(|c| s.0 .0.set_volume(c, volume)) }
