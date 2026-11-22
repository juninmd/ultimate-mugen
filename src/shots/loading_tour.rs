use super::{find, snap};
use crate::app::App;
use crate::gfx::*;

pub(super) async fn run(app: &mut App, dir: &str) {
    for zip in std::env::var("UME_LOADING_ZIPS").unwrap_or_default().split(':').filter(|z| !z.is_empty()) {
        if let Ok(bytes) = std::fs::read(zip) {
            let _ = app.reg.import_zip(&bytes);
        }
    }
    let mugen: Vec<usize> = (0..app.reg.chars.len()).filter(|&i| app.reg.chars[i].mugen).collect();
    app.team = None;
    app.sel.0 = mugen.first().copied().unwrap_or_else(|| find(app, "akio"));
    app.sel.1 = mugen.get(1).copied().unwrap_or_else(|| find(app, "lian"));
    for (name, step, progress) in [("loading-0", 0u8, 0.18), ("loading-1", 1, 0.55), ("loading-2", 2, 0.9)] {
        snap(app, dir, name, |app| {
            app.ui.begin();
            app.holo_loading_screen(step, progress, "Carregando…");
        })
        .await;
        for _ in 0..20 {
            next_frame().await;
        }
    }
    let total = app.reg.chars.len().max(1);
    let plan: Vec<(usize, usize, usize)> = (0..120).map(|k| (k % 2, 0, k % total)).collect();
    for (name, step) in [("mosaic-0", 0usize), ("mosaic-30", 30), ("mosaic-80", 80), ("mosaic-120", 120)] {
        snap(app, dir, name, |app| {
            app.ui.begin();
            app.holo_mosaic_screen(&plan, step, step as f32 / 120.0, "Carregando…");
        })
        .await;
    }
}

pub(super) async fn market_download(app: &mut App, dir: &str, repo: &str, ids: &[String]) {
    let key = |id: &str| format!("{repo}\u{0}{id}");
    app.market.queue.clear();
    app.market.progress.total_files = 9;
    for (i, id) in ids.iter().enumerate().take(2) {
        app.market.current = Some(key(id));
        app.market.progress.done_files = 2 + i;
        for _ in 0..3 {
            app.frame().await;
            next_frame().await;
        }
    }
    app.market.current = ids.get(2).map(|id| key(id));
    for id in ids.iter().skip(3).take(6) {
        app.market.queue.push_back((repo.to_string(), id.clone()));
    }
    super::screen(app, dir, "market-download", 6).await;
    app.market.current = None;
    app.market.queue.clear();
    app.frame().await;
}
