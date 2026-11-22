use super::*;
use std::time::{Duration, Instant};
use ume_market::catalog::{detect, Tree, TreeEntry};
use ume_market::MemPersist;

fn elecbyte_files() -> Files {
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, include_bytes!("../../assets/mugen/elecbyte.zip"), "").unwrap();
    vfs.into_iter().collect()
}

fn market_with_repo() -> (Market, String, Arc<MemPersist>) {
    let persist = Arc::new(MemPersist::new());
    let mut m = Market::with_persist(persist.clone());
    m.finish_hydration();
    m.state.add_repo(RepoRef::parse("a/b").unwrap());
    (m, "a/b".to_string(), persist)
}

fn character_item(files: &Files) -> Item {
    let tree = Tree {
        entries: files
            .iter()
            .map(|(p, d)| TreeEntry {
                path: p.clone(),
                size: d.len() as u64,
                ..Default::default()
            })
            .collect(),
        truncated: false,
    };
    detect(&tree, "", "kfm-pack")
        .items
        .into_iter()
        .find(|i| i.kind == Kind::Character)
        .expect("a character in the bundled pack")
}

fn wait_for_files(m: &mut Market) {
    let start = Instant::now();
    while !m.has_registry_work() && start.elapsed() < Duration::from_secs(10) {
        m.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn installed_character_is_registered_deactivated_and_reactivated_from_storage() {
    let files = elecbyte_files();
    let item = character_item(&files);
    let (mut m, repo_key, persist) = market_with_repo();
    let mut reg = Registry::new();
    let before = reg.chars.len();
    finish_install(
        &m.events,
        &(persist.clone() as Arc<dyn Persist>),
        repo_key.clone(),
        item.clone(),
        Ok(files),
    );
    m.poll();
    assert!(m.apply_to(&mut reg));
    let added = reg.chars.len() - before;
    assert!(added >= 1);
    let tag = m.state.installed_item(&repo_key, &item.id).unwrap().tag();
    assert!(reg.chars.iter().any(|c| c.id.contains(&tag)));

    m.set_active([(repo_key.as_str(), item.id.as_str())], false);
    assert!(m.apply_to(&mut reg));
    assert_eq!(reg.chars.len(), before);

    m.set_active([(repo_key.as_str(), item.id.as_str())], true);
    wait_for_files(&mut m);
    assert!(m.apply_to(&mut reg));
    assert_eq!(reg.chars.len(), before + added);
}

#[test]
fn state_survives_a_restart_and_active_items_come_back() {
    let files = elecbyte_files();
    let item = character_item(&files);
    let (mut m, repo_key, persist) = market_with_repo();
    finish_install(
        &m.events,
        &(persist.clone() as Arc<dyn Persist>),
        repo_key.clone(),
        item.clone(),
        Ok(files),
    );
    m.poll();
    let mut reborn = Market::with_persist(persist);
    reborn.finish_hydration();
    assert_eq!(reborn.state.installed.len(), 1);
    assert!(reborn.state.installed[0].active);
    wait_for_files(&mut reborn);
    let mut reg = Registry::new();
    let before = reg.chars.len();
    assert!(reborn.apply_to(&mut reg));
    assert!(reg.chars.len() > before);
}

#[test]
fn uninstall_removes_storage_and_registry_entries() {
    let files = elecbyte_files();
    let item = character_item(&files);
    let (mut m, repo_key, persist) = market_with_repo();
    let mut reg = Registry::new();
    let before = reg.chars.len();
    finish_install(
        &m.events,
        &(persist.clone() as Arc<dyn Persist>),
        repo_key.clone(),
        item.clone(),
        Ok(files),
    );
    m.poll();
    m.apply_to(&mut reg);
    m.uninstall(&repo_key, &item.id);
    assert!(m.apply_to(&mut reg));
    assert_eq!(reg.chars.len(), before);
    assert!(m.state.installed.is_empty());
    assert!(persist.read(&item_key(&repo_key, &item.id)).is_none());
}

#[test]
fn active_music_rotates_through_tracks() {
    let (mut m, repo_key, persist) = market_with_repo();
    for name in ["a", "b"] {
        let tree = Tree {
            entries: vec![TreeEntry {
                path: format!("music/{name}.ogg"),
                size: 5,
                ..Default::default()
            }],
            truncated: false,
        };
        let item = detect(&tree, "", "r").items.remove(0);
        finish_install(
            &m.events,
            &(persist.clone() as Arc<dyn Persist>),
            repo_key.clone(),
            item,
            Ok(vec![(
                format!("music/{name}.ogg"),
                vec![name.as_bytes()[0]],
            )]),
        );
    }
    m.poll();
    assert_eq!(m.active_tracks(), 2);
    let first = m.pick_track().unwrap();
    let second = m.pick_track().unwrap();
    let third = m.pick_track().unwrap();
    assert_ne!(first.0, second.0);
    assert_eq!(first.0, third.0);
    m.set_active([(repo_key.as_str(), "music:music/a.ogg")], false);
    assert_eq!(m.active_tracks(), 1);
}

#[test]
fn failed_listing_reports_the_error_and_unblocks_refresh() {
    let (mut m, repo_key, _) = market_with_repo();
    m.listing.insert(repo_key.clone());
    m.handle(Event::Listed {
        repo_key: repo_key.clone(),
        result: Err("sem rede".into()),
    });
    assert!(m.notice_error && m.notice == "sem rede");
    assert!(!m.listing.contains(&repo_key));
}

#[test]
fn add_repo_rejects_foreign_hosts() {
    let (mut m, _, _) = market_with_repo();
    assert!(m.add_repo("https://gitlab.com/a/b").is_err());
    assert!(m.add_repo("javascript:alert(1)").is_err());
    assert_eq!(m.state.repos.len(), 1);
}

#[test]
fn rate_limit_message_mentions_the_wait() {
    let msg = describe(&MarketError::RateLimited {
        reset_epoch: Some(ume_market::now_secs() + 600),
    });
    assert!(msg.contains("min") && msg.contains("token"));
}

#[test]
fn portrait_loads_from_the_cache_and_is_served_again_without_a_new_request() {
    let (mut m, repo_key, persist) = market_with_repo();
    let png = include_bytes!("../../assets/ui/hud-guard-shield.png");
    let support = vec![TreeEntry { path: "previews/chars/Goku.png".into(), size: png.len() as u64, sha: "abc123".into() }];
    let item = Item { id: "pack:chars/Goku.zip".into(), kind: Kind::Pack, name: "Goku".into(), files: vec![], bytes: 1, installable: true, version: "v".into() };
    m.catalogs.insert(repo_key.clone(), Catalog { items: vec![item], truncated: false, support });
    persist.write(&format!("portrait/{repo_key}/abc123"), png).unwrap();
    assert!(m.portrait(&repo_key, "pack:chars/Goku.zip").is_none());
    let start = Instant::now();
    let art = loop {
        m.poll();
        if let Some(art) = m.portrait(&repo_key, "pack:chars/Goku.zip") {
            break art;
        }
        assert!(start.elapsed() < Duration::from_secs(10), "portrait never became ready");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!((art.width, art.height, art.rgba.len()), (48, 24, 48 * 24 * 4));
    assert!(Arc::ptr_eq(&art.rgba, &m.portrait(&repo_key, "pack:chars/Goku.zip").unwrap().rgba));
    assert!(m.portrait(&repo_key, "pack:chars/Missing.zip").is_none());
}

#[test]
fn large_portraits_are_halved_by_averaging_two_by_two_blocks() {
    let mut px = vec![0u8; 4 * 2 * 4];
    for (i, v) in [10u8, 20, 30, 40].into_iter().enumerate() {
        let (x, y) = (i % 2, i / 2);
        px[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4].copy_from_slice(&[v, v, v, 255]);
    }
    let (w, h, out) = portrait::shrink(&px, 4, 2);
    assert_eq!((w, h, out.len()), (2, 1, 8));
    assert_eq!(&out[..4], &[25, 25, 25, 255]);
    assert_eq!(&out[4..], &[0, 0, 0, 0]);
}
