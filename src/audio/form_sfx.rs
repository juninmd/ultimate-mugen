use super::SR;
use std::f32::consts::PI;

pub const KINDS: &[(&str, u32)] = &[("transform2", 1), ("transform3", 1), ("formready", 1)];

fn sec(s: f32) -> usize {
    (s * SR as f32) as usize
}

fn tone(out: &mut [f32], at: f32, f0: f32, f1: f32, dur: f32, gain: f32, harmonics: u32) {
    let (start, count) = (sec(at), sec(dur));
    let mut phase = 0.0f32;
    for i in 0..count {
        if start + i >= out.len() {
            break;
        }
        let t = i as f32 / count as f32;
        phase = (phase + f0 * (f1 / f0).powf(t) / SR as f32).fract();
        let mut wave = (phase * 2.0 * PI).sin();
        for h in 2..=harmonics {
            wave += (phase * 2.0 * PI * h as f32).sin() / (h as f32 * 1.6);
        }
        out[start + i] += wave * gain * (t / 0.03).min(1.0) * (1.0 - t).powi(2);
    }
}

fn noise(out: &mut [f32], at: f32, dur: f32, gain: f32, lo: f32, hi: f32, seed: u32) {
    let (start, count) = (sec(at), sec(dur));
    let mut state = seed.wrapping_mul(2654435761).wrapping_add(1);
    let mut low = 0.0f32;
    for i in 0..count {
        if start + i >= out.len() {
            break;
        }
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let white = (state >> 8) as f32 / 8388608.0 - 1.0;
        let t = i as f32 / count as f32;
        let cutoff = (lo * (hi / lo).powf(t) / SR as f32).min(0.9);
        low += (white - low) * cutoff * 2.0;
        out[start + i] += low * gain * (t / 0.05).min(1.0) * (1.0 - t).powf(1.4);
    }
}

fn shimmer(out: &mut [f32], at: f32, base: f32, dur: f32, gain: f32) {
    for (i, ratio) in [1.0f32, 2.76, 5.4, 8.93].iter().enumerate() {
        tone(
            out,
            at,
            base * ratio,
            base * ratio * 0.995,
            dur,
            gain / (i as f32 + 1.0),
            1,
        );
    }
}

fn echo(out: &mut [f32], delay: f32, feedback: f32) {
    let d = sec(delay);
    for i in d..out.len() {
        out[i] += out[i - d] * feedback;
    }
}

fn charge(out: &mut [f32], chord: &[f32], at: f32, dur: f32, gain: f32) {
    for (i, ratio) in chord.iter().enumerate() {
        tone(
            out,
            at,
            70.0 * ratio,
            520.0 * ratio,
            dur,
            gain - i as f32 * 0.02,
            3,
        );
    }
    noise(
        out,
        at,
        dur,
        0.5,
        220.0,
        5200.0,
        7 + i32::from(chord.len() > 3) as u32,
    );
}

fn boom(out: &mut [f32], at: f32, depth: f32, gain: f32, tail: f32) {
    tone(out, at, depth * 2.4, depth * 0.3, tail, gain, 2);
    tone(
        out,
        at,
        depth * 4.0,
        depth * 0.8,
        tail * 0.5,
        gain * 0.55,
        1,
    );
    noise(out, at, tail * 0.8, gain * 0.9, 2600.0, 160.0, 31);
}

fn second_form() -> Vec<f32> {
    let mut out = vec![0.0f32; sec(2.2)];
    charge(&mut out, &[1.0, 1.5, 2.0, 3.0], 0.0, 1.1, 0.16);
    boom(&mut out, 1.1, 60.0, 1.1, 1.0);
    shimmer(&mut out, 1.12, 420.0, 0.9, 0.07);
    tone(&mut out, 1.3, 330.0, 660.0, 0.8, 0.08, 1);
    echo(&mut out, 0.17, 0.32);
    out
}

fn third_form() -> Vec<f32> {
    let mut out = vec![0.0f32; sec(3.4)];
    charge(&mut out, &[0.5, 1.0, 1.5, 2.0, 3.0, 4.0], 0.0, 1.4, 0.15);
    boom(&mut out, 1.4, 50.0, 1.15, 0.9);
    noise(&mut out, 1.4, 0.35, 1.0, 5200.0, 400.0, 53);
    charge(&mut out, &[1.0, 1.25, 1.5, 2.0], 1.5, 0.7, 0.12);
    boom(&mut out, 2.2, 38.0, 1.3, 1.2);
    shimmer(&mut out, 2.2, 360.0, 1.1, 0.09);
    shimmer(&mut out, 2.35, 540.0, 0.9, 0.06);
    tone(&mut out, 2.2, 110.0, 880.0, 0.9, 0.12, 4);
    echo(&mut out, 0.23, 0.38);
    echo(&mut out, 0.41, 0.2);
    out
}

fn form_ready() -> Vec<f32> {
    let mut out = vec![0.0f32; sec(0.7)];
    for (i, freq) in [659.25f32, 880.0, 1318.5].iter().enumerate() {
        tone(&mut out, i as f32 * 0.07, *freq, *freq * 1.01, 0.3, 0.2, 1);
    }
    shimmer(&mut out, 0.2, 1320.0, 0.4, 0.05);
    echo(&mut out, 0.12, 0.3);
    out
}

pub fn render(kind: &str) -> Vec<f32> {
    match kind {
        "transform3" => third_form(),
        "formready" => form_ready(),
        _ => second_form(),
    }
}
