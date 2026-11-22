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
}
