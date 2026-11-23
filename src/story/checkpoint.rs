use super::{Next, StoryRun};
use serde::{Deserialize, Serialize};

const FILE: &str = "story-checkpoint.json";
const VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
pub(crate) struct Checkpoint {
    version: u8,
    campaign: String,
    pub fighter: String,
    pub palette: i32,
    stage: usize,
    arena: String,
    score: i64,
    life: f32,
    credits: u8,
}

impl Checkpoint {
    pub(crate) fn save(run: &StoryRun, fighter: &str, palette: i32) {
        let checkpoint = Self {
            version: VERSION,
            campaign: run.script().campaign.id.clone(),
            fighter: fighter.into(),
            palette,
            stage: run.stage,
            arena: run.stage_arena().into(),
            score: run.score,
            life: run.life,
            credits: run.credits,
        };
        if let Ok(json) = serde_json::to_string(&checkpoint) {
            crate::storage::write(FILE, &json);
        }
    }

    pub(crate) fn load() -> Option<Self> {
        let checkpoint: Self = serde_json::from_str(&crate::storage::read(FILE)?).ok()?;
        checkpoint.restore()?;
        Some(checkpoint)
    }

    pub(crate) fn restore(&self) -> Option<StoryRun> {
        let run = StoryRun {
            stage: self.stage,
            score: self.score,
            life: self.life,
            credits: self.credits,
            ..StoryRun::default()
        };
        (self.version == VERSION
            && self.campaign == run.script().campaign.id
            && !self.fighter.is_empty()
            && (1..=crate::settings::PALETTE_COUNT as i32).contains(&self.palette)
            && (1..run.stage_count()).contains(&run.stage)
            && run.stage_arena() == self.arena
            && (0.0..=1.0).contains(&run.life)
            && run.life > 0.0
            && run.score >= 0
            && run.credits <= run.script().campaign.credits)
            .then_some(run)
    }

    pub(crate) fn clear() {
        crate::storage::write(FILE, "");
    }
}

pub(crate) fn record(next: Next, run: &StoryRun, fighter: &str, palette: i32) {
    match next {
        Next::StageCleared => Checkpoint::save(run, fighter, palette),
        Next::Finished => Checkpoint::clear(),
        Next::Wave => {}
    }
}
