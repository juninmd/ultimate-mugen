use std::cell::Cell;

thread_local! {
    static SCALE: Cell<f32> = const { Cell::new(1.0) };
}

pub fn with<T>(scale: f32, draw: impl FnOnce() -> T) -> T {
    SCALE.with(|cell| {
        let previous = cell.replace(scale.clamp(0.0, 1.0));
        let out = draw();
        cell.set(previous);
        out
    })
}

pub(crate) fn apply(mut rgba: [f32; 4]) -> [f32; 4] {
    rgba[3] *= SCALE.with(Cell::get);
    rgba
}
