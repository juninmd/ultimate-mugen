use crate::app::App;

impl App {
    pub(crate) fn story_snapshot(&self) -> Option<(usize, usize, u8, i64, String)> {
        let run = self.story.as_ref()?;
        Some((run.stage, run.wave, run.credits, run.score, format!("{:?}", run.phase)))
    }

    pub(crate) fn story_decision_snapshot(&self) -> Option<(Option<bool>, bool, usize, bool)> {
        let run = self.story.as_ref()?;
        Some((
            run.mercy,
            crate::story::helper::pending(run),
            run.helper_cursor,
            matches!(&run.helper, crate::story::HelperPick::Chosen(_)),
        ))
    }
}
