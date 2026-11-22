use super::screen;
use crate::app::{App, Screen};
use std::sync::Arc;
use ume_market::catalog::{detect, Tree, TreeEntry};
use ume_market::persist::{catalog_key, MemPersist, Persist, STATE_KEY};
use ume_market::view::Format;
use ume_market::{Installed, MarketState, RepoRef};

const MUGEN: [&str; 12] = ["Aquaman", "Batman_Rebirth", "BlackPanther", "BlackWidow", "CaptainAmerica", "Cell_Perfect_Form", "Goku_SS4", "Deadpool", "Ironman", "Spiderman", "Thanos", "Storm"];
const UMF: [&str; 2] = ["BlackWidow_UMF", "Goku_SSJ4_UMF"];
const STAGES: [&str; 2] = ["Academia_Ninja", "Campo3"];

fn tree() -> Tree {
    let mut entries = Vec::new();
    let mut add = |path: String, size: u64| {
        let sha = format!("{:040x}", path.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3)));
        entries.push(TreeEntry { path, size, sha });
    };
    for n in MUGEN {
        add(format!("chars/{n}.zip"), 6_000_000);
        add(format!("previews/chars/{n}.png"), 40_000);
    }
    for n in UMF {
        add(format!("umf/{n}.zip"), 9_000_000);
        add(format!("previews/umf/{n}.png"), 40_000);
    }
    for n in STAGES {
        add(format!("stages/{n}.zip"), 12_000_000);
        add(format!("previews/stages/{n}.png"), 60_000);
    }
    Tree { entries, truncated: false }
}

fn seed(app: &mut App) -> String {
    let repo = RepoRef::parse("juninmd/mugen-assets").unwrap();
    let catalog = detect(&tree(), "", &repo.name);
    let persist = Arc::new(MemPersist::new());
    let mut state = MarketState::default();
    state.add_repo(repo.clone());
    state.repo_mut(&repo.key()).unwrap().item_count = catalog.items.len();
    state.repo_mut(&repo.key()).unwrap().listed_at = ume_market::now_secs();
    for (name, version, active) in [("Aquaman", None, true), ("Goku_SS4", Some("stale"), true), ("BlackWidow_UMF", None, true), ("Storm", Some("stale"), false)] {
        let it = catalog.items.iter().find(|i| i.name == name).unwrap();
        persist.write(&ume_market::persist::item_key(&repo.key(), &it.id), &ume_market::bundle::encode(&vec![(it.files[0].path.clone(), vec![0u8])])).unwrap();
        state.upsert_installed(Installed { repo_key: repo.key(), item_id: it.id.clone(), kind: it.kind, name: it.name.clone(), bytes: it.bytes, files: 1, active, installed_at: 1, version: version.map_or(it.version.clone(), str::to_string) });
    }
    state.grid_view = true;
    if let Ok(dir) = std::env::var("UME_MARKET_PREVIEWS") {
        for e in catalog.support.iter().filter(|e| e.path.starts_with("previews/")) {
            if let Ok(bytes) = std::fs::read(format!("{dir}/{}", e.path.trim_start_matches("previews/"))) {
                persist.write(&format!("portrait/{}/{}", repo.key(), e.sha), &bytes).unwrap();
            }
        }
    }
    persist.write(&catalog_key(&repo.key()), &serde_json::to_vec(&catalog).unwrap()).unwrap();
    persist.write(STATE_KEY, &state.to_json()).unwrap();
    app.market = crate::market::Market::with_persist(persist);
    app.market.finish_hydration();
    app.market.poll();
    repo.key()
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    let key = seed(app);
    app.screen = Screen::Market;
    screen(app, dir, "market-grid", 90).await;
    app.market_ui_filter(Some(Format::Umf), false);
    screen(app, dir, "market-grid-umf", 20).await;
    app.market_ui_filter(None, true);
    screen(app, dir, "market-grid-outdated", 20).await;
    app.market_ui_filter(None, false);
    app.market.state.grid_view = false;
    screen(app, dir, "market-list", 20).await;
    app.market_ui_filter(Some(Format::Mugen), false);
    screen(app, dir, "market-list-mugen", 20).await;
    app.market_ui_filter(None, false);
    app.market_ui_mark(&key, "pack:umf/Goku_SSJ4_UMF.zip");
    app.market.state.grid_view = true;
    app.market_ui_view_installed();
    screen(app, dir, "market-grid-installed", 30).await;
}
