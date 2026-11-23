use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

pub(super) static IMG_ID: AtomicU64 = AtomicU64::new(1);

type PixelSource = Box<dyn Fn() -> Arc<Vec<u8>> + Send + Sync + std::panic::RefUnwindSafe>;

/// RGBA image owned by the engine (uploaded to a GPU texture by the renderer, cached by `id`).
pub struct ImageData {
    pub w: u32,
    pub h: u32,
    pixels: OnceLock<Arc<Vec<u8>>>,
    source: Option<PixelSource>,
    pub id: u64,
}

impl ImageData {
    pub fn new(w: u32, h: u32, rgba: impl Into<Arc<Vec<u8>>>) -> Arc<Self> {
        let pixels = OnceLock::new();
        let _ = pixels.set(rgba.into());
        Arc::new(ImageData { w, h, pixels, source: None, id: IMG_ID.fetch_add(1, Ordering::Relaxed) })
    }

    /// Pixels are produced on first access, so art that may never be drawn costs nothing until it is.
    pub fn lazy(w: u32, h: u32, source: impl Fn() -> Arc<Vec<u8>> + Send + Sync + std::panic::RefUnwindSafe + 'static) -> Arc<Self> {
        Arc::new(ImageData { w, h, pixels: OnceLock::new(), source: Some(Box::new(source)), id: IMG_ID.fetch_add(1, Ordering::Relaxed) })
    }

    pub fn rgba(&self) -> &Arc<Vec<u8>> {
        self.pixels.get_or_init(|| self.source.as_ref().map_or_else(Default::default, |make| make()))
    }

    pub fn is_pending(&self) -> bool {
        self.pixels.get().is_none()
    }
}

impl std::fmt::Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageData").field("w", &self.w).field("h", &self.h).field("id", &self.id).field("pending", &self.is_pending()).finish()
    }
}
