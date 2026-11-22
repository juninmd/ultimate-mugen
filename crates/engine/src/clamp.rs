//! Bounds helper for values that come from content we do not control.
//!
//! `f32::clamp` panics when the minimum is above the maximum, and a M.U.G.E.N. pack can ship `[Data] life = -5`,
//! inverted camera bounds, inverted lane limits or a `lifemax` below the current life. Any of those used to abort the
//! whole match (and, on the web build, the whole page). `ordered` keeps the value inside the two limits without ever
//! panicking, swapping them when the data is backwards.
pub fn ordered(value: f32, a: f32, b: f32) -> f32 {
    if a <= b {
        value.clamp(a, b)
    } else {
        value.clamp(b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::ordered;

    #[test]
    fn keeps_the_value_inside_forward_bounds() {
        assert_eq!(ordered(5.0, 0.0, 10.0), 5.0);
        assert_eq!(ordered(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(ordered(50.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn backwards_bounds_order_themselves_instead_of_panicking() {
        assert_eq!(ordered(50.0, 10.0, -5.0), 10.0);
        assert_eq!(ordered(-50.0, 10.0, -5.0), -5.0);
        assert_eq!(ordered(0.0, 10.0, -5.0), 0.0);
    }
}
