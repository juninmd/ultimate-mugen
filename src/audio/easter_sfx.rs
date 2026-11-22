use super::SR;
use std::f32::consts::PI;

pub const KINDS: &[(&str, u32)] = &[
    ("easter_toasty", 1),
    ("easter_coin", 1),
    ("easter_crown", 1),
    ("easter_clash", 1),
    ("easter_tumbleweed", 1),
    ("easter_ufo", 1),
    ("easter_kitsune", 1),
    ("easter_capybara", 1),
    ("easter_bird", 1),
    ("easter_parrot", 1),
    ("easter_wonder", 1),
    ("easter_banter", 1),
    ("easter_bankai", 1),
    ("easter_shunpo", 1),
];

fn sec(s: f32) -> usize {
    (s * SR as f32) as usize
}

fn echo(out: &mut [f32], delay: f32, feedback: f32) {
    let d = sec(delay);
    for i in d..out.len() {
        out[i] += out[i - d] * feedback;
    }
}

fn sweep(out: &mut [f32], at: f32, f0: f32, f1: f32, dur: f32, g: f32, wave: u8) {
    let (o, n) = (sec(at), sec(dur));
    let mut ph = 0.0f32;
    for i in 0..n {
        if o + i >= out.len() {
            break;
        }
        let t = i as f32 / n as f32;
        ph = (ph + f0 * (f1 / f0).powf(t) / SR as f32).fract();
        let w = match wave {
            0 => (ph * 2.0 * PI).sin(),
            1 => 4.0 * (ph - 0.5).abs() - 1.0,
            2 => ph * 2.0 - 1.0,
            _ => if ph < 0.5 { 1.0 } else { -1.0 },
        };
        let env = (t / 0.03).min(1.0) * (1.0 - t).powi(2);
        out[o + i] += w * g * env;
    }
}

fn vibrato(out: &mut [f32], at: f32, base: f32, depth: f32, rate: f32, dur: f32, g: f32) {
    let (o, n) = (sec(at), sec(dur));
    let mut ph = 0.0f32;
    for i in 0..n {
        if o + i >= out.len() {
            break;
        }
        let t = i as f32 / n as f32;
        let secs = i as f32 / SR as f32;
        let freq = base + (secs * rate * 2.0 * PI).sin() * depth;
        ph = (ph + freq / SR as f32).fract();
        let env = (t / 0.08).min(1.0) * (1.0 - t).powf(1.5);
        out[o + i] += (ph * 2.0 * PI).sin() * g * env;
    }
}

fn noise_puff(out: &mut [f32], at: f32, dur: f32, g: f32, seed: u32) {
    let (o, n) = (sec(at), sec(dur));
    let mut s = seed.max(1);
    let mut lp = 0.0f32;
    for i in 0..n {
        if o + i >= out.len() {
            break;
        }
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let raw = s as f32 / 2147483648.0 - 1.0;
        lp += (raw - lp) * 0.14;
        let t = i as f32 / n as f32;
        out[o + i] += lp * g * (t * PI).sin();
    }
}

pub fn render(kind: &str) -> Vec<f32> {
    let total = match kind {
        "easter_tumbleweed" | "easter_ufo" | "easter_wonder" | "easter_crown" => 0.95,
        "easter_toasty" | "easter_kitsune" | "easter_clash" => 0.72,
        _ => 0.58,
    };
    let mut out = vec![0.0f32; sec(total)];
    match kind {
        "easter_toasty" => {
            sweep(&mut out, 0.0, 440.0, 880.0, 0.11, 0.28, 3);
            sweep(&mut out, 0.08, 660.0, 1320.0, 0.16, 0.24, 1);
            sweep(&mut out, 0.16, 1175.0, 1568.0, 0.24, 0.22, 0);
            echo(&mut out, 0.09, 0.32);
        }
        "easter_coin" => {
            sweep(&mut out, 0.0, 987.77, 987.77, 0.09, 0.28, 3);
            sweep(&mut out, 0.08, 1318.51, 1318.51, 0.34, 0.28, 1);
            sweep(&mut out, 0.08, 2637.02, 2637.02, 0.24, 0.12, 0);
        }
        "easter_crown" => {
            for (i, f) in [523.25f32, 659.25, 783.99, 987.77, 1046.5].iter().enumerate() {
                sweep(&mut out, i as f32 * 0.065, *f, *f, 0.26, 0.2, 1);
                sweep(&mut out, i as f32 * 0.065, *f * 2.0, *f * 2.0, 0.18, 0.08, 0);
            }
            echo(&mut out, 0.11, 0.35);
        }
        "easter_clash" => {
            sweep(&mut out, 0.0, 1680.0, 520.0, 0.28, 0.26, 2);
            sweep(&mut out, 0.0, 2340.0, 980.0, 0.24, 0.2, 3);
            noise_puff(&mut out, 0.0, 0.16, 0.28, 0xCAFE_1234);
            echo(&mut out, 0.07, 0.38);
        }
        "easter_tumbleweed" => {
            vibrato(&mut out, 0.04, 880.0, 26.0, 7.5, 0.28, 0.22);
            vibrato(&mut out, 0.34, 659.25, 22.0, 6.5, 0.22, 0.2);
            vibrato(&mut out, 0.56, 880.0, 28.0, 8.0, 0.32, 0.22);
            noise_puff(&mut out, 0.0, 0.85, 0.18, 0xD353_7777);
        }
        "easter_ufo" => {
            vibrato(&mut out, 0.0, 740.0, 220.0, 11.0, 0.82, 0.25);
            sweep(&mut out, 0.05, 420.0, 1280.0, 0.42, 0.14, 1);
            sweep(&mut out, 0.45, 1280.0, 380.0, 0.42, 0.14, 1);
        }
        "easter_kitsune" => {
            sweep(&mut out, 0.0, 1480.0, 1480.0, 0.04, 0.26, 3);
            for (f, g) in [(740.0, 0.22), (1110.0, 0.16), (1760.0, 0.12), (2420.0, 0.08)] {
                sweep(&mut out, 0.06, f, f * 0.995, 0.56, g, 0);
            }
            echo(&mut out, 0.12, 0.3);
        }
        "easter_capybara" => {
            for (i, f) in [523.25f32, 659.25, 783.99, 880.0].iter().enumerate() {
                sweep(&mut out, i as f32 * 0.08, *f, *f * 1.02, 0.14, 0.26, 0);
                sweep(&mut out, i as f32 * 0.08, *f * 2.0, *f * 2.0, 0.06, 0.1, 1);
            }
        }
        "easter_bird" => {
            sweep(&mut out, 0.0, 1400.0, 2600.0, 0.09, 0.22, 0);
            sweep(&mut out, 0.1, 1800.0, 2900.0, 0.09, 0.22, 0);
            sweep(&mut out, 0.22, 2200.0, 1600.0, 0.14, 0.2, 0);
            echo(&mut out, 0.08, 0.28);
        }
        "easter_parrot" => {
            sweep(&mut out, 0.0, 920.0, 1580.0, 0.12, 0.24, 2);
            sweep(&mut out, 0.16, 1340.0, 760.0, 0.16, 0.24, 2);
            sweep(&mut out, 0.18, 1680.0, 1120.0, 0.12, 0.14, 3);
        }
        "easter_wonder" => {
            sweep(&mut out, 0.0, 146.83, 140.0, 0.85, 0.24, 0);
            for (i, f) in [587.33f32, 739.99, 880.0, 1174.66].iter().enumerate() {
                sweep(&mut out, 0.04 + i as f32 * 0.08, *f, *f, 0.35, 0.16, 0);
            }
            echo(&mut out, 0.13, 0.36);
        }
        "easter_banter" => {
            sweep(&mut out, 0.0, 480.0, 960.0, 0.07, 0.24, 1);
            sweep(&mut out, 0.07, 880.0, 1174.66, 0.16, 0.22, 0);
        }
        "easter_bankai" => {
            sweep(&mut out, 0.0, 160.0, 52.0, 0.62, 0.34, 2);
            sweep(&mut out, 0.04, 880.0, 1760.0, 0.32, 0.16, 0);
            noise_puff(&mut out, 0.0, 0.36, 0.24, 0xB44C_4100);
            echo(&mut out, 0.11, 0.38);
        }
        "easter_shunpo" => {
            sweep(&mut out, 0.0, 1900.0, 420.0, 0.11, 0.24, 1);
            sweep(&mut out, 0.05, 620.0, 2600.0, 0.14, 0.22, 0);
            noise_puff(&mut out, 0.0, 0.14, 0.28, 0x5404_9000);
        }
        _ => return Vec::new(),
    }
    out
}
