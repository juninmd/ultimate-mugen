//! Extra one-shot UI cues, synthesized at startup next to the built-in SFX set.
use super::SR;
use std::f32::consts::PI;

fn sec(s: f32) -> usize {
    (s * SR as f32) as usize
}

fn echo(out: &mut [f32], delay: f32, feedback: f32) {
    let d = sec(delay);
    for i in d..out.len() {
        out[i] += out[i - d] * feedback;
    }
}

fn tone(out: &mut [f32], at: f32, f0: f32, f1: f32, dur: f32, g: f32, triangle: bool) {
    let (o, n) = (sec(at), sec(dur));
    let mut ph = 0.0f32;
    for i in 0..n {
        if o + i >= out.len() {
            break;
        }
        let t = i as f32 / n as f32;
        ph = (ph + f0 * (f1 / f0).powf(t) / SR as f32).fract();
        let w = if triangle {
            4.0 * (ph - 0.5).abs() - 1.0
        } else {
            (ph * 2.0 * PI).sin()
        };
        out[o + i] += w * g * (t / 0.04).min(1.0) * (1.0 - t).powi(2);
    }
}

pub fn render(kind: &str) -> Vec<f32> {
    let total = if kind == "unlock" { 0.9 } else { 0.4 };
    let mut out = vec![0.0f32; sec(total)];
    match kind {
        "error" => {
            tone(&mut out, 0.0, 220.0, 180.0, 0.12, 0.3, true);
            tone(&mut out, 0.13, 165.0, 120.0, 0.16, 0.3, true);
        }
        "toggle" => tone(&mut out, 0.0, 1100.0, 1100.0, 0.03, 0.2, false),
        "tick" => tone(&mut out, 0.0, 1900.0, 1900.0, 0.02, 0.25, false),
        "load" => {
            tone(&mut out, 0.0, 660.0, 880.0, 0.08, 0.2, false);
            tone(&mut out, 0.08, 1320.0, 1320.0, 0.14, 0.2, false);
        }
        "pause" => tone(&mut out, 0.0, 600.0, 900.0, 0.1, 0.2, false),
        "results" => {
            for (i, f) in [523.25f32, 659.25, 783.99].iter().enumerate() {
                tone(&mut out, i as f32 * 0.09, *f, *f, 0.14, 0.2, true);
            }
        }
        "purchase" => {
            tone(&mut out, 0.0, 988.0, 988.0, 0.06, 0.2, false);
            tone(&mut out, 0.06, 1319.0, 1319.0, 0.14, 0.2, false);
        }
        "unlock" => {
            for (i, f) in [523.25f32, 659.25, 783.99, 1046.5].iter().enumerate() {
                tone(&mut out, i as f32 * 0.07, *f, *f, 0.16, 0.2, false);
            }
            echo(&mut out, 0.09, 0.35);
        }
        _ => return Vec::new(),
    }
    out
}
