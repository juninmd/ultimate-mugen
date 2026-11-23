use super::helper::HelperPick;
use super::script::{self, ScriptStage, StoryScript};
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
    pub helper: HelperPick,
    pub helper_cursor: usize,
    pub tally: Option<StageTally>,
}

impl Default for StoryRun {
    fn default() -> Self {
        let credits = script::active().campaign.credits;
        StoryRun {
            stage: 0,
            wave: 0,
            score: 0,
            life: 1.0,
            credits,
            phase: Phase::Intro,
            bonus: false,
            epilogue: 0,
            mercy: None,
            helper: HelperPick::Pending,
            helper_cursor: 0,
            tally: None,
        }
    }
}

impl StoryRun {
    pub fn script(&self) -> &'static StoryScript {
        script::active()
    }

    pub fn script_stage(&self) -> Option<&'static ScriptStage> {
        self.script().stage(self.stage)
    }

    pub fn stage_count(&self) -> usize {
        self.script().stages.len().max(1)
    }

    pub fn stage_def(&self) -> &'static StageDef {
        &STAGES[self.stage.min(STAGES.len() - 1)]
    }

    pub fn stage_wave_count(&self) -> usize {
        self.script_stage()
            .map(|s| s.waves.len())
            .unwrap_or_else(|| self.stage_def().waves.len())
    }

    pub fn stage_arena(&self) -> &'static str {
        self.script_stage()
            .map(|s| s.arena.as_str())
            .unwrap_or(self.stage_def().arena)
    }

    pub fn stage_accent(&self) -> [f32; 3] {
        self.script_stage()
            .map(|s| s.accent)
            .unwrap_or(self.stage_def().accent)
    }

    pub fn stage_lanes(&self) -> (f32, f32) {
        self.script_stage()
            .map(|s| (s.lane_far, s.lane_near))
            .unwrap_or((self.stage_def().lane_far, self.stage_def().lane_near))
    }

    pub fn stage_boss_id(&self, player_id: &str) -> &'static str {
        if let Some(st) = self.script_stage() {
            if st.boss == player_id {
                st.alt_boss.as_str()
            } else {
                st.boss.as_str()
            }
        } else {
            let def = self.stage_def();
            if def.boss == player_id { def.alt_boss } else { def.boss }
        }
    }

    pub fn is_boss_fight(&self) -> bool {
        self.wave >= self.stage_wave_count()
    }

    pub fn foes(&self, player_id: &str) -> Vec<&'static str> {
        if self.is_boss_fight() {
            return vec![self.stage_boss_id(player_id)];
        }
        if let Some(st) = self.script_stage() {
            if let Some(wave) = st.waves.get(self.wave) {
                return wave.iter().map(|s| s.as_str()).collect();
            }
        }
        self.stage_def().waves[self.wave].to_vec()
    }

    pub fn ai_level(&self, base: i32) -> i32 {
        let boss_push = if self.is_boss_fight() { 2 } else { 0 };
        (1 + self.stage as i32 + boss_push + base / 4).clamp(1, 8)
    }

    pub fn fight_index(&self) -> usize {
        self.wave.min(self.stage_wave_count())
    }

    pub fn won_fight(&mut self, fight_score: i64, remaining_life: f32) -> Next {
        let meta = &self.script().campaign;
        self.life = remaining_life.clamp(MIN_CARRIED_LIFE, 1.0);
        let bonus = WAVE_BONUS * (self.stage as i64 + 1);
        let boss_fight = self.is_boss_fight();
        self.score += fight_score.max(0) + if boss_fight { bonus * BOSS_BONUS_FACTOR } else { bonus };
        if !boss_fight {
            self.wave += 1;
            self.recover(meta.wave_recovery);
            return Next::Wave;
        }
        let tally = StageTally::new(self.stage, bonus * BOSS_BONUS_FACTOR, remaining_life);
        self.score += tally.life;
        self.tally = Some(tally);
        self.wave = 0;
        self.bonus = false;
        self.stage += 1;
        self.recover(meta.stage_recovery);
        let total = self.stage_count();
        if self.stage >= total {
            self.stage = total - 1;
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
