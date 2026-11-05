use std::io::Cursor;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pcm {
    pub rate: u32,
    pub frames: Vec<[f32; 2]>,
}

impl Pcm {
    pub fn seconds(&self) -> f64 {
        self.frames.len() as f64 / self.rate.max(1) as f64
    }

    pub fn slice(&self, start: usize, end: usize) -> Pcm {
        let end = end.min(self.frames.len());
        Pcm {
            rate: self.rate,
            frames: self.frames.get(start.min(end)..end).unwrap_or(&[]).to_vec(),
        }
    }

    pub fn resume_at(&self, frame: usize, looping: bool) -> Option<Pcm> {
        let n = self.frames.len();
        if n == 0 {
            return None;
        }
        if looping {
            let at = frame % n;
            let mut frames = self.frames[at..].to_vec();
            frames.extend_from_slice(&self.frames[..at]);
            return Some(Pcm {
                rate: self.rate,
                frames,
            });
        }
        (frame < n).then(|| self.slice(frame, n))
    }

    /// Restart a looping clip between `start` and `end` frames, keeping the intro up to `end` on the first pass.
    pub fn resume_segment(
        &self,
        frame: usize,
        looping: bool,
        start: usize,
        end: usize,
    ) -> Option<Pcm> {
        let n = self.frames.len();
        if n == 0 {
            return None;
        }
        let start = start.min(n);
        let end = end.min(n).max(start);
        if !looping || end <= start {
            return self.resume_at(frame, looping);
        }
        let span = end - start;
        let at = start + frame % span;
        let mut frames = self.frames[at..end].to_vec();
        frames.extend_from_slice(&self.frames[start..at]);
        Some(Pcm {
            rate: self.rate,
            frames,
        })
    }

    pub fn render(&self, gain: f32, pan: f32, rate: f32) -> Pcm {
        let (l, r) = pan_gains(pan);
        let step = rate.clamp(0.05, 8.0) as f64;
        let n = (self.frames.len() as f64 / step).floor() as usize;
        let mut frames = Vec::with_capacity(n);
        for i in 0..n {
            let pos = i as f64 * step;
            let a = pos as usize;
            let t = (pos - a as f64) as f32;
            let s0 = self.frames[a];
            let s1 = self.frames.get(a + 1).copied().unwrap_or(s0);
            let mix = |c: usize| s0[c] + (s1[c] - s0[c]) * t;
            frames.push([mix(0) * gain * l, mix(1) * gain * r]);
        }
        Pcm {
            rate: self.rate,
            frames,
        }
    }

    pub fn to_wav(&self) -> Vec<u8> {
        let n = self.frames.len() as u32;
        let mut v = Vec::with_capacity(44 + n as usize * 4);
        v.extend_from_slice(b"RIFF");
        v.extend_from_slice(&(36 + n * 4).to_le_bytes());
        v.extend_from_slice(b"WAVEfmt ");
        v.extend_from_slice(&16u32.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&2u16.to_le_bytes());
        v.extend_from_slice(&self.rate.to_le_bytes());
        v.extend_from_slice(&(self.rate * 4).to_le_bytes());
        v.extend_from_slice(&4u16.to_le_bytes());
        v.extend_from_slice(&16u16.to_le_bytes());
        v.extend_from_slice(b"data");
        v.extend_from_slice(&(n * 4).to_le_bytes());
        for f in &self.frames {
            for s in f {
                v.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
            }
        }
        v
    }
}

pub fn pan_gains(pan: f32) -> (f32, f32) {
    let a = (pan.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
    (
        a.cos() * std::f32::consts::SQRT_2,
        a.sin() * std::f32::consts::SQRT_2,
    )
}

pub fn pan_bucket(pan: f32) -> i8 {
    (pan.clamp(-1.0, 1.0) * 5.0).round() as i8
}

pub fn decode(bytes: &[u8]) -> Option<Pcm> {
    let mss = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;
    let mut format = probed.format;
    let track = format.default_track()?;
    let id = track.id;
    let mut dec = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .ok()?;
    let mut out = Pcm {
        rate: track.codec_params.sample_rate.unwrap_or(22050),
        frames: vec![],
    };
    let mut samples: Option<SampleBuffer<f32>> = None;
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != id {
            continue;
        }
        match dec.decode(&packet) {
            Ok(buf) => {
                let spec = *buf.spec();
                out.rate = spec.rate;
                let ch = spec.channels.count().max(1);
                if samples
                    .as_ref()
                    .is_none_or(|sb| sb.capacity() < buf.capacity() * ch)
                {
                    samples = Some(SampleBuffer::new(buf.capacity() as u64, spec));
                }
                let sb = samples.as_mut().unwrap();
                sb.copy_interleaved_ref(buf);
                let count = sb.samples().len() / ch;
                out.frames.reserve(count);
                if ch == 1 {
                    for &s in sb.samples() {
                        out.frames.push([s, s]);
                    }
                } else {
                    for f in sb.samples().chunks_exact(ch) {
                        out.frames.push([f[0], f[1]]);
                    }
                }
            }
            Err(Error::DecodeError(_)) => continue,
            Err(_) => break,
        }
    }
    (!out.frames.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(n: usize) -> Pcm {
        Pcm {
            rate: 22050,
            frames: (0..n).map(|i| [(i as f32 * 0.05).sin() * 0.5; 2]).collect(),
        }
    }

    #[test]
    fn wav_round_trip_through_decoder() {
        let p = tone(4000);
        let back = decode(&p.to_wav()).expect("decodes its own wav");
        assert_eq!(back.rate, 22050);
        assert_eq!(back.frames.len(), 4000);
        assert!((back.frames[100][0] - p.frames[100][0]).abs() < 1e-3);
    }

    #[test]
    fn pan_moves_energy_between_channels() {
        let (l, r) = pan_gains(0.0);
        assert!((l - 1.0).abs() < 1e-5 && (r - 1.0).abs() < 1e-5);
        let p = tone(1000).render(1.0, -1.0, 1.0);
        assert!(p.frames.iter().all(|f| f[1].abs() < 1e-5));
        assert!(p.frames.iter().any(|f| f[0].abs() > 0.1));
        assert_eq!(pan_bucket(-1.0), -5);
        assert_eq!(pan_bucket(0.04), 0);
    }

    #[test]
    fn rate_changes_length_and_slice_clamps() {
        let p = tone(1000);
        assert_eq!(p.render(1.0, 0.0, 2.0).frames.len(), 500);
        assert_eq!(p.render(1.0, 0.0, 0.5).frames.len(), 2000);
        assert_eq!(p.slice(900, 5000).frames.len(), 100);
        assert!(p.slice(2000, 3000).frames.is_empty());
    }

    #[test]
    fn resume_segment_loops_between_the_loop_points() {
        let p = tone(100);
        let looped = p.resume_segment(10, true, 40, 80).unwrap();
        assert_eq!(
            looped.frames.len(),
            40,
            "the loop wraps inside the 40..80 window"
        );
        assert_eq!(looped.frames[0], p.frames[50]);
        assert_eq!(looped.frames[29], p.frames[79]);
        assert_eq!(looped.frames[30], p.frames[40]);
        assert_eq!(looped.frames[39], p.frames[49]);
        let once = p.resume_segment(10, false, 40, 80).unwrap();
        assert_eq!(
            once.frames.len(),
            90,
            "without looping it plays out from the elapsed frame"
        );
        assert_eq!(once.frames[0], p.frames[10]);
    }

    #[test]
    fn resume_keeps_phase() {
        let p = tone(100);
        let once = p.resume_at(30, false).unwrap();
        assert_eq!(once.frames.len(), 70);
        assert_eq!(once.frames[0], p.frames[30]);
        assert!(p.resume_at(100, false).is_none());
        let looped = p.resume_at(230, true).unwrap();
        assert_eq!(looped.frames.len(), 100);
        assert_eq!(looped.frames[0], p.frames[30]);
        assert_eq!(looped.frames[70], p.frames[0]);
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(decode(b"not audio").is_none());
    }

    #[test]
    fn multiple_packets_preserve_stereo_samples_and_partial_tail() {
        let p = Pcm {
            rate: 44100,
            frames: (0..100_003)
                .map(|i| [(i as f32 * 0.05).sin() * 0.5, (i as f32 * 0.03).cos() * 0.4])
                .collect(),
        };
        let back = decode(&p.to_wav()).unwrap();
        assert_eq!(back.rate, p.rate);
        assert_eq!(back.frames.len(), p.frames.len());
        for (actual, expected) in back.frames.iter().zip(&p.frames) {
            for c in 0..2 {
                assert!((actual[c] - expected[c]).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn multiple_mono_packets_duplicate_channels_without_stale_tail() {
        let p = tone(100_003);
        let stereo = p.to_wav();
        let mut mono = stereo[..44].to_vec();
        let size = p.frames.len() as u32 * 2;
        mono[4..8].copy_from_slice(&(36 + size).to_le_bytes());
        mono[22..24].copy_from_slice(&1u16.to_le_bytes());
        mono[28..32].copy_from_slice(&(p.rate * 2).to_le_bytes());
        mono[32..34].copy_from_slice(&2u16.to_le_bytes());
        mono[40..44].copy_from_slice(&size.to_le_bytes());
        for frame in stereo[44..].as_chunks::<4>().0 {
            mono.extend_from_slice(&frame[..2]);
        }
        let back = decode(&mono).unwrap();
        assert_eq!(back.rate, p.rate);
        assert_eq!(back.frames.len(), p.frames.len());
        for (actual, expected) in back.frames.iter().zip(&p.frames) {
            assert_eq!(actual[0], actual[1]);
            assert!((actual[0] - expected[0]).abs() < 1e-4);
        }
    }
}
