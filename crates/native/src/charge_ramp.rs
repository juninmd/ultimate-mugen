use super::ability_spec::ChargeGain;

pub const START_GAIN: f32 = 9.0;
pub const GAIN_GROWTH_PER_TICK: f32 = 0.24;
pub const MAX_GAIN: f32 = 48.0;

pub fn gain_at(held_ticks: i32) -> f32 {
    (START_GAIN + held_ticks.max(0) as f32 * GAIN_GROWTH_PER_TICK).min(MAX_GAIN)
}

pub fn gain_expression() -> String {
    gain_expression_for(None)
}

pub fn gain_expression_for(gain: Option<&ChargeGain>) -> String {
    let pick = |value: Option<f32>, default: f32| value.filter(|v| v.is_finite() && *v >= 0.0).unwrap_or(default);
    let start = pick(gain.and_then(|g| g.start), START_GAIN);
    let growth = pick(gain.and_then(|g| g.growth), GAIN_GROWTH_PER_TICK);
    let max = pick(gain.and_then(|g| g.max), MAX_GAIN).max(start);
    format!("min({max:?}, {start:?} + Time * {growth:?})")
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
        assert!((1.0..=3.0).contains(&seconds), "{seconds}");
    }

    #[test]
    fn a_fighter_without_gain_keeps_the_built_in_ramp() {
        assert_eq!(gain_expression_for(None), gain_expression());
        assert_eq!(gain_expression_for(Some(&ChargeGain::default())), gain_expression());
    }

    #[test]
    fn a_flat_gain_is_expressed_as_a_constant_per_tick() {
        let flat = ChargeGain { start: Some(6.2), growth: Some(0.0), max: Some(6.2) };
        assert_eq!(gain_expression_for(Some(&flat)), "min(6.2, 6.2 + Time * 0.0)");
    }

    #[test]
    fn invalid_gain_fields_fall_back_and_max_never_drops_below_start() {
        let bad = ChargeGain { start: Some(-3.0), growth: Some(f32::NAN), max: Some(2.0) };
        assert_eq!(gain_expression_for(Some(&bad)), "min(9.0, 9.0 + Time * 0.24)");
        let low = ChargeGain { start: Some(20.0), growth: None, max: Some(5.0) };
        assert_eq!(gain_expression_for(Some(&low)), "min(20.0, 20.0 + Time * 0.24)");
    }

    #[test]
    fn expression_matches_constants() {
        assert_eq!(gain_expression(), "min(48.0, 9.0 + Time * 0.24)");
    }
}
