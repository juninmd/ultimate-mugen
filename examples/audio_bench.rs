#[allow(dead_code)]
#[path = "../src/sound_mix.rs"]
mod sound_mix;
use std::{alloc::{GlobalAlloc, Layout, System}, hint::black_box, sync::atomic::{AtomicUsize, Ordering}, time::Instant};
struct Counting;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 { ALLOCS.fetch_add(1, Ordering::Relaxed); System.alloc(l) }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { System.dealloc(p, l) }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 { ALLOCS.fetch_add(1, Ordering::Relaxed); System.realloc(p, l, n) }
}
#[global_allocator]
static A: Counting = Counting;
fn main() {
    let frames = (0..44100 * 30).map(|i| [(i as f32 * 0.05).sin() * 0.5; 2]).collect();
    let wav = sound_mix::Pcm { rate: 44100, frames }.to_wav();
    for _ in 0..5 {
        ALLOCS.store(0, Ordering::Relaxed);
        let start = Instant::now();
        let decoded = sound_mix::decode(black_box(&wav)).unwrap();
        let elapsed = start.elapsed();
        let allocs = ALLOCS.load(Ordering::Relaxed);
        assert_eq!(decoded.frames.len(), 44100 * 30);
        println!("30s stereo WAV: {:.2} ms, {allocs} allocations", elapsed.as_secs_f64() * 1000.0);
        black_box(decoded);
    }
}
