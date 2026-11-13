use crate::input::k;

mod cancel;

pub const COST: f32 = 500.0;
pub const CONTACT_WINDOW: i32 = 45;
pub const INPUT_BUFFER: i32 = 6;
pub const COOLDOWN_TICKS: i32 = 40;
pub const OVERDRIVE_TICKS: i32 = 150;
pub const OVERDRIVE_ATTACK_GAIN: f32 = 1.2;
pub const VICTIM_STUN_BONUS: i32 = 8;
pub const SLOW_MOTION_TICKS: i32 = 15;
pub const FLASH_TICKS: i32 = 10;

#[derive(Clone, Debug, Default)]
pub struct RushState {
    pub overdrive: i32,
    pub cooldown: i32,
    pub cancels: i32,
    buffered: i32,
    held: bool,
    locked: bool,
}

impl RushState {
    pub fn sense(&mut self, bits: u32) -> u32 {
        let held = bits & k::S != 0;
        if held && !self.held {
            self.buffered = INPUT_BUFFER;
        }
        self.held = held;
        if self.locked {
            if held {
                return bits & !k::S;
            }
            self.locked = false;
        }
        bits
    }

    pub fn tick(&mut self) {
        self.overdrive = (self.overdrive - 1).max(0);
        self.cooldown = (self.cooldown - 1).max(0);
        self.buffered = (self.buffered - 1).max(0);
    }

    pub fn requested(&self) -> bool {
        self.buffered > 0
    }

    pub fn attack_gain(&self) -> f32 {
        if self.overdrive > 0 {
            OVERDRIVE_ATTACK_GAIN
        } else {
            1.0
        }
    }

    pub fn overdrive_ratio(&self) -> f32 {
        self.overdrive as f32 / OVERDRIVE_TICKS as f32
    }

    pub(crate) fn begin(&mut self) {
        self.overdrive = OVERDRIVE_TICKS;
        self.cooldown = COOLDOWN_TICKS;
        self.cancels += 1;
        self.buffered = 0;
        self.locked = self.held;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_is_buffered_and_expires() {
        let mut r = RushState::default();
        r.sense(k::S);
        assert!(r.requested());
        for _ in 0..INPUT_BUFFER {
            r.tick();
        }
        assert!(!r.requested());
    }

    #[test]
    fn holding_does_not_rebuffer() {
        let mut r = RushState::default();
        r.sense(k::S);
        for _ in 0..INPUT_BUFFER {
            r.tick();
            r.sense(k::S);
        }
        assert!(!r.requested());
    }

    #[test]
    fn button_is_masked_until_released_after_a_cancel() {
        let mut r = RushState::default();
        r.sense(k::S);
        r.begin();
        assert_eq!(r.sense(k::S | k::X), k::X);
        assert_eq!(r.sense(0), 0);
        assert_eq!(r.sense(k::S), k::S);
    }

    #[test]
    fn overdrive_boosts_attack_then_fades() {
        let mut r = RushState::default();
        assert_eq!(r.attack_gain(), 1.0);
        r.begin();
        assert_eq!(r.attack_gain(), OVERDRIVE_ATTACK_GAIN);
        for _ in 0..OVERDRIVE_TICKS {
            r.tick();
        }
        assert_eq!(r.attack_gain(), 1.0);
    }
}
