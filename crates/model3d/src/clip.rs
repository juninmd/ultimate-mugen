use crate::accessor::Source;
use crate::error::ModelError;
use crate::math::{lerp3, slerp};
use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Path {
    Translation,
    Rotation,
    Scale,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub node: usize,
    pub path: Path,
    pub times: Vec<f32>,
    pub values: Vec<f32>,
    pub step: bool,
}

#[derive(Clone, Debug)]
pub struct Clip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<Track>,
}

pub fn read_clips(src: &Source) -> Result<Vec<Clip>, ModelError> {
    let mut clips = Vec::new();
    for (index, a) in src.json["animations"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let samplers = a["samplers"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let mut tracks = Vec::new();
        for ch in a["channels"].as_array().into_iter().flatten() {
            let path = match ch["target"]["path"].as_str() {
                Some("translation") => Path::Translation,
                Some("rotation") => Path::Rotation,
                Some("scale") => Path::Scale,
                _ => continue,
            };
            let Some(node) = ch["target"]["node"].as_u64() else {
                continue;
            };
            let s: &Value = samplers
                .get(ch["sampler"].as_u64().unwrap_or(u64::MAX) as usize)
                .ok_or(ModelError::OutOfRange("sampler"))?;
            let interpolation = s["interpolation"].as_str().unwrap_or("LINEAR");
            if interpolation == "CUBICSPLINE" {
                return Err(ModelError::Unsupported("cubic spline interpolation"));
            }
            let times = src
                .floats(
                    s["input"]
                        .as_u64()
                        .ok_or(ModelError::Missing("sampler input"))? as usize,
                )?
                .0;
            let values = src
                .floats(
                    s["output"]
                        .as_u64()
                        .ok_or(ModelError::Missing("sampler output"))? as usize,
                )?
                .0;
            tracks.push(Track {
                node: node as usize,
                path,
                times,
                values,
                step: interpolation == "STEP",
            });
        }
        let duration = tracks
            .iter()
            .filter_map(|t| t.times.last().copied())
            .fold(0.0, f32::max);
        let name = a["name"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| format!("clip{index}"));
        clips.push(Clip {
            name,
            duration,
            tracks,
        });
    }
    Ok(clips)
}

fn bracket(times: &[f32], t: f32) -> (usize, usize, f32) {
    let last = times.len() - 1;
    if t.is_nan() || t <= times[0] {
        return (0, 0, 0.0);
    }
    if t >= times[last] {
        return (last, last, 0.0);
    }
    let hi = times.partition_point(|&x| x <= t);
    let lo = hi.saturating_sub(1);
    (
        lo,
        hi,
        (t - times[lo]) / (times[hi] - times[lo]).max(f32::EPSILON),
    )
}

impl Track {
    pub fn sample<const N: usize>(&self, t: f32) -> Option<[f32; N]> {
        if self.times.is_empty() || self.values.len() < self.times.len() * N {
            return None;
        }
        let (lo, hi, k) = bracket(&self.times, t);
        let at = |i: usize| -> [f32; N] { std::array::from_fn(|n| self.values[i * N + n]) };
        let (a, b) = (at(lo), at(hi));
        if self.step || lo == hi {
            return Some(a);
        }
        Some(match N {
            4 => {
                let q = slerp([a[0], a[1], a[2], a[3]], [b[0], b[1], b[2], b[3]], k);
                std::array::from_fn(|n| q[n])
            }
            _ => {
                let v = lerp3([a[0], a[1], a[2]], [b[0], b[1], b[2]], k);
                std::array::from_fn(|n| v[n])
            }
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nan_time_samples_the_first_key() {
        assert_eq!(super::bracket(&[0.0, 1.0, 2.0], f32::NAN), (0, 0, 0.0));
        assert_eq!(super::bracket(&[f32::NAN, 1.0], 0.5).0, 0);
    }
}
