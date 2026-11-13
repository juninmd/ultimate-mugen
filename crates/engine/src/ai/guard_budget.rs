const GUARD_LIMIT: i32 = 48;
const GUARD_REST: i32 = 45;

#[derive(Default)]
pub(super) struct GuardBudget {
    run: i32,
    rest: i32,
}

impl GuardBudget {
    pub(super) fn can_guard(&self) -> bool {
        self.rest == 0
    }

    pub(super) fn on_guard(&mut self, ticks: i32) {
        self.run += ticks;
        if self.run >= GUARD_LIMIT {
            self.run = 0;
            self.rest = GUARD_REST;
        }
    }

    pub(super) fn on_free(&mut self) {
        self.run = (self.run - 1).max(0);
    }

    pub(super) fn on_rest_tick(&mut self) {
        self.rest = (self.rest - 1).max(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_guarding_hits_the_limit_and_rests() {
        let mut g = GuardBudget::default();
        let mut blocks = 0;
        while g.can_guard() && blocks < 100 {
            g.on_guard(14);
            blocks += 1;
        }
        assert_eq!(blocks, 4);
        assert!(!g.can_guard());
        for _ in 0..GUARD_REST {
            assert!(!g.can_guard());
            g.on_rest_tick();
        }
        assert!(g.can_guard());
    }

    #[test]
    fn idle_time_restores_the_budget() {
        let mut g = GuardBudget::default();
        g.on_guard(14);
        g.on_guard(14);
        for _ in 0..40 {
            g.on_free();
        }
        g.on_guard(14);
        g.on_guard(14);
        g.on_guard(14);
        assert!(g.can_guard());
    }
}
