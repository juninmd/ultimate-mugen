use super::*;

thread_local! {
    static TRANSIENT_GENERATION: Cell<u32> = const { Cell::new(0) };
    static TRANSIENT_FULL: Cell<bool> = const { Cell::new(false) };
}

pub fn generation(transient: bool) -> u32 {
    if transient { TRANSIENT_GENERATION.with(|g| g.get()) } else { super::generation() }
}

pub fn transient_resets() -> u32 { TRANSIENT_GENERATION.with(|g| g.get()) }

pub fn bump() { TRANSIENT_GENERATION.with(|g| g.set(g.get().wrapping_add(1))); TRANSIENT_FULL.with(|a| a.set(false)); }

pub fn reset_if_full(f: &mut Frame) {
    if !TRANSIENT_FULL.with(|a| a.get()) { return; }
    f.gpu.rgba_atlas.clear_transient();
    bump();
}

pub fn alloc(f: &mut Frame, inner: &TexInner) -> Option<ume_render::Rect> {
    if inner.transient {
        let found = f.gpu.rgba_atlas.alloc_transient(inner.w, inner.h);
        if found.is_none() { TRANSIENT_FULL.with(|a| a.set(true)); }
        return found;
    }
    let found = f.gpu.rgba_atlas.alloc(inner.w, inner.h);
    if found.is_none() {
        if f.gpu.rgba_atlas.transient_used() > 0 { TRANSIENT_FULL.with(|a| a.set(true)); } else { ATLAS_FULL.with(|a| a.set(true)); }
    }
    found
}

impl Texture2D {
    /// Art that can be uploaded again at any time (fighter frames): it is evicted on its own without a full atlas reset.
    pub fn from_rgba_transient(w: u16, h: u16, rgba: Arc<Vec<u8>>) -> Texture2D {
        let base = Texture2D::from_rgba_shared(w, h, rgba);
        let inner = &base.0;
        Texture2D(Rc::new(TexInner { slot: Cell::new(None), pixels: inner.pixels.clone(), w: inner.w, h: inner.h, transient: true }))
    }
}
