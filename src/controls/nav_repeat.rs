const REPEATING: u32 = 1 | 2 | 4 | 8;
const FIRST_DELAY: f32 = 0.38;
const INTERVAL: f32 = 0.075;

#[derive(Default)]
pub struct NavRepeat {
    held: u32,
    waits: [f32; 4],
}

impl NavRepeat {
    pub fn pressed(&mut self, down: u32, dt: f32) -> u32 {
        let fresh = down & !self.held;
        let mut out = fresh;
        for (slot, wait) in self.waits.iter_mut().enumerate() {
            let bit = 1u32 << slot;
            if REPEATING & bit == 0 {
                continue;
            }
            if fresh & bit != 0 {
                *wait = FIRST_DELAY;
            } else if down & bit != 0 {
                *wait -= dt.max(0.0);
                if *wait <= 0.0 {
                    *wait += INTERVAL;
                    if *wait <= 0.0 {
                        *wait = INTERVAL;
                    }
                    out |= bit;
                }
            }
        }
        self.held = down;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: u32 = 1;
    const OK: u32 = 16;

    fn presses_while_held(bits: u32, seconds: f32) -> usize {
        let mut repeat = NavRepeat::default();
        let frames = (seconds * 60.0) as usize;
        (0..frames)
            .filter(|_| repeat.pressed(bits, 1.0 / 60.0) & bits != 0)
            .count()
    }

    #[test]
    fn a_tap_gives_exactly_one_press() {
        let mut repeat = NavRepeat::default();
        assert_eq!(repeat.pressed(UP, 1.0 / 60.0), UP);
        assert_eq!(repeat.pressed(0, 1.0 / 60.0), 0);
        assert_eq!(repeat.pressed(UP, 1.0 / 60.0), UP);
    }

    #[test]
    fn holding_waits_before_repeating() {
        assert_eq!(presses_while_held(UP, 0.3), 1);
    }

    #[test]
    fn holding_repeats_steadily_after_the_delay() {
        let held = presses_while_held(UP, 1.38);
        assert!((12..=15).contains(&held), "{held}");
    }

    #[test]
    fn the_confirm_button_never_repeats() {
        assert_eq!(presses_while_held(OK, 2.0), 1);
    }

    #[test]
    fn a_long_frame_gives_one_press_not_a_burst() {
        let mut repeat = NavRepeat::default();
        repeat.pressed(UP, 0.0);
        assert_eq!(repeat.pressed(UP, 5.0), UP);
        assert_eq!(repeat.pressed(UP, 0.0), 0);
    }
}
