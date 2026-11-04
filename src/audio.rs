//! Audio: procedurally synthesized SFX + battle music (WAV generated at runtime), MUGEN .snd samples.
use crate::sound_mix::{decode, pan_bucket, Pcm};
use crate::gfx::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};
use std::collections::HashMap;
use ume_engine::matchs::BgmAction;
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
    let total = match kind { "transform" => 2.6, "ko" => 1.9, "super" => 2.1, "defeat" => 1.7, "round" | "fight" | "win" | "perfect" => 1.1, "hitheavy" | "counter" => 0.6, "meter" => 0.5, _ => 0.4 };
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
            tone(&mut out, 0.55, 80.0, 24.0, 0.7, 1.2, 0); noise(&mut out, 0.55, 0.5, 0.8, 1, 1600.0, 200.0, 0.3, 0.0); metal(&mut out, 520.0, 0.5, 0.0);
            // grander finish: a second, deeper boom with a rumbling tail
            tone(&mut out, 1.05, 60.0, 18.0, 0.95, 1.1, 0); noise(&mut out, 1.05, 0.9, 0.7, 0, 900.0, 90.0, 0.3, 0.0); metal(&mut out, 340.0, 1.0, 0.0);
            tone(&mut out, 0.0, 40.0, 160.0, 1.0, 0.35, 0); reverb(&mut out, 0.38, 0.4);
        }
        "transform" => {
            for (i, r) in [1.0f32, 1.5, 2.0, 3.0].iter().enumerate() { tone(&mut out, 0.0, 70.0 * r, 700.0 * r, 1.3, 0.16 - i as f32 * 0.025, 2); }
            noise(&mut out, 0.0, 1.3, 0.55, 1, 200.0, 5200.0, 0.4, 0.8);
            tone(&mut out, 1.3, 90.0, 20.0, 1.1, 1.3, 0); tone(&mut out, 1.3, 220.0, 50.0, 0.5, 0.7, 0);
            noise(&mut out, 1.3, 0.9, 0.9, 1, 2200.0, 160.0, 0.3, 0.0); metal(&mut out, 420.0, 1.2, 0.0); reverb(&mut out, 0.42, 0.5);
        }
        "defeat" => {
            tone(&mut out, 0.0, 440.0, 220.0, 0.6, 0.22, 1); tone(&mut out, 0.25, 330.0, 110.0, 0.9, 0.22, 1); tone(&mut out, 0.0, 120.0, 40.0, 1.2, 0.5, 0);
            noise(&mut out, 0.0, 0.8, 0.25, 0, 900.0, 120.0, 0.3, 0.0); reverb(&mut out, 0.4, 0.5);
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundOpts {
    pub volume: f32,
    pub pan: f32,
    pub rate: f32,
    pub looping: bool,
    pub owner: u32,
    pub channel: i32,
    pub low_priority: bool,
    pub loop_start: u32,
    pub loop_end: u32,
    pub loop_count: i32,
}

impl Default for SoundOpts {
    fn default() -> Self {
        SoundOpts { volume: 1.0, pan: 0.0, rate: 1.0, looping: false, owner: 0, channel: -1, low_priority: false, loop_start: 0, loop_end: 0, loop_count: 0 }
    }
}

fn loop_frames(opts: &SoundOpts, rate: u32, len: usize) -> (usize, usize) {
    if opts.loop_start == 0 && opts.loop_end == 0 { return (0, len); }
    let to_frames = |ms: u32| ((ms as u64 * rate.max(1) as u64) / 1000) as usize;
    let start = to_frames(opts.loop_start).min(len);
    let end = if opts.loop_end > opts.loop_start { to_frames(opts.loop_end).min(len) } else { len };
    (start.min(end), end)
}

fn loop_expired(opts: &SoundOpts, elapsed: usize, start: usize, end: usize) -> bool {
    if !opts.looping || opts.loop_count <= 0 || end <= start { return false; }
    elapsed >= end + (opts.loop_count as usize - 1) * (end - start)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MusicLoop {
    pub start: usize,
    pub end: usize,
}

type SampleKey = (String, i32, i32);
type VoiceKey = (SampleKey, i8, u16);
pub type CharSounds<'a> = (&'a str, &'a HashMap<(i32, i32), Vec<u8>>);

enum Source {
    Pcm(Pcm),
    Raw(Vec<u8>),
}

struct Voice {
    sound: Sound,
    end: f64,
    start: f64,
    key: Option<SampleKey>,
    opts: SoundOpts,
}

struct Music {
    intro: Option<Sound>,
    body: Sound,
    switch_at: Option<f64>,
}

pub struct Audio {
    pub enabled: bool,
    builtin: HashMap<&'static str, Vec<Sound>>,
    builtin_pcm: HashMap<(&'static str, u32), Pcm>,
    builtin_pan: HashMap<(&'static str, u32, i8), Option<Sound>>,
    rr: std::cell::Cell<u32>,
    pub duck: f32, duck_now: f32,
    sources: HashMap<SampleKey, Option<Source>>,
    voices: HashMap<VoiceKey, Option<(Sound, f64)>>,
    pending: Vec<(VoiceKey, SoundOpts)>,
    pending_sfx: Vec<(&'static str, u32, i8)>,
    channels: HashMap<(u32, i32), Voice>,
    repan: Vec<((u32, i32), f32)>,
    loop_seq: i32,
    music: HashMap<u32, Sound>,
    music_wav: HashMap<u32, Vec<u8>>,
    music_job: Option<(u32, std::thread::JoinHandle<Vec<u8>>)>,
    preload_job: Option<std::thread::JoinHandle<Vec<(SampleKey, Result<Pcm, Vec<u8>>)>>>,
    playing: Option<Music>,
    music_files: HashMap<String, (Vec<u8>, MusicLoop)>,
    pending_bgm: Vec<BgmAction>,
    bgm_gain: f32,
    bgm_looping: bool,
    pub volume: f32,
    pub music_volume: f32,
}

fn rate_key(rate: f32) -> u16 {
    (rate.clamp(0.05, 8.0) * 100.0).round() as u16
}

impl Audio {
    pub async fn new(enabled: bool) -> Self {
        let mut a = Audio {
            enabled, builtin: HashMap::new(), builtin_pcm: HashMap::new(), builtin_pan: HashMap::new(), rr: std::cell::Cell::new(0), duck: 1.0, duck_now: 1.0,
            sources: HashMap::new(), voices: HashMap::new(), pending: vec![], pending_sfx: vec![], channels: HashMap::new(), repan: vec![], loop_seq: -1, music: HashMap::new(), music_wav: HashMap::new(), music_job: None, preload_job: None, playing: None, music_files: HashMap::new(), pending_bgm: vec![], bgm_gain: 1.0, bgm_looping: true, volume: 0.8, music_volume: 1.0,
        };
        if enabled {
            for (k, n) in [("hit", 3u32), ("hitheavy", 3), ("counter", 1), ("guard", 3), ("swing", 3), ("swingh", 2), ("land", 2), ("dash", 1), ("jump", 1), ("super", 1), ("transform", 1), ("defeat", 1), ("ko", 1), ("round", 1), ("fight", 1), ("win", 1),
                ("perfect", 1), ("meter", 1), ("select", 1), ("back", 1), ("confirm", 1), ("combo1", 1), ("combo2", 1), ("combo3", 1), ("combo4", 1), ("combo5", 1), ("combo6", 1)] {
                for v in 0..n {
                    let mono = synth(k, v);
                    if let Ok(s) = load_sound_from_bytes(&wav(&mono)).await { a.builtin.entry(k).or_default().push(s); }
                    a.builtin_pcm.insert((k, v), Pcm { rate: SR, frames: mono.iter().map(|&x| [x, x]).collect() });
                }
            }
        }
        a
    }
    pub fn set_levels(&mut self, master: f32, music: f32, sfx: f32) { self.volume = master * sfx; self.music_volume = master * music; }
    pub fn sfx(&mut self, k: &str) {
        self.sfx_at(k, 0.0);
    }
    pub fn sfx_at(&mut self, k: &str, pan: f32) {
        if !self.enabled { return; }
        let Some((&name, v)) = self.builtin.get_key_value(k) else { return };
        let i = self.rr.get().wrapping_mul(1664525).wrapping_add(1013904223); self.rr.set(i);
        let variant = (i >> 16) % v.len() as u32;
        let bucket = pan_bucket(pan);
        if bucket == 0 {
            play_sound(&v[variant as usize], PlaySoundParams { looped: false, volume: self.volume });
            return;
        }
        match self.builtin_pan.get(&(name, variant, bucket)) {
            Some(Some(s)) => play_sound(s, PlaySoundParams { looped: false, volume: self.volume }),
            Some(None) => {}
            None => { self.builtin_pan.insert((name, variant, bucket), None); self.pending_sfx.push((name, variant, bucket)); }
        }
    }
    pub fn sample(&mut self, def_id: &str, g: i32, n: i32, wavb: Option<&Vec<u8>>, opts: SoundOpts) {
        if !self.enabled { return; }
        let key: SampleKey = (def_id.to_string(), g, n);
        if !self.sources.contains_key(&key) {
            let Some(b) = wavb else { return };
            let src = decode(b).map(Source::Pcm).unwrap_or_else(|| Source::Raw(b.clone()));
            self.sources.insert(key.clone(), Some(src));
        }
        let raw = matches!(self.sources.get(&key), Some(Some(Source::Raw(_))));
        let vkey = (key, if raw { 0 } else { pan_bucket(opts.pan) }, if raw { 100 } else { rate_key(opts.rate) });
        match self.voices.get(&vkey) {
            Some(Some((s, secs))) => { let (s, secs) = (s.clone(), *secs); self.start(&s, secs, opts, Some(vkey.0.clone())); }
            Some(None) => {}
            None => { self.voices.insert(vkey.clone(), None); self.pending.push((vkey, opts)); }
        }
    }
    /// Decodes the match's samples off the render thread (inline on wasm). Returns true once everything is in `sources`.
    pub fn preload_match(&mut self, chars: &[CharSounds]) -> bool {
        if !self.enabled { return true; }
        // Browsers have no worker threads here: decoding every sample of an HD pack blocks the page for ~1 s.
        // Samples are decoded on first play instead (a few ms each), so the VS screen stays responsive.
        #[cfg(target_arch = "wasm32")]
        {
            self.sources.retain(|k, _| chars.iter().any(|(id, _)| *id == k.0));
            self.voices.retain(|k, _| chars.iter().any(|(id, _)| *id == k.0 .0));
            return true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(h) = self.preload_job.take() {
            if !h.is_finished() { self.preload_job = Some(h); return false; }
            for (key, r) in h.join().unwrap_or_default() { self.sources.insert(key, Some(r.map(Source::Pcm).unwrap_or_else(Source::Raw))); }
            return true;
        }
        self.sources.retain(|k, _| chars.iter().any(|(id, _)| *id == k.0));
        self.voices.retain(|k, _| chars.iter().any(|(id, _)| *id == k.0 .0));
        let todo: Vec<(SampleKey, Vec<u8>)> = chars.iter()
            .flat_map(|(id, sounds)| sounds.iter().map(move |(k, b)| ((id.to_string(), k.0, k.1), b)))
            .filter(|(k, _)| !self.sources.contains_key(k))
            .map(|(k, b)| (k, b.clone()))
            .collect();
        if todo.is_empty() { return true; }
        let work = move || { let decoded = ume_formats::par::map(&todo, |(_, b)| decode(b)); todo.into_iter().zip(decoded).map(|((k, b), pcm)| (k, pcm.ok_or(b))).collect::<Vec<_>>() };
        if let Ok(h) = std::thread::Builder::new().name("sfx-decode".into()).spawn(work) { self.preload_job = Some(h); false } else { true }
    }
    fn start(&mut self, s: &Sound, secs: f64, o: SoundOpts, key: Option<SampleKey>) {
        let now = crate::gfx::get_time();
        if o.channel >= 0 && o.low_priority && self.channels.get(&(o.owner, o.channel)).is_some_and(|v| v.end > now) { return; }
        let slot = if o.channel >= 0 { Some(o.channel) } else if o.looping { self.loop_seq -= 1; Some(self.loop_seq) } else { None };
        if let Some(ch) = slot {
            let end = if o.looping { f64::INFINITY } else { now + secs };
            let voice = Voice { sound: s.clone(), end, start: now, key, opts: o };
            if let Some(old) = self.channels.insert((o.owner, ch), voice) { stop_sound(&old.sound); }
        }
        play_sound(s, PlaySoundParams { looped: o.looping, volume: self.volume * o.volume.clamp(0.0, 1.5) });
    }
    pub fn stop(&mut self, owner: u32, channel: i32) {
        self.channels.retain(|&(o, ch), v| {
            let hit = o == owner && (channel < 0 || ch == channel);
            if hit { stop_sound(&v.sound); }
            !hit
        });
    }
    pub fn stop_all(&mut self) {
        for (_, v) in self.channels.drain() { stop_sound(&v.sound); }
    }
    pub fn set_volume(&mut self, owner: u32, channel: i32, volume: f32) {
        let now = crate::gfx::get_time();
        for (&k, v) in self.channels.iter_mut() {
            if k.0 == owner && (channel < 0 || k.1 == channel) && v.end > now {
                v.opts.volume = volume;
                set_sound_volume(&v.sound, self.volume * volume.clamp(0.0, 1.5));
            }
        }
    }
    pub fn pan(&mut self, owner: u32, channel: i32, pan: f32) {
        let now = crate::gfx::get_time();
        for (&k, v) in self.channels.iter_mut() {
            if k.0 == owner && (channel < 0 || k.1 == channel) && v.end > now && pan_bucket(pan) != pan_bucket(v.opts.pan) {
                v.opts.pan = pan;
                self.repan.push((k, pan));
            }
        }
    }
    async fn apply_repan(&mut self) {
        let now = crate::gfx::get_time();
        let mut stop = vec![];
        for (k, pan) in std::mem::take(&mut self.repan) {
            let Some(v) = self.channels.get(&k) else { continue };
            let Some(Some(Source::Pcm(p))) = v.key.as_ref().and_then(|key| self.sources.get(key)) else { continue };
            let (start_frame, end_frame) = loop_frames(&v.opts, p.rate, p.frames.len());
            let rendered = p.render(1.0, pan, v.opts.rate);
            let elapsed = ((now - v.start) * rendered.rate as f64).max(0.0) as usize;
            if loop_expired(&v.opts, elapsed, start_frame, end_frame) { stop.push(k); continue; }
            let Some(clip) = rendered.resume_segment(elapsed, v.opts.looping, start_frame, end_frame) else { stop.push(k); continue };
            let Ok(s) = load_sound_from_bytes(&clip.to_wav()).await else { continue };
            let Some(v) = self.channels.get_mut(&k) else { continue };
            stop_sound(&v.sound);
            play_sound(&s, PlaySoundParams { looped: v.opts.looping, volume: self.volume * v.opts.volume.clamp(0.0, 1.5) });
            v.sound = s;
        }
        for k in stop { if let Some(v) = self.channels.remove(&k) { stop_sound(&v.sound); } }
    }
    /// Retune the loop window (in milliseconds) and loop count of every live channel of `owner`.
    pub fn set_loop(&mut self, owner: u32, channel: i32, start: u32, end: u32, count: i32) {
        let mut keys = vec![];
        for (&k, v) in self.channels.iter_mut() {
            if k.0 == owner && (channel < 0 || k.1 == channel) {
                v.opts.loop_start = start;
                v.opts.loop_end = end;
                v.opts.loop_count = count;
                keys.push((k, v.opts.pan));
            }
        }
        self.repan.extend(keys);
    }
    pub async fn process(&mut self) {
        self.apply_repan().await;
        self.run_pending_bgm().await;
        if let Some(m) = self.playing.as_mut() {
            self.duck_now += (self.duck - self.duck_now) * 0.12;
            let v = self.music_volume * 0.35 * self.duck_now * self.bgm_gain;
            if let Some(at) = m.switch_at {
                if crate::gfx::get_time() >= at {
                    m.switch_at = None;
                    if let Some(i) = m.intro.take() { stop_sound(&i); }
                    play_sound(&m.body, PlaySoundParams { looped: true, volume: v });
                }
            }
            set_sound_volume(m.intro.as_ref().unwrap_or(&m.body), v);
        }
        for (name, variant, bucket) in std::mem::take(&mut self.pending_sfx) {
            let Some(p) = self.builtin_pcm.get(&(name, variant)) else { continue };
            let bytes = p.render(1.0, bucket as f32 / 5.0, 1.0).to_wav();
            if let Ok(s) = load_sound_from_bytes(&bytes).await {
                play_sound(&s, PlaySoundParams { looped: false, volume: self.volume });
                self.builtin_pan.insert((name, variant, bucket), Some(s));
            }
        }
        for (vkey, opts) in std::mem::take(&mut self.pending) {
            let (bytes, secs) = match self.sources.get(&vkey.0) {
                Some(Some(Source::Pcm(p))) => { let r = p.render(1.0, vkey.1 as f32 / 5.0, vkey.2 as f32 / 100.0); (r.to_wav(), r.seconds()) }
                Some(Some(Source::Raw(b))) => (b.clone(), 0.5),
                _ => continue,
            };
            if let Ok(s) = load_sound_from_bytes(&bytes).await {
                self.start(&s, secs, opts, Some(vkey.0.clone()));
                self.voices.insert(vkey, Some((s, secs)));
            }
        }
    }
    /// Synthesizes the track off the render thread (inline on wasm). Returns true once `prepare_music` is cheap.
    pub fn music_ready(&mut self, id: u32) -> bool {
        if !self.enabled || self.music.contains_key(&id) || self.music_wav.contains_key(&id) { return true; }
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self.music_job.take() {
                Some((job_id, h)) if job_id == id => {
                    if !h.is_finished() { self.music_job = Some((id, h)); return false; }
                    if let Ok(bytes) = h.join() { self.music_wav.insert(id, bytes); }
                    return true;
                }
                _ => {}
            }
            match std::thread::Builder::new().name("music-synth".into()).spawn(move || wav(&music(id))) {
                Ok(h) => { self.music_job = Some((id, h)); false }
                Err(_) => true,
            }
        }
        #[cfg(target_arch = "wasm32")]
        true
    }
    pub async fn prepare_music(&mut self, id: u32) {
        if !self.enabled || self.music.contains_key(&id) { return; }
        let bytes = self.music_wav.remove(&id).unwrap_or_else(|| wav(&music(id)));
        if let Ok(s) = load_sound_from_bytes(&bytes).await { self.music.insert(id, s); }
    }
    pub async fn play_music(&mut self, id: u32) {
        if !self.enabled { return; }
        self.stop_music();
        self.prepare_music(id).await;
        if let Some(s) = self.music.get(&id).cloned() {
            play_sound(&s, PlaySoundParams { looped: self.bgm_looping, volume: self.music_volume * 0.35 * self.bgm_gain });
            self.playing = Some(Music { intro: None, body: s, switch_at: None });
            self.duck_now = 1.0;
        }
    }
    pub async fn play_music_file(&mut self, bytes: &[u8], lp: MusicLoop) -> bool {
        if !self.enabled { return false; }
        self.stop_music();
        let v = self.music_volume * 0.35 * self.bgm_gain;
        let pcm = decode(bytes);
        if pcm.is_none() { eprintln!("audio: unsupported or corrupt music file; falling back to the built-in track"); }
        let looped = pcm.as_ref().filter(|_| self.bgm_looping && (lp.start > 0 || lp.end > lp.start));
        let Some(p) = looped else {
            let Ok(s) = load_sound_from_bytes(bytes).await else { return false };
            play_sound(&s, PlaySoundParams { looped: self.bgm_looping, volume: v });
            self.playing = Some(Music { intro: None, body: s, switch_at: None });
            return true;
        };
        let end = if lp.end > lp.start { lp.end.min(p.frames.len()) } else { p.frames.len() };
        let intro_pcm = p.slice(0, end);
        let (Ok(intro), Ok(body)) = (load_sound_from_bytes(&intro_pcm.to_wav()).await, load_sound_from_bytes(&p.slice(lp.start, end).to_wav()).await) else { return false };
        play_sound(&intro, PlaySoundParams { looped: false, volume: v });
        let at = crate::gfx::get_time() + intro_pcm.seconds();
        self.playing = Some(Music { intro: Some(intro), body, switch_at: Some(at) });
        self.duck_now = 1.0;
        true
    }
    pub fn reset_bgm(&mut self) {
        self.pending_bgm.clear();
        self.bgm_gain = 1.0;
        self.bgm_looping = true;
    }
    pub fn register_music_file(&mut self, name: &str, bytes: Vec<u8>, lp: MusicLoop) {
        self.music_files.insert(name.to_ascii_lowercase(), (bytes, lp));
    }
    pub fn queue_bgm(&mut self, action: BgmAction) {
        match action {
            BgmAction::Volume(v) => self.bgm_gain = v,
            BgmAction::Loop(on) => self.bgm_looping = on,
            BgmAction::Stop => { self.pending_bgm.clear(); self.stop_music(); }
            BgmAction::LoopPoints { .. } | BgmAction::Rate(_) => {}
            play => self.pending_bgm.push(play),
        }
    }
    async fn run_pending_bgm(&mut self) {
        for action in std::mem::take(&mut self.pending_bgm) {
            let BgmAction::Play { file, looping, volume, loop_start, loop_end, .. } = action else { continue };
            self.bgm_gain = volume;
            self.bgm_looping = looping;
            let key = file.trim().to_ascii_lowercase();
            if let Some(id) = key.strip_prefix("builtin:").and_then(|n| n.trim().parse::<u32>().ok()) {
                self.play_music(id).await;
                continue;
            }
            let Some((bytes, lp)) = self.music_files.get(&key).cloned() else { continue };
            let lp = if loop_start > 0 || loop_end > 0 { MusicLoop { start: loop_start, end: loop_end } } else { lp };
            self.play_music_file(&bytes, lp).await;
        }
    }
    pub fn stop_music(&mut self) {
        self.stop_all();
        if let Some(m) = self.playing.take() {
            if let Some(i) = m.intro { stop_sound(&i); }
            stop_sound(&m.body);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loop_window_and_count_are_derived_from_milliseconds() {
        let mut o = SoundOpts { looping: true, loop_start: 100, loop_end: 500, loop_count: 3, ..Default::default() };
        let (s, e) = loop_frames(&o, 1000, 10_000);
        assert_eq!((s, e), (100, 500));
        assert!(!loop_expired(&o, 999, s, e), "still inside the loop");
        assert!(loop_expired(&o, 500 + 3 * 400 - 1, s, e), "three passes ran out");
        o.loop_count = 0;
        assert!(!loop_expired(&o, 1_000_000, s, e), "zero means endless");
        o.loop_start = 0;
        o.loop_end = 0;
        assert_eq!(loop_frames(&o, 1000, 10_000), (0, 10_000), "without points the whole clip loops");
    }
    fn energy(v: &[f32]) -> f32 { v.iter().map(|x| x * x).sum::<f32>() / v.len().max(1) as f32 }
    #[test]
    fn every_effect_is_audible_finite_and_clipped() {
        for k in ["hit", "hitheavy", "counter", "guard", "swing", "swingh", "land", "dash", "jump", "super", "transform", "defeat", "ko", "round", "fight", "win", "perfect", "meter", "select", "back", "confirm", "combo1", "combo6"] {
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
