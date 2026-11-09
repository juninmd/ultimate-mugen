use super::stages::{StageDef, STAGES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Intro,
    Cleared,
    Defeated,
    Victory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    Wave,
    StageCleared,
    Finished,
}

pub const STARTING_CREDITS: u8 = 3;
const WAVE_RECOVERY: f32 = 0.2;
const STAGE_RECOVERY: f32 = 0.6;
const MIN_CARRIED_LIFE: f32 = 0.08;
const CONTINUE_LIFE: f32 = 0.6;
const WAVE_BONUS: i64 = 500;
const BOSS_BONUS_FACTOR: i64 = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct StoryRun {
    pub stage: usize,
    pub wave: usize,
    pub score: i64,
    pub life: f32,
    pub credits: u8,
    pub phase: Phase,
}

impl Default for StoryRun {
    fn default() -> Self {
        StoryRun {
            stage: 0,
            wave: 0,
            score: 0,
            life: 1.0,
            credits: STARTING_CREDITS,
            phase: Phase::Intro,
        }
    }
}

impl StoryRun {
    pub fn stage_def(&self) -> &'static StageDef {
        &STAGES[self.stage.min(STAGES.len() - 1)]
    }

    pub fn is_boss_fight(&self) -> bool {
        self.wave >= self.stage_def().waves.len()
    }

    pub fn foes(&self, player_id: &str) -> Vec<&'static str> {
        let def = self.stage_def();
        if self.is_boss_fight() {
            let boss = if def.boss == player_id { def.alt_boss } else { def.boss };
            return vec![boss];
        }
        def.waves[self.wave].to_vec()
    }

    pub fn ai_level(&self, base: i32) -> i32 {
        let boss_push = if self.is_boss_fight() { 2 } else { 0 };
        (1 + self.stage as i32 + boss_push + base / 4).clamp(1, 8)
    }

    pub fn fight_index(&self) -> usize {
        self.wave.min(self.stage_def().waves.len())
    }

    pub fn won_fight(&mut self, fight_score: i64, remaining_life: f32) -> Next {
        self.life = remaining_life.clamp(MIN_CARRIED_LIFE, 1.0);
        let bonus = WAVE_BONUS * (self.stage as i64 + 1);
        let boss_fight = self.is_boss_fight();
        self.score += fight_score.max(0) + if boss_fight { bonus * BOSS_BONUS_FACTOR } else { bonus };
        if !boss_fight {
            self.wave += 1;
            self.recover(WAVE_RECOVERY);
            return Next::Wave;
        }
        self.wave = 0;
        self.stage += 1;
        self.recover(STAGE_RECOVERY);
        if self.stage >= STAGES.len() {
            self.stage = STAGES.len() - 1;
            self.phase = Phase::Victory;
            return Next::Finished;
        }
        self.phase = Phase::Cleared;
        Next::StageCleared
    }

    pub fn lost_fight(&mut self, fight_score: i64) {
        self.score += fight_score.max(0);
        self.phase = Phase::Defeated;
    }

    pub fn can_continue(&self) -> bool {
        self.credits > 0
    }

    pub fn continue_fight(&mut self) {
        self.credits = self.credits.saturating_sub(1);
        self.life = CONTINUE_LIFE;
        self.phase = Phase::Intro;
    }

    fn recover(&mut self, ratio: f32) {
        self.life = (self.life + ratio).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stage_is_its_waves_then_one_boss() {
        let mut run = StoryRun::default();
        let waves = run.stage_def().waves.len();
        for _ in 0..waves {
            assert!(!run.is_boss_fight());
            assert_eq!(run.won_fight(0, 1.0), Next::Wave);
        }
        assert!(run.is_boss_fight());
        assert_eq!(run.foes("kaito"), vec!["bruno"]);
        assert_eq!(run.won_fight(0, 1.0), Next::StageCleared);
        assert_eq!((run.stage, run.wave), (1, 0));
    }

    #[test]
    fn playing_as_the_boss_swaps_in_the_alternate() {
        let mut run = StoryRun::default();
        run.wave = run.stage_def().waves.len();
        assert_eq!(run.foes("bruno"), vec!["daichi"]);
    }

    #[test]
    fn the_last_boss_ends_the_story() {
        let mut run = StoryRun::default();
        run.stage = STAGES.len() - 1;
        run.wave = run.stage_def().waves.len();
        assert_eq!(run.won_fight(0, 1.0), Next::Finished);
        assert_eq!(run.phase, Phase::Victory);
    }

    #[test]
    fn continuing_spends_a_credit_and_restores_part_of_the_bar() {
        let mut run = StoryRun::default();
        run.life = 0.0;
        run.lost_fight(120);
        assert_eq!(run.phase, Phase::Defeated);
        run.continue_fight();
        assert_eq!(run.credits, STARTING_CREDITS - 1);
        assert!((run.life - CONTINUE_LIFE).abs() < 1e-6);
    }

    #[test]
    fn health_carries_over_and_recovers_a_little() {
        let mut run = StoryRun::default();
        run.won_fight(0, 0.3);
        assert!((run.life - (0.3 + WAVE_RECOVERY)).abs() < 1e-6);
        run.won_fight(0, 0.0);
        assert!(run.life >= MIN_CARRIED_LIFE);
    }

    #[test]
    fn longer_stages_have_a_strip_segment_per_fight() {
        for stage in STAGES.iter() {
            assert!(stage.waves.len() >= 4);
            let mid = super::super::stages::fight_center(stage, super::super::stages::fights_in(stage) / 2);
            assert!(mid.abs() <= super::super::stages::SEGMENT_WORLD);
        }
    }

    #[test]
    fn credits_run_out() {
        let mut run = StoryRun::default();
        for _ in 0..STARTING_CREDITS {
            run.continue_fight();
        }
        assert!(!run.can_continue());
    }

    #[test]
    fn difficulty_rises_with_stage_and_boss() {
        let mut run = StoryRun::default();
        let first = run.ai_level(4);
        run.stage = 3;
        assert!(run.ai_level(4) > first);
        run.wave = run.stage_def().waves.len();
        assert!(run.ai_level(4) <= 8);
    }

    #[test]
    fn every_stage_has_non_empty_waves_and_a_distinct_alternate_boss() {
        for stage in STAGES.iter() {
            assert!(stage.waves.iter().all(|w| !w.is_empty() && w.len() <= 4));
            assert_ne!(stage.boss, stage.alt_boss);
        }
    }

    #[test]
    fn wave_villains_have_around_ten_percent_of_protagonist_life_and_advance_right_to_boss() {
        for stage in STAGES.iter() {
            for wave in stage.waves {
                for &id in *wave {
                    let ratio = super::super::stages::villain_life_ratio(id);
                    assert!((0.08..=0.12).contains(&ratio), "{id} ratio {ratio} must be ~10%");
                }
            }
            let total = super::super::stages::fights_in(stage);
            for i in 1..total {
                let prev = super::super::stages::fight_center(stage, i - 1);
                let next = super::super::stages::fight_center(stage, i);
                assert!(next > prev, "fight {i} must advance right ({prev} -> {next})");
            }
        }
    }
}

#[cfg(test)]
mod strip_tests {
    use super::*;

    #[test]
    fn every_arena_strip_has_one_segment_per_fight() {
        for stage in STAGES.iter() {
            let strip = crate::stages::story_strip(stage.arena).expect("strip");
            assert_eq!(strip.w as usize, 836 * crate::story::fights_in(stage));
            assert_eq!(strip.h, 470);
        }
    }
}
