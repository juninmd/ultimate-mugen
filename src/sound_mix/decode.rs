use super::Pcm;
use std::io::Cursor;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
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
