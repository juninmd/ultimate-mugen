mod store;
pub use store::{load, save};

pub const SUPER_COST: f32 = 1000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Goal {
    Hits(i32),
    Damage(f32),
    SuperHits(i32),
    CleanHits(i32),
}

pub const LADDER: [Goal; 7] = [
    Goal::Hits(3),
    Goal::Hits(5),
    Goal::Hits(8),
    Goal::Damage(0.25),
    Goal::SuperHits(6),
    Goal::CleanHits(10),
    Goal::Damage(0.45),
];

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Sample {
    pub hits: i32,
    pub damage: f32,
    pub life_max: f32,
    pub power: f32,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Finished {
    pub hits: i32,
    pub damage_share: f32,
    pub meter_spent: f32,
}

impl Goal {
    pub fn met(self, f: &Finished) -> bool {
        match self {
            Goal::Hits(n) => f.hits >= n,
            Goal::Damage(share) => f.damage_share >= share,
            Goal::SuperHits(n) => f.hits >= n && f.meter_spent >= SUPER_COST,
            Goal::CleanHits(n) => f.hits >= n && f.meter_spent < 1.0,
        }
    }
}

#[derive(Default)]
pub struct Tracker {
    pub cleared: usize,
    pub last: Option<Finished>,
    pub flash: f32,
    pub demo: Option<Sample>,
    live: Option<Live>,
    idle_power: f32,
}

#[derive(Clone, Copy)]
struct Live {
    hits: i32,
    damage: f32,
    life_max: f32,
    start_power: f32,
    min_power: f32,
}

impl Tracker {
    pub fn with_progress(cleared: usize) -> Self {
        Tracker { cleared: cleared.min(LADDER.len()), ..Default::default() }
    }

    pub fn goal(&self) -> Option<Goal> {
        LADDER.get(self.cleared).copied()
    }

    pub fn live_hits(&self) -> i32 {
        self.live.map_or(0, |l| l.hits)
    }

    pub fn live_share(&self) -> f32 {
        self.live.map_or(0.0, |l| if l.life_max > 0.0 { l.damage / l.life_max } else { 0.0 })
    }

    pub fn tick(&mut self, dt: f32) {
        self.flash = (self.flash - dt).max(0.0);
    }

    pub fn observe(&mut self, s: Sample) -> Option<usize> {
        if !(s.damage.is_finite() && s.power.is_finite() && s.life_max.is_finite()) {
            return None;
        }
        if s.hits <= 0 {
            self.idle_power = s.power;
            return self.finish();
        }
        let start = self.idle_power;
        let live = self.live.get_or_insert(Live { hits: 0, damage: 0.0, life_max: s.life_max, start_power: start, min_power: start });
        live.hits = live.hits.max(s.hits);
        live.damage = s.damage.max(live.damage);
        live.min_power = live.min_power.min(s.power);
        None
    }

    fn finish(&mut self) -> Option<usize> {
        let l = self.live.take()?;
        let done = Finished {
            hits: l.hits,
            damage_share: if l.life_max > 0.0 { l.damage / l.life_max } else { 0.0 },
            meter_spent: (l.start_power - l.min_power).max(0.0),
        };
        self.last = Some(done);
        let goal = self.goal()?;
        if !goal.met(&done) {
            return None;
        }
        self.cleared += 1;
        self.flash = 2.2;
        Some(self.cleared - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(t: &mut Tracker, hits: i32, damage: f32, power: [f32; 2]) -> Option<usize> {
        t.observe(Sample { hits: 0, damage: 0.0, life_max: 1000.0, power: power[0] });
        t.observe(Sample { hits, damage, life_max: 1000.0, power: power[1] });
        t.observe(Sample { hits: 0, damage: 0.0, life_max: 1000.0, power: power[1] })
    }

    #[test]
    fn a_combo_only_counts_once_it_ends() {
        let mut t = Tracker::default();
        assert_eq!(t.observe(Sample { hits: 4, damage: 100.0, life_max: 1000.0, power: 0.0 }), None);
        assert_eq!(t.cleared, 0);
        assert_eq!(t.observe(Sample { hits: 0, ..Default::default() }), Some(0));
        assert_eq!(t.cleared, 1);
    }

    #[test]
    fn too_short_a_combo_clears_nothing_but_is_remembered() {
        let mut t = Tracker::default();
        assert_eq!(feed(&mut t, 2, 50.0, [0.0, 0.0]), None);
        assert_eq!(t.last.map(|f| f.hits), Some(2));
    }

    #[test]
    fn goals_are_cleared_in_order_never_skipped() {
        let mut t = Tracker::default();
        assert_eq!(feed(&mut t, 12, 400.0, [0.0, 0.0]), Some(0));
        assert_eq!(feed(&mut t, 12, 400.0, [0.0, 0.0]), Some(1));
        assert_eq!(t.goal(), Some(Goal::Hits(8)));
    }

    #[test]
    fn the_super_trial_needs_meter_and_the_clean_trial_forbids_it() {
        let super_goal = Goal::SuperHits(6);
        let clean_goal = Goal::CleanHits(10);
        let spent = Finished { hits: 9, damage_share: 0.3, meter_spent: 1000.0 };
        let kept = Finished { hits: 12, damage_share: 0.3, meter_spent: 0.0 };
        assert!(super_goal.met(&spent) && !super_goal.met(&kept));
        assert!(clean_goal.met(&kept) && !clean_goal.met(&Finished { hits: 12, damage_share: 0.3, meter_spent: 500.0 }));
    }

    #[test]
    fn meter_spent_is_measured_across_the_whole_combo() {
        let mut t = Tracker::with_progress(4);
        assert_eq!(feed(&mut t, 7, 300.0, [3000.0, 1900.0]), Some(4));
        assert!(t.last.unwrap().meter_spent >= SUPER_COST);
    }

    #[test]
    fn finishing_the_ladder_stops_asking_for_goals() {
        let mut t = Tracker::with_progress(99);
        assert_eq!(t.cleared, LADDER.len());
        assert_eq!(t.goal(), None);
        assert_eq!(feed(&mut t, 30, 900.0, [0.0, 0.0]), None);
    }

    #[test]
    fn non_finite_samples_are_ignored() {
        let mut t = Tracker::default();
        assert_eq!(t.observe(Sample { hits: 5, damage: f32::NAN, life_max: 1000.0, power: 0.0 }), None);
        assert_eq!(t.live_hits(), 0);
    }
}
