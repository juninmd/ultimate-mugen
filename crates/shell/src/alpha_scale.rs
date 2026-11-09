use std::cell::Cell;

thread_local! {
    static SCALE: Cell<f32> = const { Cell::new(1.0) };
}

struct Restore(f32);

impl Drop for Restore {
    fn drop(&mut self) {
        SCALE.with(|cell| cell.set(self.0));
    }
}

pub fn with<T>(scale: f32, draw: impl FnOnce() -> T) -> T {
    let previous = SCALE.with(|cell| cell.replace(scale.clamp(0.0, 1.0)));
    let _restore = Restore(previous);
    draw()
}

pub(crate) fn apply(mut rgba: [f32; 4]) -> [f32; 4] {
    rgba[3] *= SCALE.with(Cell::get);
    rgba
}
