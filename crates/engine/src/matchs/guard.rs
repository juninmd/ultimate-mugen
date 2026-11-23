use super::{Event, Match};

pub const GUARD_TIME_DIVISOR: i32 = 3;
pub const PERFECT_GUARD_HEAL_RATIO: f32 = 0.015;
pub const PERFECT_GUARD_METER: f32 = 120.0;
pub const PERFECT_GUARD_PUSHBACK_SCALE: f32 = 0.65;

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

pub(super) fn apply_perfect_guard_reward(m: &mut Match, d: usize) {
    if m.fighters[d].rush.perfect != crate::rush::PERFECT_FLASH {
        return;
    }
    let (fighter, side, healed) = {
        let fd = &mut m.fighters[d];
        let before = fd.life;
        let heal_amt = (fd.life_max * PERFECT_GUARD_HEAL_RATIO).round();
        fd.life = (fd.life + heal_amt).min(fd.life_max);
        fd.ghv.xvel *= PERFECT_GUARD_PUSHBACK_SCALE;
        (fd.id, fd.side, (fd.life - before).max(0.0))
    };
    m.add_power(d, PERFECT_GUARD_METER);
    m.events.push(Event::PerfectGuard {
        fighter,
        side,
        healed,
        meter: PERFECT_GUARD_METER,
    });
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
