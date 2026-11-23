#[derive(Clone, Debug)]
pub(crate) struct ArcadeRun {
    pub(crate) ladder: Vec<usize>,
    pub(crate) index: usize,
    route_stages: Option<Vec<Option<usize>>>,
}

impl ArcadeRun {
    pub(crate) fn new(ladder: Vec<usize>, route_stages: Option<Vec<Option<usize>>>) -> Self {
        Self {
            ladder,
            index: 0,
            route_stages,
        }
    }

    pub(crate) fn at(ladder: Vec<usize>, index: usize) -> Self {
        Self {
            ladder,
            index,
            route_stages: None,
        }
    }

    pub(crate) fn stage_override(&self, fight: usize) -> Option<usize> {
        self.route_stages
            .as_ref()
            .and_then(|stages| stages.get(fight))
            .copied()
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::ArcadeRun;

    #[test]
    fn stage_overrides_follow_their_planned_fight() {
        let mut run = ArcadeRun::new(vec![1, 2, 3], Some(vec![Some(4), None, Some(2)]));
        assert_eq!(run.stage_override(0), Some(4));
        run.index = 2;
        assert_eq!(run.stage_override(run.index), Some(2));
        assert_eq!(run.stage_override(3), None);
    }
}
