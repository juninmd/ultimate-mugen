#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    from: f32,
    to: f32,
    elapsed: f32,
    duration: f32,
}

impl Tween {
    pub fn settled(value: f32) -> Self {
        Self { from: value, to: value, elapsed: 0.0, duration: 0.0 }
    }

    pub fn retarget(&mut self, to: f32, duration: f32) {
        self.from = self.value();
        self.to = to;
        self.elapsed = 0.0;
        self.duration = duration.max(0.0);
    }

    pub fn advance(&mut self, dt: f32) {
        self.elapsed = (self.elapsed + dt.max(0.0)).min(self.duration);
    }

    pub fn value(&self) -> f32 {
        if self.duration <= 0.0 {
            return self.to;
        }
        let t = self.elapsed / self.duration;
        self.from + (self.to - self.from) * ease_out_cubic(t)
    }

    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}
