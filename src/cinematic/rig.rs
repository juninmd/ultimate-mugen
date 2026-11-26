use super::{frame, Frame, Subject};
use ume_engine::matchs::Match;

const FOLLOW_RATE: f32 = 6.0;
const CHEST_SHARE: f32 = 0.55;
const SNAP_BIAS: f32 = 0.01;

#[derive(Clone, Copy, Default)]
struct Target {
    zoom: f32,
    bias: f32,
    point: Option<(f32, f32)>,
    shake: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Rig {
    zoom: f32,
    bias: f32,
    point: (f32, f32),
    shake: f32,
    target: Target,
}

fn chest(f: &ume_engine::fighter::Fighter) -> (f32, f32) {
    (
        f.pos.x,
        f.pos.y + f.lane - f.def.c("size.height", 60.0) as f32 * CHEST_SHARE,
    )
}

fn subject_point(m: &Match, frame: &Frame) -> Option<(f32, f32)> {
    if frame.subject == Subject::Both {
        return None;
    }
    let (sum, n) = m
        .roots()
        .filter(|f| frame.covers(f.side.min(1), m.winner_side))
        .map(chest)
        .fold(((0.0, 0.0), 0.0), |(a, n), c| ((a.0 + c.0, a.1 + c.1), n + 1.0));
    (n > 0.0).then(|| (sum.0 / n, sum.1 / n))
}

impl Rig {
    pub fn retarget(&mut self, m: &Match) {
        self.target = match frame(m) {
            Some(frame) => {
                let point = subject_point(m, &frame);
                Target {
                    zoom: frame.sample.zoom,
                    bias: point.map_or(0.0, |_| frame.sample.focus),
                    point,
                    shake: frame.sample.shake,
                }
            }
            None => Target::default(),
        };
        self.shake = self.target.shake;
    }

    pub fn ease(&mut self, dt: f32) {
        let k = (FOLLOW_RATE * dt).min(1.0);
        if let Some(point) = self.target.point {
            self.point = if self.bias < SNAP_BIAS {
                point
            } else {
                (
                    self.point.0 + (point.0 - self.point.0) * k,
                    self.point.1 + (point.1 - self.point.1) * k,
                )
            };
        }
        self.zoom += (self.target.zoom - self.zoom) * k;
        self.bias += (self.target.bias - self.bias) * k;
    }

    pub fn zoom(&self, gain: f32) -> f32 {
        self.zoom * gain
    }

    pub fn shake(&self, gain: f32) -> f32 {
        self.shake * gain
    }

    pub fn focus(&self, mid: (f32, f32), gain: f32) -> (f32, f32) {
        let b = (self.bias * gain).clamp(0.0, 1.0);
        (
            mid.0 + (self.point.0 - mid.0) * b,
            mid.1 + (self.point.1 - mid.1) * b,
        )
    }
}
