#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRule {
    pub max_multiplier: f32,
    pub start_ticks: i32,
    pub ramp_ticks: i32,
    pub exponent: f32,
}

impl TimeRule {
    pub fn progress(&self, fight_ticks: i32) -> f32 {
        let elapsed = (fight_ticks - self.start_ticks).max(0) as f32;
        (elapsed / self.ramp_ticks.max(1) as f32).clamp(0.0, 1.0)
    }

    pub fn multiplier(&self, fight_ticks: i32) -> f32 {
        let progress = self.progress(fight_ticks);
        let shaped = if self.exponent == 1.0 {
            progress
        } else {
            progress.powf(self.exponent)
        };
        1.0 + (self.max_multiplier - 1.0) * shaped
    }
}
