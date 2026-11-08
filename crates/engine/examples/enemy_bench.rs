use std::{hint::black_box, sync::Arc, time::Instant};
use ume_engine::{chardef::CharDef, common::{common_def, flat_stage}, matchs::Match};
fn main() {
    let mut m = Match::new(common_def(), None, flat_stage());
    let d = Arc::new(CharDef::empty("bench"));
    for i in 0..10 { let n = m.add_player(d.clone(), i % 2, 0, 1); m.fighters[n].pos.x = (i as f32 * 97.0) % 317.0; }
    let count = 10_000_000;
    let start = Instant::now();
    for i in 0..count { black_box(m.enemy(black_box(i % 10), 0)); }
    println!("10 fighters: {:.2} ns per nearest-enemy lookup", start.elapsed().as_nanos() as f64 / count as f64);
}
