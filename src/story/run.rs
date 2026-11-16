use super::stages::{StageDef, STAGES};
use super::tally::StageTally;

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
    pub bonus: bool,
    pub epilogue: usize,
    pub mercy: Option<bool>,
    pub tally: Option<StageTally>,
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
            bonus: false,
            epilogue: 0,
            mercy: None,
            tally: None,
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
        let tally = StageTally::new(self.stage, bonus * BOSS_BONUS_FACTOR, remaining_life);
        self.score += tally.life;
        self.tally = Some(tally);
        self.wave = 0;
        self.bonus = false;
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
#[path = "run_tests.rs"]
mod run_tests;
