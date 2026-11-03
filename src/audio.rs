//! Audio: procedurally synthesized SFX + battle music (WAV generated at runtime), MUGEN .snd samples.
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use std::collections::HashMap;
use std::f32::consts::PI;

const SR: u32 = 22050;

fn wav(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let mut v = Vec::with_capacity(44 + n as usize * 2);
    v.extend_from_slice(b"RIFF"); v.extend_from_slice(&(36 + n * 2).to_le_bytes()); v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes()); v.extend_from_slice(&1u16.to_le_bytes()); v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&SR.to_le_bytes()); v.extend_from_slice(&(SR * 2).to_le_bytes()); v.extend_from_slice(&2u16.to_le_bytes()); v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data"); v.extend_from_slice(&(n * 2).to_le_bytes());
    for s in samples { v.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes()); }
    v
}
struct Noise(u32);
impl Noise { fn n(&mut self) -> f32 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 17; self.0 ^= self.0 << 5; self.0 as f32 / 2147483648.0 - 1.0 } }

fn synth(kind: &str) -> Vec<f32> {
    let mut nz = Noise(0x1234_5678);
    let len = |s: f32| (s * SR as f32) as usize;
    let mut out = vec![0.0f32; len(if kind == "ko" { 0.9 } else if kind == "super" { 0.8 } else { 0.35 })];
    let thump = |out: &mut Vec<f32>, f0: f32, f1: f32, dur: f32, g: f32| {
        let mut ph = 0.0;
        for i in 0..len(dur).min(out.len()) {
            let t = i as f32 / len(dur) as f32;
            let f = f0 * (f1 / f0).powf(t);
            ph += 2.0 * PI * f / SR as f32;
            out[i] += ph.sin() * g * (1.0 - t).powi(2);
        }
    };
    let mut burst = |out: &mut Vec<f32>, dur: f32, g: f32, lp: f32| {
        let mut y = 0.0;
        for i in 0..len(dur).min(out.len()) {
            let t = i as f32 / len(dur) as f32;
            y += (nz.n() - y) * lp;
            out[i] += y * g * (1.0 - t).powi(2);
        }
    };
    match kind {
        "hit" => { thump(&mut out, 180.0, 60.0, 0.12, 0.9); burst(&mut out, 0.09, 0.9, 0.6); }
        "hitheavy" => { thump(&mut out, 140.0, 40.0, 0.25, 1.1); burst(&mut out, 0.18, 1.0, 0.35); burst(&mut out, 0.08, 0.4, 0.9); }
        "guard" => { thump(&mut out, 900.0, 500.0, 0.06, 0.35); burst(&mut out, 0.06, 0.6, 0.95); }
        "swing" => { burst(&mut out, 0.12, 0.3, 0.25); }
        "land" => { thump(&mut out, 90.0, 40.0, 0.1, 0.4); }
        "super" => {
            let mut ph = 0.0;
            for i in 0..out.len() { let t = i as f32 / out.len() as f32; ph += 2.0 * PI * (120.0 * (7.5f32).powf(t.min(0.6) / 0.6)) / SR as f32; out[i] += ((ph.sin() > 0.0) as i32 as f32 * 2.0 - 1.0) * 0.18 * (1.0 - t); }
            burst(&mut out, 0.6, 0.35, 0.5);
        }
        "ko" => { thump(&mut out, 220.0, 30.0, 0.8, 1.2); burst(&mut out, 0.5, 0.6, 0.15); }
        "select" => { thump(&mut out, 880.0, 660.0, 0.05, 0.25); }
        "confirm" => { thump(&mut out, 660.0, 1320.0, 0.12, 0.3); }
        _ => {}
    }
    out
}

fn music(seed: u32) -> Vec<f32> {
    let bpm = 138.0 + (seed % 3) as f32 * 8.0;
    let step = 60.0 / bpm / 4.0;
    let scales: [&[i32]; 3] = [&[0, 3, 5, 7, 10, 12], &[0, 2, 3, 7, 8, 12], &[0, 3, 5, 6, 7, 10]];
    let sc = scales[seed as usize % 3];
    let root = 41 + (seed % 5) as i32;
    let prog = [0, 0, 5, 3, 0, 0, 7, 5];
    let steps = 16 * 8;
    let total = (steps as f32 * step * SR as f32) as usize;
    let mut out = vec![0.0f32; total];
    let midi = |n: i32| 440.0 * 2f32.powf((n - 69) as f32 / 12.0);
    let mut nz = Noise(seed.wrapping_mul(7919) | 1);
    for i in 0..steps {
        let bar = (i / 16) % 8; let s = i % 16;
        let t0 = (i as f32 * step * SR as f32) as usize;
        let r = root + prog[bar];
        let mut add = |f: &dyn Fn(f32) -> f32, dur: f32| { let n = (dur * SR as f32) as usize; for k in 0..n { let idx = (t0 + k) % total; out[idx] += f(k as f32 / SR as f32); } };
        if s % 4 == 0 { add(&|t| (2.0 * PI * (150.0 * (0.25f32).powf(t / 0.15)) * t).sin() * (1.0 - t / 0.18).max(0.0) * 0.9, 0.18); }
        if s % 8 == 4 { let ns: Vec<f32> = (0..(0.15 * SR as f32) as usize).map(|_| nz.n()).collect(); add(&|t| ns[((t * SR as f32) as usize).min(ns.len() - 1)] * (1.0 - t / 0.15).max(0.0) * 0.45, 0.15); }
        if s % 2 == 0 { let ns: Vec<f32> = (0..(0.04 * SR as f32) as usize).map(|_| nz.n()).collect(); add(&|t| { let i = ((t * SR as f32) as usize).min(ns.len() - 1); (ns[i] - if i > 0 { ns[i - 1] } else { 0.0 }) * (1.0 - t / 0.04).max(0.0) * 0.12 }, 0.04); }
        if s % 2 == 0 {
            let f = midi(r - 12 + if s % 8 == 6 { 12 } else { 0 });
            add(&|t| { let ph = (t * f).fract(); (ph * 2.0 - 1.0) * 0.22 * (1.0 - t / (step * 1.8)).max(0.0) }, step * 1.8);
        }
        if s % 2 == 1 || s % 3 == 0 {
            let f = midi(r + 12 + sc[(s * 3 + bar) % sc.len()]);
            add(&|t| { let sq = if (t * f).fract() < 0.5 { 1.0 } else { -1.0 }; sq * 0.06 * (1.0 - t / (step * 0.9)).max(0.0) }, step * 0.9);
        }
    }
    // gentle low-pass
    let mut y = 0.0;
    for s in out.iter_mut() { y += (*s - y) * 0.45; *s = y * 0.7; }
    out
}

pub struct Audio {
    pub enabled: bool,
    builtin: HashMap<&'static str, Sound>,
    samples: HashMap<(String, i32, i32), Option<Sound>>,
    pending: Vec<(String, i32, i32, Vec<u8>, f32)>,
    music: HashMap<u32, Sound>,
    playing: Option<u32>,
    pub volume: f32,
}

impl Audio {
    pub async fn new(enabled: bool) -> Self {
        let mut a = Audio { enabled, builtin: HashMap::new(), samples: HashMap::new(), pending: vec![], music: HashMap::new(), playing: None, volume: 0.8 };
        if enabled {
            for k in ["hit", "hitheavy", "guard", "swing", "land", "super", "ko", "select", "confirm"] {
                if let Ok(s) = load_sound_from_bytes(&wav(&synth(k))).await { a.builtin.insert(k, s); }
            }
        }
        a
    }
    pub fn sfx(&self, k: &str) {
        if !self.enabled { return; }
        if let Some(s) = self.builtin.get(k) { play_sound(s, PlaySoundParams { looped: false, volume: self.volume }); }
    }
    /// Play a MUGEN sample (decoded lazily; first play is deferred to `process`).
    pub fn sample(&mut self, def_id: &str, g: i32, n: i32, wavb: Option<&Vec<u8>>, vol: f32) {
        if !self.enabled { return; }
        let key = (def_id.to_string(), g, n);
        match self.samples.get(&key) {
            Some(Some(s)) => play_sound(s, PlaySoundParams { looped: false, volume: self.volume * vol.min(1.5) }),
            Some(None) => {}
            None => if let Some(b) = wavb { self.pending.push((key.0.clone(), g, n, b.clone(), vol)); self.samples.insert(key, None); },
        }
    }
    pub async fn process(&mut self) {
        let list: Vec<_> = self.pending.drain(..).collect();
        for (id, g, n, bytes, vol) in list {
            if let Ok(s) = load_sound_from_bytes(&bytes).await {
                play_sound(&s, PlaySoundParams { looped: false, volume: self.volume * vol.min(1.5) });
                self.samples.insert((id, g, n), Some(s));
            }
        }
    }
    /// Synthesize + decode a track ahead of time (done behind the VS/loading screen so the fight starts instantly).
    pub async fn prepare_music(&mut self, id: u32) {
        if !self.enabled || self.music.contains_key(&id) { return; }
        if let Ok(s) = load_sound_from_bytes(&wav(&music(id))).await { self.music.insert(id, s); }
    }
    pub async fn play_music(&mut self, id: u32) {
        if !self.enabled { return; }
        self.stop_music();
        self.prepare_music(id).await;
        if let Some(s) = self.music.get(&id) { play_sound(s, PlaySoundParams { looped: true, volume: self.volume * 0.35 }); self.playing = Some(id); }
    }
    pub fn stop_music(&mut self) {
        if let Some(id) = self.playing.take() { if let Some(s) = self.music.get(&id) { stop_sound(s); } }
    }
}
