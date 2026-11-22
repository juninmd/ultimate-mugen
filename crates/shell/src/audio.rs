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
pub fn unlock() { backend::unlock() }

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

    pub fn unlock() {}

    impl Drop for Clip {
        fn drop(&mut self) { let _ = CTX.try_with(|c| self.0.delete(c)); }
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use super::PlaySoundParams;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;

    thread_local! {
        static CTX: Option<web_sys::AudioContext> = web_sys::AudioContext::new().ok();
    }

    #[derive(Default)]
    struct Shared { buffer: RefCell<Option<web_sys::AudioBuffer>>, decoding: RefCell<bool>, queued: RefCell<Vec<PlaySoundParams>>, voices: RefCell<Vec<(web_sys::AudioBufferSourceNode, web_sys::GainNode, f64)>> }

    pub struct Clip(Rc<Shared>);

    pub async fn load(data: &[u8]) -> Result<Clip, String> {
        let shared = Rc::new(Shared::default());
        let Some(promise) = CTX.with(|c| c.as_ref().map(|c| c.decode_audio_data(&js_sys::Uint8Array::from(data).buffer()))) else { return Ok(Clip(shared)) };
        let promise = promise.map_err(|e| format!("{e:?}"))?;
        *shared.decoding.borrow_mut() = true;
        let pending = shared.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(buffer) => *pending.buffer.borrow_mut() = buffer.dyn_into().ok(),
                Err(e) => web_sys::console::error_1(&e),
            }
            *pending.decoding.borrow_mut() = false;
            let queued = std::mem::take(&mut *pending.queued.borrow_mut());
            for p in queued { start(&pending, p); }
        });
        Ok(Clip(shared))
    }

    pub fn unlock() {
        CTX.with(|c| { if let Some(c) = c.as_ref() { if c.state() == web_sys::AudioContextState::Suspended { let _ = c.resume(); } } })
    }

    fn start(shared: &Shared, p: PlaySoundParams) {
        let Some(buffer) = shared.buffer.borrow().clone() else { return };
        CTX.with(|c| {
            let Some(c) = c.as_ref() else { return };
            if c.state() == web_sys::AudioContextState::Suspended { let _ = c.resume(); }
            let (Ok(src), Ok(gain)) = (c.create_buffer_source(), c.create_gain()) else { return };
            src.set_buffer(Some(&buffer));
            src.set_loop(p.looped);
            gain.gain().set_value(p.volume);
            let _ = src.connect_with_audio_node(&gain).and_then(|_| gain.connect_with_audio_node(&c.destination()));
            let _ = src.start();
            let now = c.current_time();
            let end = if p.looped { f64::INFINITY } else { now + buffer.duration() };
            let mut v = shared.voices.borrow_mut();
            v.retain(|(_, _, e)| *e > now);
            v.push((src, gain, end));
        })
    }

    impl Clip {
        pub fn play(&self, p: PlaySoundParams) {
            if *self.0.decoding.borrow() { self.0.queued.borrow_mut().push(p); } else { start(&self.0, p); }
        }

        pub fn stop(&self) {
            self.0.queued.borrow_mut().clear();
            for (s, _, _) in self.0.voices.borrow_mut().drain(..) { #[allow(deprecated)] let _ = s.stop(); }
        }

        pub fn set_volume(&self, v: f32) {
            for p in self.0.queued.borrow_mut().iter_mut() { p.volume = v; }
            for (_, g, _) in self.0.voices.borrow().iter() { g.gain().set_value(v); }
        }
    }
}
