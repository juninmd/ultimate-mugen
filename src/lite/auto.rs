pub const WINDOW_SECONDS: f32 = 4.0;
pub const SLOW_FRAME_MS: f32 = 24.0;
pub const WARMUP_SECONDS: f32 = 3.0;

#[derive(Default)]
pub struct Monitor {
    warmup: f32,
    elapsed: f32,
    frames: u32,
}

impl Monitor {
    pub fn reset(&mut self) {
        *self = Monitor::default();
    }

    pub fn feed(&mut self, dt: f32) -> bool {
        if dt > 0.25 {
            return false;
        }
        if self.warmup < WARMUP_SECONDS {
            self.warmup += dt;
            return false;
        }
        self.elapsed += dt;
        self.frames += 1;
        if self.elapsed < WINDOW_SECONDS {
            return false;
        }
        let slow = self.elapsed / self.frames as f32 * 1000.0 > SLOW_FRAME_MS;
        self.elapsed = 0.0;
        self.frames = 0;
        slow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dt: f32, seconds: f32) -> bool {
        let mut m = Monitor::default();
        let mut hit = false;
        for _ in 0..(seconds / dt) as u32 {
            hit |= m.feed(dt);
        }
        hit
    }

    #[test]
    fn slow_frames_trigger_after_warmup() {
        assert!(run(1.0 / 20.0, 9.0));
        assert!(!run(1.0 / 20.0, 3.5));
    }

    #[test]
    fn smooth_frames_never_trigger() {
        assert!(!run(1.0 / 60.0, 30.0));
    }

    #[test]
    fn hitches_are_ignored() {
        let mut m = Monitor::default();
        assert!(!m.feed(1.0));
    }
}
