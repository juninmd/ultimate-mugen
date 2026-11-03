//! Audio: procedurally synthesized SFX + battle music (WAV generated at runtime), MUGEN .snd samples.
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use std::collections::HashMap;
use std::f32::consts::PI;

const SR: u32 = 32000;

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

/// State-variable filter (low / band / high-pass from one pass).
struct Svf { lp: f32, bp: f32 }
impl Svf {
    fn run(&mut self, x: f32, fc: f32, q: f32) -> (f32, f32, f32) {
        let f = 2.0 * (PI * (fc / SR as f32).min(0.45)).sin();
        let hp = x - self.lp - q * self.bp;
        self.bp += f * hp; self.lp += f * self.bp;
        (self.lp, self.bp, hp)
    }
}
/// Tiny Schroeder-style reverb (3 combs) — gives KO / super / stingers a room tail.
fn reverb(out: &mut Vec<f32>, mix: f32, tail: f32) {
    let extra = (tail * SR as f32) as usize;
    out.resize(out.len() + extra, 0.0);
    let dry = out.clone();
    for (ms, fb) in [(29.7, 0.62), (37.1, 0.58), (41.1, 0.55)] {
        let d = (ms * 0.001 * SR as f32) as usize;
        let mut buf = vec![0.0f32; d];
        let mut lp = 0.0;
        for i in 0..out.len() { let x = dry[i] + buf[i % d] * fb; lp += (x - lp) * 0.35; buf[i % d] = lp; out[i] += lp * mix * 0.45; }
    }
}

/// Layered, procedurally synthesized SFX. `v` (0..3) detunes/re-seeds so repeated hits never sound identical.
fn synth(kind: &str, v: u32) -> Vec<f32> {
    let mut nz = Noise(0x1234_5678 ^ v.wrapping_mul(0x9e37_79b9) ^ (kind.len() as u32 * 7919));
    let p = 1.0 + (v as f32 - 1.0) * 0.07;
    let sec = |s: f32| (s * SR as f32) as usize;
    let total = match kind { "ko" => 1.9, "super" => 1.4, "round" | "fight" | "win" | "perfect" => 1.1, "hitheavy" | "counter" => 0.6, "meter" => 0.5, _ => 0.4 };
    let mut out = vec![0.0f32; sec(total)];
    // pitch-swept oscillator with attack/decay envelope; wave 0 sine · 1 triangle · 2 saw · 3 square
    let tone = |out: &mut Vec<f32>, at: f32, f0: f32, f1: f32, dur: f32, g: f32, wave: u8| {
        let (o, n) = (sec(at), sec(dur));
        let mut ph = 0.0f32;
        for i in 0..n { if o + i >= out.len() { break; }
            let t = i as f32 / n as f32;
            ph += f0 * (f1 / f0).powf(t) / SR as f32; ph = ph.fract();
            let w = match wave { 0 => (ph * 2.0 * PI).sin(), 1 => 4.0 * (ph - 0.5).abs() - 1.0, 2 => ph * 2.0 - 1.0, _ => if ph < 0.5 { 1.0 } else { -1.0 } };
            out[o + i] += w * g * (t / 0.04).min(1.0) * (1.0 - t).powi(2);
        } };
    // filtered noise burst; mode 0 low-pass · 1 band-pass · 2 high-pass; fc sweeps f0→f1
    let mut noise = |out: &mut Vec<f32>, at: f32, dur: f32, g: f32, mode: u8, f0: f32, f1: f32, q: f32, shape: f32| {
        let (o, n) = (sec(at), sec(dur));
        let mut f = Svf { lp: 0.0, bp: 0.0 };
        for i in 0..n { if o + i >= out.len() { break; }
            let t = i as f32 / n as f32;
            let (lp, bp, hp) = f.run(nz.n(), f0 * (f1 / f0).powf(t), q);
            let env = if shape > 0.0 { (t * PI).sin().powf(shape) } else { (1.0 - t).powi(2) };
            out[o + i] += [lp, bp * 1.6, hp][mode as usize] * g * env;
        } };
    let metal = |out: &mut Vec<f32>, base: f32, dur: f32, g: f32| { for (k, r) in [1.0f32, 1.67, 2.64, 3.91].iter().enumerate() { let (o, n) = (0, sec(dur)); for i in 0..n.min(out.len()) { let t = i as f32 / n as f32; out[o + i] += (2.0 * PI * base * r * i as f32 / SR as f32).sin() * g / (1.0 + k as f32) * (1.0 - t).powi(3); } } };
    match kind {
        "hit" => { noise(&mut out, 0.0, 0.015, 0.9, 2, 3000.0, 3000.0, 0.7, 0.0); tone(&mut out, 0.0, 190.0 * p, 55.0, 0.13, 0.95, 0); noise(&mut out, 0.0, 0.08, 0.7, 1, 1900.0 * p, 700.0, 0.35, 0.0); tone(&mut out, 0.0, 420.0 * p, 120.0, 0.05, 0.3, 1); }
        "hitheavy" => {
            noise(&mut out, 0.0, 0.02, 1.0, 2, 2500.0, 2500.0, 0.7, 0.0); tone(&mut out, 0.0, 95.0 * p, 28.0, 0.38, 1.15, 0); tone(&mut out, 0.0, 150.0 * p, 45.0, 0.22, 0.8, 0);
            noise(&mut out, 0.0, 0.2, 0.8, 1, 1300.0 * p, 350.0, 0.3, 0.0); noise(&mut out, 0.02, 0.3, 0.45, 0, 600.0, 120.0, 0.6, 0.0); reverb(&mut out, 0.18, 0.05);
        }
        "counter" => {
            noise(&mut out, 0.0, 0.02, 1.0, 2, 3500.0, 3500.0, 0.7, 0.0); tone(&mut out, 0.0, 100.0 * p, 26.0, 0.4, 1.1, 0); noise(&mut out, 0.0, 0.2, 0.8, 1, 1500.0, 400.0, 0.3, 0.0);
            metal(&mut out, 1100.0 * p, 0.3, 0.22); tone(&mut out, 0.0, 2400.0, 3600.0, 0.18, 0.12, 0); reverb(&mut out, 0.22, 0.1);
        }
        "guard" => { metal(&mut out, 620.0 * p, 0.16, 0.3); noise(&mut out, 0.0, 0.02, 0.8, 2, 2800.0, 2800.0, 0.7, 0.0); tone(&mut out, 0.0, 240.0, 120.0, 0.06, 0.4, 1); }
        "swing" => { noise(&mut out, 0.0, 0.16, 0.5, 1, 500.0 * p, 2800.0, 0.5, 1.5); }
        "swingh" => { noise(&mut out, 0.0, 0.26, 0.6, 1, 260.0 * p, 1900.0, 0.45, 1.3); tone(&mut out, 0.0, 90.0, 160.0, 0.2, 0.18, 0); }
        "land" => { tone(&mut out, 0.0, 100.0, 38.0, 0.12, 0.55, 0); noise(&mut out, 0.0, 0.09, 0.45, 0, 700.0, 200.0, 0.8, 0.0); }
        "dash" => { noise(&mut out, 0.0, 0.2, 0.35, 1, 2200.0, 500.0, 0.5, 1.2); }
        "jump" => { noise(&mut out, 0.0, 0.1, 0.25, 1, 700.0, 1800.0, 0.5, 1.0); tone(&mut out, 0.0, 280.0, 520.0, 0.08, 0.12, 1); }
        "super" => {
            tone(&mut out, 0.0, 90.0, 900.0, 0.55, 0.28, 2); noise(&mut out, 0.0, 0.55, 0.5, 1, 300.0, 4200.0, 0.4, 0.8);
            tone(&mut out, 0.55, 80.0, 24.0, 0.7, 1.2, 0); noise(&mut out, 0.55, 0.5, 0.8, 1, 1600.0, 200.0, 0.3, 0.0); metal(&mut out, 520.0, 0.5, 0.0); reverb(&mut out, 0.3, 0.3);
        }
        "ko" => {
            noise(&mut out, 0.0, 0.03, 1.0, 2, 2000.0, 2000.0, 0.7, 0.0); tone(&mut out, 0.0, 80.0, 22.0, 0.9, 1.3, 0); tone(&mut out, 0.0, 200.0, 40.0, 0.4, 0.8, 0);
            noise(&mut out, 0.0, 0.6, 0.7, 1, 1200.0, 150.0, 0.3, 0.0); metal(&mut out, 300.0, 0.9, 0.12); reverb(&mut out, 0.38, 0.5);
        }
        "round" => { for (f, g) in [(110.0, 0.3), (165.0, 0.22), (220.0, 0.18)] { tone(&mut out, 0.0, f, f * 1.005, 0.6, g, 2); } noise(&mut out, 0.0, 0.5, 0.2, 1, 200.0, 1200.0, 0.5, 1.4); tone(&mut out, 0.0, 70.0, 35.0, 0.3, 0.6, 0); reverb(&mut out, 0.3, 0.3); }
        "fight" => { noise(&mut out, 0.0, 0.18, 0.5, 1, 400.0, 3500.0, 0.4, 1.2); tone(&mut out, 0.0, 200.0, 420.0, 0.18, 0.25, 2); tone(&mut out, 0.17, 110.0, 30.0, 0.5, 1.0, 0); noise(&mut out, 0.17, 0.25, 0.8, 1, 1800.0, 300.0, 0.3, 0.0); tone(&mut out, 0.17, 330.0, 330.0, 0.5, 0.15, 3); reverb(&mut out, 0.28, 0.25); }
        "win" => { for (i, f) in [523.25f32, 659.25, 783.99, 1046.5].iter().enumerate() { tone(&mut out, i as f32 * 0.1, *f, *f, 0.5, 0.2, 1); tone(&mut out, i as f32 * 0.1, f * 2.0, f * 2.0, 0.3, 0.05, 0); } reverb(&mut out, 0.35, 0.35); }
        "perfect" => { for i in 0..6 { let f = 880.0 * 1.2599f32.powi(i); tone(&mut out, i as f32 * 0.055, f, f, 0.22, 0.14, 0); } reverb(&mut out, 0.3, 0.3); }
        "meter" => { tone(&mut out, 0.0, 1480.0, 1480.0, 0.16, 0.22, 0); tone(&mut out, 0.07, 2217.0, 2217.0, 0.25, 0.16, 0); reverb(&mut out, 0.3, 0.2); }
        "select" => { tone(&mut out, 0.0, 1500.0, 1050.0, 0.03, 0.2, 0); noise(&mut out, 0.0, 0.01, 0.2, 2, 4000.0, 4000.0, 0.7, 0.0); }
        "back" => { tone(&mut out, 0.0, 700.0, 380.0, 0.08, 0.2, 1); }
        "confirm" => { tone(&mut out, 0.0, 660.0, 660.0, 0.09, 0.22, 1); tone(&mut out, 0.07, 990.0, 990.0, 0.16, 0.22, 1); noise(&mut out, 0.0, 0.01, 0.2, 2, 4000.0, 4000.0, 0.7, 0.0); reverb(&mut out, 0.2, 0.1); }
        k if k.starts_with("combo") => { let n: i32 = k[5..].parse().unwrap_or(1); let f = 640.0 * 1.0595f32.powi(n * 2); tone(&mut out, 0.0, f, f, 0.1, 0.18, 0); tone(&mut out, 0.0, f * 2.0, f * 2.0, 0.07, 0.07, 0); }
        _ => {}
    }
    // gentle soft clip so layered peaks stay punchy, not harsh
    for x in out.iter_mut() { *x = (*x * 0.9).tanh(); }
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
    builtin: HashMap<&'static str, Vec<Sound>>,
    rr: std::cell::Cell<u32>,
    /// music ducking (1 = full): lowered during supers / KO, eased back each frame
    pub duck: f32, duck_now: f32,
    samples: HashMap<(String, i32, i32), Option<Sound>>,
    pending: Vec<(String, i32, i32, Vec<u8>, f32)>,
    music: HashMap<u32, Sound>,
    playing: Option<u32>,
    pub volume: f32,
}

impl Audio {
    pub async fn new(enabled: bool) -> Self {
        let mut a = Audio { enabled, builtin: HashMap::new(), rr: std::cell::Cell::new(0), duck: 1.0, duck_now: 1.0, samples: HashMap::new(), pending: vec![], music: HashMap::new(), playing: None, volume: 0.8 };
        if enabled {
            // hits / guards / swings get 3 variants (round-robin) so repeated impacts don't sound identical
            for (k, n) in [("hit", 3u32), ("hitheavy", 3), ("counter", 1), ("guard", 3), ("swing", 3), ("swingh", 2), ("land", 2), ("dash", 1), ("jump", 1), ("super", 1), ("ko", 1), ("round", 1), ("fight", 1), ("win", 1),
                ("perfect", 1), ("meter", 1), ("select", 1), ("back", 1), ("confirm", 1), ("combo1", 1), ("combo2", 1), ("combo3", 1), ("combo4", 1), ("combo5", 1), ("combo6", 1)] {
                for v in 0..n {
                    if let Ok(s) = load_sound_from_bytes(&wav(&synth(k, v))).await { a.builtin.entry(k).or_default().push(s); }
                }
            }
        }
        a
    }
    pub fn sfx(&self, k: &str) {
        if !self.enabled { return; }
        if let Some(v) = self.builtin.get(k) {
            let i = self.rr.get().wrapping_mul(1664525).wrapping_add(1013904223); self.rr.set(i);
            play_sound(&v[(i >> 16) as usize % v.len()], PlaySoundParams { looped: false, volume: self.volume });
        }
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
        // ease the music toward the ducking target (supers / KO pull it down so the effects hit harder)
        if self.playing.is_some() {
            self.duck_now += (self.duck - self.duck_now) * 0.12;
            if let Some(s) = self.playing.and_then(|id| self.music.get(&id)) { macroquad::audio::set_sound_volume(s, self.volume * 0.35 * self.duck_now); }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    fn energy(v: &[f32]) -> f32 { v.iter().map(|x| x * x).sum::<f32>() / v.len().max(1) as f32 }
    #[test]
    fn every_effect_is_audible_finite_and_clipped() {
        for k in ["hit", "hitheavy", "counter", "guard", "swing", "swingh", "land", "dash", "jump", "super", "ko", "round", "fight", "win", "perfect", "meter", "select", "back", "confirm", "combo1", "combo6"] {
            let s = synth(k, 0);
            assert!(!s.is_empty() && s.iter().all(|x| x.is_finite() && x.abs() <= 1.0), "{k}: finite, within [-1,1]");
            assert!(energy(&s) > 1e-5, "{k}: not silent");
        }
    }
    #[test]
    fn variants_differ_and_heavy_hits_are_bigger() {
        let (a, b) = (synth("hit", 0), synth("hit", 2));
        assert!(a.iter().zip(&b).any(|(x, y)| (x - y).abs() > 1e-3), "round-robin variants are not identical");
        let (h, l) = (synth("hitheavy", 1), synth("hit", 1));
        assert!(energy(&h) > energy(&l), "heavy hit carries more energy");
        assert!(synth("ko", 0).len() > synth("hit", 0).len() * 3, "KO has a long tail");
    }
}
