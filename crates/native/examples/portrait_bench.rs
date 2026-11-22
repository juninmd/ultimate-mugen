use std::sync::Arc;
use std::time::Instant;
use ume_native::{parse_char, register_native_warm, select_portrait};

fn rss_mb() -> f64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let kb = status
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))
        .and_then(|v| v.split_whitespace().next()?.parse::<f64>().ok());
    kb.unwrap_or(0.0) / 1024.0
}

fn main() {
    let mut chars: Vec<_> = std::fs::read_dir("assets/chars")
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| parse_char(&std::fs::read_to_string(e.path()).ok()?).ok())
        .collect();
    chars.sort_by(|a, b| a.id.cmp(&b.id));
    let warm = std::env::args().nth(1).is_some_and(|a| a == "warm");
    if warm {
        for c in &chars {
            register_native_warm(Arc::new(c.clone()));
        }
        std::thread::sleep(std::time::Duration::from_millis(600));
    }
    let base = rss_mb();
    let start = Instant::now();
    let total = chars.len();
    let found = chars
        .iter()
        .filter(|c| select_portrait(c, 1).is_some())
        .count();
    let cold = start.elapsed().as_secs_f64() * 1000.0;
    let after = rss_mb();
    let start = Instant::now();
    for c in &chars {
        select_portrait(c, 2);
    }
    let pal = start.elapsed().as_secs_f64() * 1000.0;
    println!("{total} chars, {found} portraits: cold {cold:.1} ms, palette swap {pal:.1} ms, rss {base:.0} -> {after:.0} MB");
}
