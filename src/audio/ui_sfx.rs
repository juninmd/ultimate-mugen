//! Extra one-shot UI cues, synthesized at startup next to the built-in SFX set.
use super::SR;
use std::f32::consts::PI;

fn sec(s: f32) -> usize {
    (s * SR as f32) as usize
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
    let mut out = vec![0.0f32; sec(0.4)];
    match kind {
        "error" => {
            tone(&mut out, 0.0, 220.0, 180.0, 0.12, 0.3, true);
            tone(&mut out, 0.13, 165.0, 120.0, 0.16, 0.3, true);
        }
        "toggle" => tone(&mut out, 0.0, 1100.0, 1100.0, 0.03, 0.2, false),
        _ => return Vec::new(),
    }
    out
}
