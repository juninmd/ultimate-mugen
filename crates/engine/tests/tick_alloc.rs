use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_zip, find_char_defs, Vfs};

struct Counting;
thread_local! { static ALLOCS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
fn count() {
    ALLOCS.with(|c| c.set(c.get() + 1));
}
fn allocs() -> usize {
    ALLOCS.with(|c| c.get())
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        count();
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        count();
        System.realloc(p, l, n)
    }
}
#[global_allocator]
static A: Counting = Counting;

#[test]
fn tick_allocations() {
    let mut v = Vfs::new();
    add_zip(
        &mut v,
        include_bytes!("../../../assets/mugen/elecbyte.zip"),
        "",
    )
    .unwrap();
    let def = find_char_defs(&v).into_iter().next().unwrap();
    let d = Arc::new(load_mugen_char(&v, &def, 1, false).unwrap());
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 8, 1);
    m.add_player(d, 1, 8, 2);
    m.start_round();
    for _ in 0..200 {
        m.tick(&[0, 0]);
    }
    let before = allocs();
    let n = 600;
    for _ in 0..n {
        m.tick(&[0, 0]);
    }
    let per = (allocs() - before) as f64 / n as f64;
    println!("allocations per tick: {per:.2}");
    assert!(per <= 0.5, "match tick allocates {per:.2} times per tick");
}

#[test]
fn ten_fighters_tick_time() {
    let mut v = Vfs::new();
    add_zip(
        &mut v,
        include_bytes!("../../../assets/mugen/elecbyte.zip"),
        "",
    )
    .unwrap();
    let def = find_char_defs(&v).into_iter().next().unwrap();
    let d = Arc::new(load_mugen_char(&v, &def, 1, false).unwrap());
    let mut m = Match::new(common_def(), None, flat_stage());
    m.team_simul = true;
    for k in 0..10 {
        m.add_player(d.clone(), k % 2, 8, 1 + k as i32 % 4);
    }
    m.start_round();
    let inputs = [0u32; 10];
    for _ in 0..200 {
        m.tick(&inputs);
    }
    let n = 1000;
    let t = std::time::Instant::now();
    for _ in 0..n {
        m.tick(&inputs);
    }
    let us = t.elapsed().as_secs_f64() * 1e6 / n as f64;
    println!("10-fighter tick: {us:.1} us");
    if !cfg!(debug_assertions) {
        assert!(us < 1000.0, "10-fighter tick took {us:.1} us");
    }
}
