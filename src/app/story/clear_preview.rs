use crate::app::App;
use crate::story::{Phase, StageTally};

impl App {
    pub(crate) fn story_clear_preview(&mut self) {
        let Some(run) = self.story.as_mut() else { return };
        run.stage = 1;
        run.score = 12000;
        run.phase = Phase::Cleared;
        run.tally = Some(StageTally::new(0, 2000, 0.8));
    }
}
