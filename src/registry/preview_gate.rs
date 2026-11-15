use super::*;

impl Registry {
    pub fn stage_preview_started(&self, i: usize) -> bool {
        self.stages.get(i).is_some_and(|e| {
            matches!(e.src, StageSrc::Builtin(_))
                || !matches!(*e.preview.borrow(), Preview::Unknown)
        })
    }

    pub fn trim_stage_previews(&self, keep: impl Fn(usize) -> bool) -> usize {
        let mut freed = 0;
        for (i, entry) in self.stages.iter().enumerate() {
            if keep(i) || !matches!(entry.src, StageSrc::Mugen { .. }) {
                continue;
            }
            if matches!(*entry.preview.borrow(), Preview::Ready(_)) {
                *entry.preview.borrow_mut() = Preview::Unknown;
                entry.stage.borrow_mut().take();
                freed += 1;
            }
        }
        freed
    }
}
