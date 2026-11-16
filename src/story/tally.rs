const LIFE_BONUS_BASE: f32 = 2000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageTally {
    pub clear: i64,
    pub life: i64,
}

impl StageTally {
    pub fn new(stage: usize, clear: i64, remaining_life: f32) -> Self {
        let ratio = remaining_life.clamp(0.0, 1.0);
        StageTally {
            clear,
            life: (ratio * LIFE_BONUS_BASE * (stage as f32 + 1.0)) as i64,
        }
    }

    pub fn total(&self) -> i64 {
        self.clear + self.life
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fuller_life_bar_pays_more_and_later_stages_pay_more() {
        assert!(StageTally::new(0, 0, 1.0).life > StageTally::new(0, 0, 0.4).life);
        assert!(StageTally::new(2, 0, 0.5).life > StageTally::new(0, 0, 0.5).life);
        assert_eq!(StageTally::new(0, 100, 0.0).total(), 100);
    }

    #[test]
    fn out_of_range_life_is_clamped() {
        assert_eq!(StageTally::new(0, 0, 3.0).life, StageTally::new(0, 0, 1.0).life);
        assert_eq!(StageTally::new(0, 0, -1.0).life, 0);
    }
}
