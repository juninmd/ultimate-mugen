use std::cell::RefCell;
use std::collections::HashMap;

const STIFFNESS: f32 = 260.0;
const DAMPING: f32 = 26.0;
const MAX_STEP: f32 = 0.05;

thread_local! {
    static SPRINGS: RefCell<HashMap<&'static str, (f32, f32, f64)>> = RefCell::new(HashMap::new());
}

pub fn follow(key: &'static str, target: f32) -> f32 {
    let now = ume_shell::quad::get_time();
    SPRINGS.with(|springs| {
        let mut springs = springs.borrow_mut();
        let (pos, vel, last) = springs.entry(key).or_insert((target, 0.0, now));
        let dt = ((now - *last) as f32).clamp(0.0, MAX_STEP);
        *last = now;
        let accel = STIFFNESS * (target - *pos) - DAMPING * *vel;
        *vel += accel * dt;
        *pos += *vel * dt;
        *pos
    })
}
