pub(crate) struct ArcadeRun {
    pub(crate) ladder: Vec<usize>,
    pub(crate) index: usize,
    route_stages: Option<Vec<Option<usize>>>,
    route_storyboards: Option<Vec<Vec<String>>>,
    storyboard_cursors: Option<Vec<usize>>,
    pub(crate) storyboard_player: Option<super::storyboard::Playback>,
    pub(crate) storyboard_after: Option<super::storyboard::After>,
}

impl ArcadeRun {
    pub(crate) fn new(
        ladder: Vec<usize>,
        route_stages: Option<Vec<Option<usize>>>,
        route_storyboards: Option<Vec<Vec<String>>>,
    ) -> Self {
        Self {
            ladder,
            index: 0,
            route_stages,
            storyboard_cursors: route_storyboards
                .as_ref()
                .map(|slots| vec![0; slots.len()]),
            route_storyboards,
            storyboard_player: None,
            storyboard_after: None,
        }
    }

    pub(crate) fn at(ladder: Vec<usize>, index: usize) -> Self {
        Self {
            ladder,
            index,
            route_stages: None,
            route_storyboards: None,
            storyboard_cursors: None,
            storyboard_player: None,
            storyboard_after: None,
        }
    }

    pub(crate) fn stage_override(&self, fight: usize) -> Option<usize> {
        self.route_stages
            .as_ref()
            .and_then(|stages| stages.get(fight))
            .copied()
            .flatten()
    }

    pub(crate) fn take_storyboard(&mut self, slot: usize) -> Option<String> {
        let path = self.route_storyboards.as_ref()?.get(slot)?.get(
            *self.storyboard_cursors.as_ref()?.get(slot)?,
        )?.clone();
        *self.storyboard_cursors.as_mut()?.get_mut(slot)? += 1;
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::ArcadeRun;

    #[test]
    fn stage_overrides_follow_their_planned_fight() {
        let mut run = ArcadeRun::new(
            vec![1, 2, 3],
            Some(vec![Some(4), None, Some(2)]),
            Some(vec![vec!["opening.def".into(), "rival.def".into()], vec![], vec!["ending.def".into()]]),
        );
        assert_eq!(run.stage_override(0), Some(4));
        run.index = 2;
        assert_eq!(run.stage_override(run.index), Some(2));
        assert_eq!(run.stage_override(3), None);
        assert_eq!(run.take_storyboard(0).as_deref(), Some("opening.def"));
        assert_eq!(run.take_storyboard(0).as_deref(), Some("rival.def"));
        assert_eq!(run.take_storyboard(0), None);
        assert_eq!(run.take_storyboard(2).as_deref(), Some("ending.def"));
    }
}
