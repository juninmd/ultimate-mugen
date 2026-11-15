use std::cell::Cell;
use ume_render::Waves;

thread_local! {
    static PENDING: Cell<Waves> = const { Cell::new(Waves { n: 0, items: [[0.0; 8]; ume_render::MAX_WAVES] }) };
}

pub fn shockwave(x: f32, y: f32, radius: f32, width: f32, strength: f32) -> bool {
    let mut w = PENDING.get();
    let ok = w.push(x, y, radius, width, strength);
    PENDING.set(w);
    ok
}

pub fn take() -> Waves {
    PENDING.replace(Waves::default())
}
