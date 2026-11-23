use super::*;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

const MAX_PORTRAIT_WORKERS: usize = 8;
static PORTRAIT_WORKERS: AtomicUsize = AtomicUsize::new(0);

pub(super) fn try_claim_portrait_worker() -> bool {
    PORTRAIT_WORKERS
        .fetch_update(Relaxed, Relaxed, |n| (n < MAX_PORTRAIT_WORKERS).then_some(n + 1))
        .is_ok()
}

pub(super) fn release_portrait_worker() {
    let _ = PORTRAIT_WORKERS.fetch_update(Relaxed, Relaxed, |n| Some(n.saturating_sub(1)));
}

pub(super) fn spawn_portrait_worker(vfs: Arc<Vfs>, def: String) -> Option<std::thread::JoinHandle<Option<Sprite>>> {
    if !try_claim_portrait_worker() {
        return None;
    }
    let res = std::thread::Builder::new().name("char-portrait".into()).spawn(move || {
        let out = load_mugen_portrait(&vfs, &def);
        release_portrait_worker();
        out
    });
    if res.is_err() {
        release_portrait_worker();
    }
    res.ok()
}

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

