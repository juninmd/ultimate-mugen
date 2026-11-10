use super::{Pilot, Session};
use ume_engine::input::k;
use ume_engine::matchs::Match;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DummyPlan {
    #[default]
    Stand,
    Crouch,
    Jump,
    Guard,
    CrouchGuard,
}

impl DummyPlan {
    pub const ALL: [DummyPlan; 5] = [
        DummyPlan::Stand,
        DummyPlan::Crouch,
        DummyPlan::Jump,
        DummyPlan::Guard,
        DummyPlan::CrouchGuard,
    ];

    pub fn next(self) -> DummyPlan {
        let at = DummyPlan::ALL.iter().position(|p| *p == self).unwrap_or(0);
        DummyPlan::ALL[(at + 1) % DummyPlan::ALL.len()]
    }

    pub fn label_key(self) -> &'static str {
        match self {
            DummyPlan::Stand => "dummy.stand",
            DummyPlan::Crouch => "dummy.crouch",
            DummyPlan::Jump => "dummy.jump",
            DummyPlan::Guard => "dummy.guard",
            DummyPlan::CrouchGuard => "dummy.crouch_guard",
        }
    }

    pub fn held_bits(self, facing: f32, airborne: bool, threatened: bool) -> u32 {
        let away = if facing >= 0.0 { k::L } else { k::R };
        let block = if threatened { away } else { 0 };
        match self {
            DummyPlan::Stand => 0,
            DummyPlan::Crouch => k::D,
            DummyPlan::Jump => {
                if airborne {
                    0
                } else {
                    k::U
                }
            }
            DummyPlan::Guard => block,
            DummyPlan::CrouchGuard => k::D | block,
        }
    }

    pub fn bits(self, m: &Match, fighter: usize) -> u32 {
        let me = &m.fighters[fighter];
        let attacked = m
            .fighters
            .iter()
            .any(|f| f.side != me.side && f.alive && f.move_type == 'A');
        let threatened = attacked || !m.projectiles.is_empty();
        self.held_bits(me.facing, me.state_type == 'A', threatened)
    }
}

impl Session {
    pub fn dummy_plan(&self) -> Option<DummyPlan> {
        self.pilots.iter().find_map(|p| match p {
            Pilot::Dummy(plan) => Some(*plan),
            _ => None,
        })
    }

    pub fn set_dummy_plan(&mut self, plan: DummyPlan) {
        for pilot in &mut self.pilots {
            if let Pilot::Dummy(current) = pilot {
                *current = plan;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycling_visits_every_plan_and_wraps() {
        let mut plan = DummyPlan::default();
        let mut seen = vec![plan];
        for _ in 1..DummyPlan::ALL.len() {
            plan = plan.next();
            seen.push(plan);
        }
        assert_eq!(seen, DummyPlan::ALL);
        assert_eq!(plan.next(), DummyPlan::Stand);
    }

    #[test]
    fn guard_holds_away_from_the_rival_only_while_threatened() {
        assert_eq!(DummyPlan::Guard.held_bits(1.0, false, false), 0);
        assert_eq!(DummyPlan::Guard.held_bits(1.0, false, true), k::L);
        assert_eq!(DummyPlan::Guard.held_bits(-1.0, false, true), k::R);
        assert_eq!(
            DummyPlan::CrouchGuard.held_bits(-1.0, false, true),
            k::D | k::R
        );
        assert_eq!(DummyPlan::CrouchGuard.held_bits(1.0, false, false), k::D);
    }

    #[test]
    fn jump_releases_up_in_the_air_so_every_landing_jumps_again() {
        assert_eq!(DummyPlan::Jump.held_bits(1.0, false, false), k::U);
        assert_eq!(DummyPlan::Jump.held_bits(1.0, true, false), 0);
        assert_eq!(DummyPlan::Stand.held_bits(1.0, false, true), 0);
        assert_eq!(DummyPlan::Crouch.held_bits(1.0, false, true), k::D);
    }

    #[test]
    fn every_plan_has_a_translated_label() {
        for plan in DummyPlan::ALL {
            assert_ne!(
                crate::i18n::t(plan.label_key()),
                plan.label_key(),
                "{plan:?}"
            );
        }
    }
}
