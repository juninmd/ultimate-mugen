pub const START_GAIN: f32 = 3.0;
pub const GAIN_GROWTH_PER_TICK: f32 = 0.08;
pub const MAX_GAIN: f32 = 16.0;

pub fn gain_at(held_ticks: i32) -> f32 {
    (START_GAIN + held_ticks.max(0) as f32 * GAIN_GROWTH_PER_TICK).min(MAX_GAIN)
}

pub fn gain_expression() -> String {
    format!("min({MAX_GAIN:?}, {START_GAIN:?} + Time * {GAIN_GROWTH_PER_TICK:?})")
}

pub fn ticks_to_fill(power_max: f32) -> i32 {
    let (mut power, mut ticks) = (0.0, 0);
    while power < power_max && ticks < 100_000 {
        power += gain_at(ticks);
        ticks += 1;
    }
    ticks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_accelerates_then_caps() {
        assert_eq!(gain_at(0), START_GAIN);
        assert!(gain_at(30) > gain_at(10));
        assert_eq!(gain_at(10_000), MAX_GAIN);
        assert!((0..400).all(|t| gain_at(t + 1) >= gain_at(t)));
    }

    #[test]
    fn negative_ticks_restart_the_ramp() {
        assert_eq!(gain_at(-5), START_GAIN);
    }

    #[test]
    fn full_bar_takes_a_few_seconds() {
        let seconds = ticks_to_fill(3000.0) as f32 / 60.0;
        assert!((3.0..=6.0).contains(&seconds), "{seconds}");
    }

    #[test]
    fn expression_matches_constants() {
        assert_eq!(gain_expression(), "min(16.0, 3.0 + Time * 0.08)");
    }
}
