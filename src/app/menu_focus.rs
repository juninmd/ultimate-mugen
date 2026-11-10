pub const MAX_ITEMS: usize = 24;
const SETTLE_RATE: f32 = 16.0;

pub struct MenuFocus {
    levels: [f32; MAX_ITEMS],
}

impl Default for MenuFocus {
    fn default() -> Self {
        MenuFocus {
            levels: [0.0; MAX_ITEMS],
        }
    }
}

impl MenuFocus {
    pub fn settle(&mut self, focused: usize, dt: f32) {
        let blend = 1.0 - (-SETTLE_RATE * dt.max(0.0)).exp();
        for (i, level) in self.levels.iter_mut().enumerate() {
            let target = if i == focused { 1.0 } else { 0.0 };
            *level += (target - *level) * blend;
        }
    }

    pub fn snap(&mut self, focused: usize) {
        for (i, level) in self.levels.iter_mut().enumerate() {
            *level = if i == focused { 1.0 } else { 0.0 };
        }
    }

    pub fn level(&self, item: usize) -> f32 {
        let raw = self.levels.get(item).copied().unwrap_or(0.0);
        raw * raw * (3.0 - 2.0 * raw)
    }
}

pub fn mix(from: f32, to: f32, k: f32) -> f32 {
    from + (to - from) * k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_moves_smoothly_and_lands_on_the_new_item() {
        let mut focus = MenuFocus::default();
        focus.snap(0);
        focus.settle(1, 1.0 / 60.0);
        assert!(focus.level(0) < 1.0 && focus.level(0) > 0.5);
        assert!(focus.level(1) > 0.0 && focus.level(1) < 0.5);
        for _ in 0..60 {
            focus.settle(1, 1.0 / 60.0);
        }
        assert!(focus.level(0) < 0.001 && focus.level(1) > 0.999);
    }

    #[test]
    fn the_transition_takes_the_same_time_at_any_frame_rate() {
        let (mut at_60, mut at_144) = (MenuFocus::default(), MenuFocus::default());
        for _ in 0..12 {
            at_60.settle(3, 1.0 / 60.0);
        }
        for _ in 0..(12 * 144 / 60) {
            at_144.settle(3, 1.0 / 144.0);
        }
        assert!((at_60.level(3) - at_144.level(3)).abs() < 0.02);
    }

    #[test]
    fn items_outside_the_table_are_never_focused() {
        let mut focus = MenuFocus::default();
        focus.settle(MAX_ITEMS + 3, 0.5);
        assert_eq!(focus.level(MAX_ITEMS + 3), 0.0);
        assert_eq!(mix(22.0, 24.0, 0.5), 23.0);
    }
}
