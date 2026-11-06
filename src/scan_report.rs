//! Headless startup probe (`--scan-report`): roster size, seconds and RSS without opening a window.

use crate::registry::{rss_mb, Registry};

pub fn run() -> ! {
    let clock = std::time::Instant::now();
    let mut reg = Registry::new();
    let builtin = clock.elapsed();
    reg.add_bundled_mugen();
    let bundled = clock.elapsed();
    reg.scan_folders();
    let scanned = clock.elapsed();
    reg.dedupe();
    reg.sort_mugen();
    println!(
        "bundled {:6.2} s | scan {:6.2} s | total {:6.2} s | {} chars, {} stages | rss {:.0} MB",
        builtin.as_secs_f64(),
        (scanned - bundled).as_secs_f64(),
        clock.elapsed().as_secs_f64(),
        reg.chars.len(),
        reg.stages.len(),
        rss_mb()
    );
    std::process::exit(0);
}
