use crate::sound_mix::Pcm;
use std::f32::consts::TAU;

const RATE: u32 = 22050;
const BEAT: f32 = 60.0 / 72.0;
const BARS: usize = 8;
const CHORDS: [[i32; 3]; 4] = [[62, 66, 69], [57, 61, 64], [59, 62, 66], [55, 59, 62]];
const PROGRESSION: [usize; BARS] = [0, 1, 2, 3, 0, 3, 1, 0];
const MELODY: [[i32; 4]; BARS] = [
    [7, 9, 7, 4],
    [9, 7, 4, 2],
    [4, 7, 9, 7],
    [2, 4, 2, 0],
    [0, 4, 7, 9],
    [7, 9, 11, 9],
    [7, 4, 2, 4],
    [2, 0, -3, 0],
];

struct Score {
    buf: Vec<f32>,
}

impl Score {
    fn at(&mut self, start: f32, dur: f32, voice: impl Fn(f32) -> f32) {
        let len = self.buf.len();
        let first = (start * RATE as f32) as usize;
        for k in 0..(dur * RATE as f32) as usize {
            self.buf[(first + k) % len] += voice(k as f32 / RATE as f32);
        }
    }
}

fn hz(note: i32) -> f32 {
    440.0 * 2f32.powf((note - 69) as f32 / 12.0)
}

pub fn pcm() -> Pcm {
    let len = (BARS as f32 * 4.0 * BEAT * RATE as f32) as usize;
    let mut s = Score { buf: vec![0.0; len] };
    for bar in 0..BARS {
        let at = bar as f32 * 4.0 * BEAT;
        let chord = CHORDS[PROGRESSION[bar]];
        let pad_len = 4.0 * BEAT;
        s.at(at, pad_len, |t| {
            let sum: f32 = chord
                .iter()
                .map(|&n| {
                    let f = hz(n);
                    (TAU * f * t).sin() + 0.4 * (TAU * f * 1.004 * t).sin()
                })
                .sum();
            sum * 0.035 * (t / 0.5).min(1.0) * (1.0 - t / pad_len).max(0.0)
        });
        for beat in 0..4 {
            let bass = hz(chord[0] - 24);
            s.at(at + beat as f32 * BEAT, 0.9 * BEAT, |t| {
                (TAU * bass * t).sin() * 0.2 * (-t * 5.0).exp()
            });
            for half in 0..2 {
                let note = chord[(beat * 2 + half) % 3] + 12;
                let f = hz(note);
                s.at(at + (beat as f32 + half as f32 * 0.5) * BEAT, 0.6 * BEAT, |t| {
                    (TAU * f * t).sin() * 0.035 * (t / 0.01).min(1.0) * (-t * 9.0).exp()
                });
            }
            let f = hz(62 + MELODY[bar][beat]);
            s.at(at + beat as f32 * BEAT, 2.0 * BEAT, |t| {
                ((TAU * f * t).sin() + 0.3 * (TAU * 2.0 * f * t).sin())
                    * 0.16
                    * (t / 0.02).min(1.0)
                    * (-t * 2.2).exp()
            });
        }
    }
    let peak = s.buf.iter().fold(1e-4f32, |m, x| m.max(x.abs()));
    let gain = 0.8 / peak;
    Pcm {
        rate: RATE,
        frames: s.buf.iter().map(|x| [x * gain; 2]).collect(),
    }
}

pub fn wav() -> Vec<u8> {
    pcm().to_wav()
}

#[cfg(test)]
mod tests {
    #[test]
    fn epilogue_track_is_audible_bounded_and_finite() {
        let pcm = super::pcm();
        assert!(pcm.seconds() > 20.0);
        assert!(pcm
            .frames
            .iter()
            .all(|f| f[0].is_finite() && f[0].abs() <= 1.0));
        assert!(pcm.frames.iter().any(|f| f[0].abs() > 0.5));
    }
}
