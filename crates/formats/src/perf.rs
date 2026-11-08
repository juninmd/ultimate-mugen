//! Opt-in timing marks (`?perf=1` on the web build, `UME_TRACE=1` on desktop).
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static CLOCK: AtomicUsize = AtomicUsize::new(0);
static SINK: AtomicUsize = AtomicUsize::new(0);

pub fn enable(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool { ENABLED.load(Ordering::Relaxed) }

pub fn set_clock(f: fn() -> f64) { CLOCK.store(f as usize, Ordering::Relaxed); }

pub fn set_sink(f: fn(&str)) { SINK.store(f as usize, Ordering::Relaxed); }

pub fn now_ms() -> f64 {
    let f = CLOCK.load(Ordering::Relaxed);
    if f == 0 { return 0.0; }
    let f: fn() -> f64 = unsafe { std::mem::transmute(f) };
    f()
}

pub fn emit(msg: &str) {
    let f = SINK.load(Ordering::Relaxed);
    if f == 0 { return; }
    let f: fn(&str) = unsafe { std::mem::transmute(f) };
    f(msg);
}

/// Times `body` and reports it when tracing is on.
pub fn time<T>(label: &str, body: impl FnOnce() -> T) -> T {
    if !enabled() { return body(); }
    let t0 = now_ms();
    let out = body();
    emit(&format!("[perf] {label}: {:.1} ms", now_ms() - t0));
    out
}
