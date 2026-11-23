use std::cell::Cell;

thread_local! { static PINNED: Cell<Option<f64>> = const { Cell::new(None) }; }

/// Art pixels per game unit: the value pinned by `with_res` on this thread, else the global setting.
pub(super) fn res() -> f64 {
    PINNED.with(Cell::get).unwrap_or_else(|| super::effects::art_res() as f64)
}

/// Runs `draw` at the resolution an effect was sized for, even if the quality setting changed since.
pub(super) fn with_res<T>(res: f64, draw: impl FnOnce() -> T) -> T {
    let before = PINNED.with(|pinned| pinned.replace(Some(res)));
    let out = draw();
    PINNED.with(|pinned| pinned.set(before));
    out
}
