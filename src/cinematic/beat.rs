use super::ease::Ease;
use serde::Deserialize;

const MIN_DURATION: f32 = 0.05;
const MAX_DURATION: f32 = 10.0;
const MAX_YAW: f32 = std::f32::consts::TAU * 2.0;
const MAX_PITCH: f32 = 1.2;
const MAX_SWEEP: f32 = 200.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subject {
    #[default]
    Both,
    P1,
    P2,
    Winner,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sample {
    pub zoom: f32,
    pub focus: f32,
    pub yaw: f32,
    pub shake: f32,
    pub pitch: f32,
    pub lens: f32,
    pub detail: f32,
    pub aim: f32,
    pub sweep: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Beat {
    pub subject: Subject,
    pub duration: f32,
    pub ease: Ease,
    pub zoom: [f32; 2],
    pub focus: [f32; 2],
    pub yaw: [f32; 2],
    pub shake: [f32; 2],
    pub pitch: [f32; 2],
    pub lens: [f32; 2],
    pub detail: [f32; 2],
    pub aim: [f32; 2],
    pub sweep: [f32; 2],
}

impl Default for Beat {
    fn default() -> Self {
        Beat {
            subject: Subject::Both,
            duration: 1.0,
            ease: Ease::InOut,
            zoom: [0.0; 2],
            focus: [0.0; 2],
            yaw: [0.0; 2],
            shake: [0.0; 2],
            pitch: [0.0; 2],
            lens: [0.0; 2],
            detail: [0.0; 2],
            aim: [0.55; 2],
            sweep: [0.0; 2],
        }
    }
}

fn clamp_pair(pair: [f32; 2], lo: f32, hi: f32) -> [f32; 2] {
    let fix = |v: f32| if v.is_finite() { v.clamp(lo, hi) } else { 0.0 };
    [fix(pair[0]), fix(pair[1])]
}

fn mix(pair: [f32; 2], p: f32) -> f32 {
    pair[0] + (pair[1] - pair[0]) * p
}

impl Beat {
    pub fn sanitized(mut self) -> Beat {
        self.duration = if self.duration.is_finite() {
            self.duration.clamp(MIN_DURATION, MAX_DURATION)
        } else {
            1.0
        };
        self.zoom = clamp_pair(self.zoom, -0.5, 1.5);
        self.focus = clamp_pair(self.focus, 0.0, 1.0);
        self.yaw = clamp_pair(self.yaw, -MAX_YAW, MAX_YAW);
        self.shake = clamp_pair(self.shake, 0.0, 1.0);
        self.pitch = clamp_pair(self.pitch, -MAX_PITCH, MAX_PITCH);
        self.lens = clamp_pair(self.lens, 0.0, 0.6);
        self.detail = clamp_pair(self.detail, 0.0, 2.0);
        self.aim = clamp_pair(self.aim, 0.0, 1.0);
        self.sweep = clamp_pair(self.sweep, -MAX_SWEEP, MAX_SWEEP);
        self
    }

    pub fn at(&self, progress: f32) -> Sample {
        let p = self.ease.apply(progress);
        Sample {
            zoom: mix(self.zoom, p),
            focus: mix(self.focus, p),
            yaw: mix(self.yaw, p),
            shake: mix(self.shake, p),
            pitch: mix(self.pitch, p),
            lens: mix(self.lens, p),
            detail: mix(self.detail, p),
            aim: mix(self.aim, p),
            sweep: mix(self.sweep, p),
        }
    }
}

pub fn sample(beats: &[Beat], seconds: f32) -> Option<(Subject, Sample)> {
    let mut start = 0.0;
    for beat in beats {
        if seconds < start + beat.duration {
            return Some((beat.subject, beat.at((seconds - start) / beat.duration)));
        }
        start += beat.duration;
    }
    beats.last().map(|beat| (beat.subject, beat.at(1.0)))
}
