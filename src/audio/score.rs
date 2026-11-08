use super::{Noise, SR};
use std::f32::consts::PI;

#[derive(Clone, Copy)]
enum Drums {
    Rock,
    Taiko,
    Samba,
    Breaks,
    March,
}

#[derive(Clone, Copy)]
enum Lead {
    Square,
    Saw,
    Pluck,
    Flute,
}

struct Theme {
    bpm: f32,
    scale: &'static [i32],
    root: i32,
    prog: [i32; 8],
    drums: Drums,
    lead: Lead,
    pad: bool,
}

const MINOR_PENT: &[i32] = &[0, 3, 5, 7, 10, 12];
const PHRYGIAN: &[i32] = &[0, 1, 3, 5, 7, 8, 12];
const BLUES: &[i32] = &[0, 3, 5, 6, 7, 10, 12];
const YO: &[i32] = &[0, 2, 5, 7, 9, 12];
const HARMONIC: &[i32] = &[0, 2, 3, 5, 7, 8, 11, 12];
const DORIAN: &[i32] = &[0, 2, 3, 5, 7, 9, 10, 12];
const MIXO: &[i32] = &[0, 2, 4, 5, 7, 9, 10, 12];
const DOUBLE_HARM: &[i32] = &[0, 1, 4, 5, 7, 8, 11, 12];

const THEMES: [Theme; 21] = [
    Theme {
        bpm: 128.0,
        scale: YO,
        root: 40,
        prog: [0, 0, 5, 5, 3, 3, 7, 5],
        drums: Drums::Taiko,
        lead: Lead::Flute,
        pad: true,
    },
    Theme {
        bpm: 150.0,
        scale: MINOR_PENT,
        root: 43,
        prog: [0, 0, 3, 3, 5, 5, 7, 3],
        drums: Drums::Breaks,
        lead: Lead::Saw,
        pad: true,
    },
    Theme {
        bpm: 118.0,
        scale: YO,
        root: 38,
        prog: [0, 5, 0, 5, 3, 7, 3, 5],
        drums: Drums::Taiko,
        lead: Lead::Pluck,
        pad: true,
    },
    Theme {
        bpm: 136.0,
        scale: DORIAN,
        root: 41,
        prog: [0, 0, 5, 3, 0, 0, 7, 5],
        drums: Drums::Rock,
        lead: Lead::Square,
        pad: false,
    },
    Theme {
        bpm: 124.0,
        scale: PHRYGIAN,
        root: 39,
        prog: [0, 1, 0, 1, 5, 3, 1, 0],
        drums: Drums::March,
        lead: Lead::Flute,
        pad: true,
    },
    Theme {
        bpm: 146.0,
        scale: BLUES,
        root: 42,
        prog: [0, 0, 0, 0, 5, 5, 0, 7],
        drums: Drums::Rock,
        lead: Lead::Saw,
        pad: false,
    },
    Theme {
        bpm: 132.0,
        scale: MIXO,
        root: 45,
        prog: [0, 0, 5, 5, 7, 7, 5, 3],
        drums: Drums::Samba,
        lead: Lead::Pluck,
        pad: false,
    },
    Theme {
        bpm: 126.0,
        scale: DOUBLE_HARM,
        root: 38,
        prog: [0, 0, 1, 1, 5, 5, 1, 0],
        drums: Drums::Taiko,
        lead: Lead::Flute,
        pad: true,
    },
    Theme {
        bpm: 140.0,
        scale: PHRYGIAN,
        root: 40,
        prog: [0, 0, 3, 5, 0, 0, 1, 3],
        drums: Drums::March,
        lead: Lead::Square,
        pad: true,
    },
    Theme {
        bpm: 138.0,
        scale: BLUES,
        root: 41,
        prog: [0, 3, 0, 5, 0, 3, 7, 5],
        drums: Drums::Rock,
        lead: Lead::Pluck,
        pad: false,
    },
    Theme {
        bpm: 122.0,
        scale: HARMONIC,
        root: 43,
        prog: [0, 0, 5, 5, 3, 3, 7, 7],
        drums: Drums::March,
        lead: Lead::Saw,
        pad: true,
    },
    Theme {
        bpm: 130.0,
        scale: DORIAN,
        root: 44,
        prog: [0, 5, 3, 5, 0, 5, 7, 3],
        drums: Drums::Samba,
        lead: Lead::Flute,
        pad: true,
    },
    Theme {
        bpm: 144.0,
        scale: MINOR_PENT,
        root: 40,
        prog: [0, 0, 7, 7, 5, 5, 3, 7],
        drums: Drums::Breaks,
        lead: Lead::Square,
        pad: false,
    },
    Theme {
        bpm: 116.0,
        scale: YO,
        root: 45,
        prog: [0, 0, 3, 3, 5, 5, 3, 0],
        drums: Drums::Taiko,
        lead: Lead::Flute,
        pad: true,
    },
    Theme {
        bpm: 134.0,
        scale: HARMONIC,
        root: 42,
        prog: [0, 0, 5, 5, 0, 0, 7, 5],
        drums: Drums::March,
        lead: Lead::Pluck,
        pad: true,
    },
    Theme {
        bpm: 142.0,
        scale: MIXO,
        root: 38,
        prog: [0, 0, 5, 7, 0, 0, 5, 3],
        drums: Drums::Rock,
        lead: Lead::Saw,
        pad: true,
    },
    Theme {
        bpm: 120.0,
        scale: DOUBLE_HARM,
        root: 41,
        prog: [0, 1, 0, 1, 3, 3, 1, 5],
        drums: Drums::March,
        lead: Lead::Flute,
        pad: true,
    },
    Theme {
        bpm: 148.0,
        scale: PHRYGIAN,
        root: 43,
        prog: [0, 0, 1, 1, 0, 0, 5, 3],
        drums: Drums::Breaks,
        lead: Lead::Saw,
        pad: false,
    },
    Theme {
        bpm: 127.0,
        scale: DORIAN,
        root: 39,
        prog: [0, 3, 5, 3, 0, 3, 7, 5],
        drums: Drums::Samba,
        lead: Lead::Pluck,
        pad: true,
    },
    Theme {
        bpm: 133.0,
        scale: MINOR_PENT,
        root: 44,
        prog: [0, 0, 5, 5, 7, 7, 3, 5],
        drums: Drums::Rock,
        lead: Lead::Square,
        pad: true,
    },
    Theme {
        bpm: 125.0,
        scale: BLUES,
        root: 37,
        prog: [0, 0, 5, 3, 0, 0, 7, 5],
        drums: Drums::Taiko,
        lead: Lead::Saw,
        pad: true,
    },
];

pub fn theme_count() -> u32 {
    THEMES.len() as u32
}

fn midi(n: i32) -> f32 {
    440.0 * 2f32.powf((n - 69) as f32 / 12.0)
}

pub fn music(seed: u32) -> Vec<f32> {
    let th = &THEMES[seed as usize % THEMES.len()];
    let step = 60.0 / th.bpm / 4.0;
    let bars = 8usize;
    let steps = 16 * bars;
    let total = (steps as f32 * step * SR as f32) as usize;
    let mut out = vec![0.0f32; total];
    let mut nz = Noise(seed.wrapping_mul(7919) | 1);
    let sr = SR as f32;
    for i in 0..steps {
        let bar = (i / 16) % bars;
        let s = i % 16;
        let t0 = (i as f32 * step * sr) as usize;
        let r = th.root + th.prog[bar];
        let accent = if bar % 4 == 3 && s >= 12 { 1.25 } else { 1.0 };
        let mut add = |f: &dyn Fn(f32) -> f32, dur: f32| {
            let n = (dur * sr) as usize;
            for k in 0..n {
                let idx = (t0 + k) % total;
                out[idx] += f(k as f32 / sr);
            }
        };
        let kick = |add: &mut dyn FnMut(&dyn Fn(f32) -> f32, f32), g: f32| {
            add(
                &|t| {
                    (2.0 * PI * (160.0 * (0.22f32).powf(t / 0.16)) * t).sin()
                        * (1.0 - t / 0.2).max(0.0)
                        * g
                },
                0.2,
            );
        };
        let mut burst =
            |add: &mut dyn FnMut(&dyn Fn(f32) -> f32, f32), dur: f32, g: f32, hp: bool| {
                let ns: Vec<f32> = (0..(dur * sr) as usize).map(|_| nz.n()).collect();
                add(
                    &|t| {
                        let i = ((t * sr) as usize).min(ns.len() - 1);
                        let v = if hp && i > 0 {
                            ns[i] - ns[i - 1]
                        } else {
                            ns[i]
                        };
                        v * (1.0 - t / dur).max(0.0) * g
                    },
                    dur,
                );
            };
        let mut tom = |add: &mut dyn FnMut(&dyn Fn(f32) -> f32, f32), f0: f32, g: f32| {
            add(
                &|t| {
                    (2.0 * PI * (f0 * (0.5f32).powf(t / 0.2)) * t).sin()
                        * (1.0 - t / 0.34).max(0.0)
                        * g
                },
                0.34,
            );
        };
        match th.drums {
            Drums::Rock => {
                if s % 4 == 0 {
                    kick(&mut add, 0.9 * accent);
                }
                if s % 8 == 4 {
                    burst(&mut add, 0.16, 0.5, false);
                }
                if s % 2 == 0 {
                    burst(&mut add, 0.04, 0.12, true);
                }
            }
            Drums::Taiko => {
                if matches!(s, 0 | 6 | 10) {
                    tom(&mut add, 95.0, 1.0 * accent);
                }
                if matches!(s, 4 | 12) {
                    tom(&mut add, 140.0, 0.7);
                }
                if s % 4 == 2 {
                    burst(&mut add, 0.03, 0.14, true);
                }
                if s == 8 {
                    burst(&mut add, 0.12, 0.35, false);
                }
            }
            Drums::Samba => {
                if matches!(s, 0 | 8) {
                    kick(&mut add, 0.8);
                }
                if matches!(s, 4 | 12) {
                    tom(&mut add, 70.0, 0.9);
                }
                burst(&mut add, 0.035, if s % 4 == 2 { 0.2 } else { 0.1 }, true);
                if matches!(s, 3 | 7 | 10 | 14) {
                    burst(&mut add, 0.07, 0.3, false);
                }
            }
            Drums::Breaks => {
                if matches!(s, 0 | 7 | 10) {
                    kick(&mut add, 0.95 * accent);
                }
                if matches!(s, 4 | 12) || (bar % 4 == 3 && matches!(s, 14 | 15)) {
                    burst(&mut add, 0.15, 0.55, false);
                }
                if s % 2 == 0 {
                    burst(&mut add, 0.035, 0.13, true);
                }
            }
            Drums::March => {
                if matches!(s, 0 | 8) {
                    kick(&mut add, 0.85);
                }
                if s % 4 == 0 || (bar % 2 == 1 && s % 2 == 0) {
                    burst(&mut add, 0.1, if s % 8 == 4 { 0.5 } else { 0.28 }, false);
                }
                if s % 4 == 2 {
                    burst(&mut add, 0.03, 0.12, true);
                }
            }
        }
        if s % 2 == 0 {
            let f = midi(r - 12 + if s % 8 == 6 { 12 } else { 0 });
            add(
                &|t| {
                    let ph = (t * f).fract();
                    (ph * 2.0 - 1.0) * 0.22 * (1.0 - t / (step * 1.8)).max(0.0)
                },
                step * 1.8,
            );
        }
        if s == 0 && th.pad {
            let notes = [r, r + th.scale[2.min(th.scale.len() - 1)], r + 7];
            let dur = step * 15.0;
            add(
                &|t| {
                    let env = (t / 0.25).min(1.0) * (1.0 - t / dur).max(0.0);
                    notes
                        .iter()
                        .map(|&n| {
                            let f = midi(n);
                            (2.0 * PI * f * t).sin() + 0.5 * (2.0 * PI * f * 1.003 * t).sin()
                        })
                        .sum::<f32>()
                        * 0.035
                        * env
                },
                dur,
            );
        }
        let lead_on = match th.lead {
            Lead::Flute => s % 4 == 0 || s % 8 == 6,
            _ => s % 2 == 1 || s % 3 == 0,
        };
        if lead_on {
            let idx = (s * 3 + bar * 2 + seed as usize) % th.scale.len();
            let f = midi(r + 12 + th.scale[idx]);
            match th.lead {
                Lead::Square => add(
                    &|t| {
                        let sq = if (t * f).fract() < 0.5 { 1.0 } else { -1.0 };
                        sq * 0.06 * (1.0 - t / (step * 0.9)).max(0.0)
                    },
                    step * 0.9,
                ),
                Lead::Saw => add(
                    &|t| ((t * f).fract() * 2.0 - 1.0) * 0.07 * (1.0 - t / (step * 1.2)).max(0.0),
                    step * 1.2,
                ),
                Lead::Pluck => add(
                    &|t| {
                        (2.0 * PI * f * t).sin() * 0.11 * (-t * 14.0).exp()
                            + (2.0 * PI * f * 2.0 * t).sin() * 0.04 * (-t * 22.0).exp()
                    },
                    step * 2.5,
                ),
                Lead::Flute => add(
                    &|t| {
                        let vib = 1.0 + 0.006 * (2.0 * PI * 5.5 * t).sin();
                        let env = (t / 0.03).min(1.0) * (1.0 - t / (step * 3.0)).max(0.0);
                        (2.0 * PI * f * vib * t).sin() * 0.1 * env
                    },
                    step * 3.0,
                ),
            }
        }
    }
    let mut y = 0.0;
    for s in out.iter_mut() {
        y += (*s - y) * 0.45;
        *s = y * 0.7;
    }
    let peak = out.iter().fold(0.0f32, |m, x| m.max(x.abs())).max(1e-4);
    let g = 0.85 / peak;
    for s in out.iter_mut() {
        *s = (*s * g * 1.15).tanh();
    }
    out
}
