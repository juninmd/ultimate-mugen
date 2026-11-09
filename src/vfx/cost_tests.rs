use super::{quality, Vfx};
use std::time::Instant;

#[test]
#[ignore]
fn simulation_side_cost_per_frame() {
    for level in [0usize, 1, 2] {
        quality::set(level);
        let mut v = Vfx::new();
        let frames = 20000;
        let start = Instant::now();
        for i in 0..frames {
            if i % 20 == 0 {
                v.spark_burst(0.0, 0.0, 1.0, quality::count(26), 9.0, [1.0, 0.5, 0.2], 1.0);
                v.cracks(0.0, 30.0, 1.0, 46.0);
                v.ring(0.0, 0.0, 58.0, 0.24, 1.4, [1.0, 0.5, 0.2]);
            }
            v.update(1.0 / 60.0);
        }
        let per_frame = start.elapsed().as_secs_f64() * 1e6 / frames as f64;
        println!("quality {level}: {per_frame:.2} us per frame (hit every 20 frames)");
    }
    quality::set(2);
}
