use super::*;

/// A stage being loaded off the render thread (MUGEN stages decode a whole SFF).
pub struct StageJob {
    pub(super) index: usize,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) thread: Option<std::thread::JoinHandle<Result<StageBuild, String>>>,
    #[cfg(target_arch = "wasm32")]
    pub(super) future: Option<std::pin::Pin<Box<dyn std::future::Future<Output = StageBuild>>>>,
    pub(super) immediate: Option<StageBuild>,
}

pub struct StageBuild {
    pub stage: Stage,
    pub preview: Option<Arc<ume_engine::chardef::ImageData>>,
}

impl StageJob {
    pub(super) fn ready(index: usize, build: StageBuild) -> Self {
        StageJob {
            index,
            immediate: Some(build),
            #[cfg(not(target_arch = "wasm32"))]
            thread: None,
            #[cfg(target_arch = "wasm32")]
            future: None,
        }
    }
    pub fn index(&self) -> usize {
        self.index
    }
    /// True once the stage is built; polls the browser job, never blocks.
    pub fn ready_now(&mut self) -> bool {
        if self.immediate.is_some() {
            return true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.thread.as_ref().is_some_and(|h| h.is_finished())
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(fut) = self.future.as_mut() {
                let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
                if let std::task::Poll::Ready(build) = fut.as_mut().poll(&mut cx) {
                    self.immediate = Some(build);
                    self.future = None;
                }
            }
            self.immediate.is_some()
        }
    }
    /// Collects the result; call only after [`Self::ready_now`] returned true.
    pub fn take(&mut self) -> Result<StageBuild, String> {
        if let Some(b) = self.immediate.take() {
            return Ok(b);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            join_stage_thread(self.thread.take())
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err("stage job is still pending".into())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn join_stage_thread(
    thread: Option<std::thread::JoinHandle<Result<StageBuild, String>>>,
) -> Result<StageBuild, String> {
    let h = thread.ok_or("stage job already collected")?;
    h.join()
        .unwrap_or_else(|_| Err("stage loader thread panicked".into()))
}
