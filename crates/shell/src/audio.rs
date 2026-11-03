use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaySoundParams { pub looped: bool, pub volume: f32 }

impl Default for PlaySoundParams {
    fn default() -> Self { PlaySoundParams { looped: false, volume: 1.0 } }
}

#[derive(Clone)]
pub struct Sound(Rc<backend::Clip>);

pub async fn load_sound_from_bytes(data: &[u8]) -> Result<Sound, String> {
    if data.is_empty() { return Err("empty sound".into()); }
    backend::load(data).await.map(|c| Sound(Rc::new(c)))
}

pub fn play_sound(s: &Sound, params: PlaySoundParams) { s.0.play(params) }
pub fn play_sound_once(s: &Sound) { play_sound(s, PlaySoundParams::default()) }
pub fn stop_sound(s: &Sound) { s.0.stop() }
pub fn set_sound_volume(s: &Sound, volume: f32) { s.0.set_volume(volume) }

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::PlaySoundParams;

    thread_local! {
        static CTX: quad_snd::AudioContext = quad_snd::AudioContext::new();
    }

    pub struct Clip(quad_snd::Sound);

    pub async fn load(data: &[u8]) -> Result<Clip, String> { Ok(Clip(CTX.with(|c| quad_snd::Sound::load(c, data)))) }

    impl Clip {
        pub fn play(&self, p: PlaySoundParams) { CTX.with(|c| { self.0.play(c, quad_snd::PlaySoundParams { looped: p.looped, volume: p.volume }); }) }
        pub fn stop(&self) { CTX.with(|c| self.0.stop(c)) }
        pub fn set_volume(&self, v: f32) { CTX.with(|c| self.0.set_volume(c, v)) }
    }

    impl Drop for Clip {
        fn drop(&mut self) { let _ = CTX.try_with(|c| self.0.delete(c)); }
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use super::PlaySoundParams;
    use std::cell::RefCell;
    use wasm_bindgen::JsCast;

    thread_local! {
        static CTX: Option<web_sys::AudioContext> = web_sys::AudioContext::new().ok();
    }

    pub struct Clip { buffer: Option<web_sys::AudioBuffer>, voices: RefCell<Vec<(web_sys::AudioBufferSourceNode, web_sys::GainNode, f64)>> }

    pub async fn load(data: &[u8]) -> Result<Clip, String> {
        let Some(promise) = CTX.with(|c| c.as_ref().map(|c| c.decode_audio_data(&js_sys::Uint8Array::from(data).buffer()))) else { return Ok(Clip { buffer: None, voices: RefCell::new(vec![]) }) };
        let promise = promise.map_err(|e| format!("{e:?}"))?;
        let buffer = wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|e| format!("{e:?}"))?;
        Ok(Clip { buffer: buffer.dyn_into().ok(), voices: RefCell::new(vec![]) })
    }

    impl Clip {
        pub fn play(&self, p: PlaySoundParams) {
            let Some(buffer) = self.buffer.as_ref() else { return };
            CTX.with(|c| {
                let Some(c) = c.as_ref() else { return };
                if c.state() == web_sys::AudioContextState::Suspended { let _ = c.resume(); }
                let (Ok(src), Ok(gain)) = (c.create_buffer_source(), c.create_gain()) else { return };
                src.set_buffer(Some(buffer));
                src.set_loop(p.looped);
                gain.gain().set_value(p.volume);
                let _ = src.connect_with_audio_node(&gain).and_then(|_| gain.connect_with_audio_node(&c.destination()));
                let _ = src.start();
                let now = c.current_time();
                let end = if p.looped { f64::INFINITY } else { now + buffer.duration() };
                let mut v = self.voices.borrow_mut();
                v.retain(|(_, _, e)| *e > now);
                v.push((src, gain, end));
            })
        }

        pub fn stop(&self) { for (s, _, _) in self.voices.borrow_mut().drain(..) { #[allow(deprecated)] let _ = s.stop(); } }

        pub fn set_volume(&self, v: f32) { for (_, g, _) in self.voices.borrow().iter() { g.gain().set_value(v); } }
    }
}
