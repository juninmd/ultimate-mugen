pub const GUARD_TIME_DIVISOR: i32 = 3;

pub(super) fn scaled_ticks(ticks: i32) -> i32 {
    if ticks <= 0 {
        ticks
    } else {
        ((ticks + 1) / GUARD_TIME_DIVISOR).max(1)
    }
}

pub(super) fn guard_hit_time(base: i32, perfect: bool) -> i32 {
    let reduced = scaled_ticks(base);
    if perfect && reduced > 0 {
        (reduced * 2 / 3).max(1)
    } else {
        reduced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_guard_timings_shrink_threefold_with_a_one_tick_floor() {
        assert_eq!(scaled_ticks(0), 0);
        assert_eq!(scaled_ticks(-1), -1);
        assert_eq!(scaled_ticks(1), 1);
        assert_eq!(scaled_ticks(7), 2);
        assert_eq!(scaled_ticks(9), 3);
        assert_eq!(scaled_ticks(12), 4);
        assert_eq!(scaled_ticks(16), 5);
        assert_eq!(scaled_ticks(20), 7);
    }

    #[test]
    fn perfect_guard_trims_the_reduced_stun_further() {
        assert_eq!(guard_hit_time(12, false), 4);
        assert_eq!(guard_hit_time(12, true), 2);
        assert_eq!(guard_hit_time(20, false), 7);
        assert_eq!(guard_hit_time(20, true), 4);
        assert_eq!(guard_hit_time(0, true), 0);
    }
}
