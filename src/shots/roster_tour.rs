use super::{find, screen};
use crate::app::{App, Mode, Screen};
use crate::gfx::*;
use std::time::Instant;

fn rss_mb() -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok())
        })
        .map_or(0.0, |kb| kb / 1024.0)
}

fn percentile(samples: &mut [f64], p: f64) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    samples
        .get(((samples.len().max(1) - 1) as f64 * p) as usize)
        .copied()
        .unwrap_or(0.0)
}

async fn timed_frames(app: &mut App, label: &str, frames: usize, step: impl Fn(&mut App, usize)) {
    let mut samples = Vec::with_capacity(frames);
    for n in 0..frames {
        step(app, n);
        let t = Instant::now();
        app.frame().await;
        samples.push(t.elapsed().as_secs_f64() * 1000.0);
        next_frame().await;
    }
    let mean = samples.iter().sum::<f64>() / samples.len().max(1) as f64;
    let p95 = percentile(&mut samples, 0.95);
    let max = percentile(&mut samples, 1.0);
    println!("roster-perf: {label} · {frames} frames · cpu mean {mean:.2} ms · p95 {p95:.2} ms · max {max:.2} ms · rss {:.0} MB", rss_mb());
}

fn view_cost(app: &mut App, sort: usize) {
    app.settings.sort = sort;
    let base = app.playable();
    let runs = 200;
    let t = Instant::now();
    for _ in 0..runs {
        std::hint::black_box(app.compute_char_view(&base));
    }
    let uncached = t.elapsed().as_secs_f64() * 1000.0 / runs as f64;
    let _ = app.cached_char_view();
    let t = Instant::now();
    for _ in 0..runs {
        std::hint::black_box(app.cached_char_view());
    }
    let cached = t.elapsed().as_secs_f64() * 1000.0 / runs as f64;
    println!("roster-perf: char view sort {sort} · {} fighters · uncached {uncached:.3} ms · cached {cached:.3} ms", base.len());
}

fn select_at(app: &mut App, i: usize) {
    app.screen = Screen::Select {
        cur: [i, 0],
        picking: 0,
        picked: [None, None],
    };
}

pub(super) async fn run(app: &mut App, dir: &str) {
    println!(
        "roster-perf: {} fighters · {} stages · rss {:.0} MB",
        app.reg.chars.len(),
        app.reg.stages.len(),
        rss_mb()
    );
    app.mode = Mode::Cpu;
    for sort in 0..5 {
        view_cost(app, sort);
    }
    app.settings.sort = 0;
    let count = app.reg.chars.len();
    let walk = move |app: &mut App, n: usize| select_at(app, (n * 7) % count);
    select_at(app, find(app, "kaito"));
    timed_frames(app, "select idle", 120, |_, _| {}).await;
    timed_frames(app, "select walking 1000 fighters", 240, walk).await;
    select_at(app, count / 2);
    screen(app, dir, "roster-select-middle", 90).await;
    select_at(app, count - 1);
    screen(app, dir, "roster-select-end", 90).await;
    app.set_search_query("kor", false);
    select_at(app, find(app, "kaito"));
    screen(app, dir, "roster-search-grid", 60).await;
    app.set_search_query("warusaki", true);
    screen(app, dir, "roster-search-page", 20).await;
    app.set_search_query("", false);
    for k in [3usize, 10, 40, 77, 120, 300, 640] {
        let i = (k * 13) % count;
        if !app.is_favorite(i) {
            app.toggle_favorite(i);
        }
        app.note_recent((k * 31) % count);
    }
    select_at(app, 39);
    screen(app, dir, "roster-favorite-marks", 40).await;
    app.set_select_filter(4);
    select_at(app, 39);
    screen(app, dir, "roster-filter-favorites", 40).await;
    app.set_select_filter(5);
    screen(app, dir, "roster-filter-recent", 40).await;
    app.set_select_filter(0);
    app.set_author_filter(Some("Kong"));
    screen(app, dir, "roster-filter-author", 40).await;
    app.set_author_filter(None);
    app.mode = Mode::TeamSimul;
    app.team_n = 5;
    select_at(app, count / 3);
    screen(app, dir, "roster-team-middle", 60).await;
    app.mode = Mode::Cpu;
    let stages = app.reg.stages.len() + 1;
    app.screen = Screen::Stages { cur: 1 };
    timed_frames(app, "stages idle", 120, |_, _| {}).await;
    timed_frames(app, "stages walking 1000 stages", 240, move |app, n| {
        app.screen = Screen::Stages {
            cur: (n * 13) % stages,
        }
    })
    .await;
    app.screen = Screen::Stages { cur: stages / 2 };
    screen(app, dir, "roster-stages-middle", 90).await;
    app.set_stage_filter(2, "");
    app.screen = Screen::Stages { cur: 0 };
    screen(app, dir, "roster-stages-mugen", 90).await;
    app.set_stage_filter(0, "temple");
    screen(app, dir, "roster-stages-search", 90).await;
    app.set_stage_filter(0, "");
}
