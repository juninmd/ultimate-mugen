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

    pub(crate) fn story_ally_preview(&mut self) {
        if let Some(run) = self.story.as_mut() {
            run.helper = crate::story::HelperPick::Chosen("shion".into());
        }
    }

    pub(crate) fn story_helper_preview(&mut self) {
        let Some(run) = self.story.as_mut() else { return };
        run.stage = crate::story::STAGES.len() - 1;
        run.wave = run.stage_wave_count();
        run.phase = Phase::Intro;
        run.mercy = Some(false);
        run.prefight = crate::story::prefight::PAGES;
        self.story_prepare_fight();
    }

    pub(crate) fn story_prefight_preview(&mut self) {
        let Some(run) = self.story.as_mut() else { return };
        run.stage = crate::story::STAGES.len() - 1;
        run.wave = run.stage_wave_count();
        run.phase = Phase::Intro;
        self.story_prepare_fight();
    }
}
